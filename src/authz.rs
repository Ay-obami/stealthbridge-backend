//! Tenant role-policy primitives, deliberately separate from authentication.
//! These helpers do not verify a wallet, session, organization membership,
//! proof, jurisdiction or a payment signature. Never call them as a substitute
//! for a trusted, signed identity and a tenant-scoped database lookup.

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Role { Owner, Admin, Operator, Approver, Viewer }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Permission {
    ReadSettlement, ReadAudit, CreateDraft,
    ApproveIntent, SubmitAuthorizedIntent, ManageMembers, ManageCorridors,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum AccessError { Forbidden, SameInitiator, InvalidSubject }

impl Role {
    pub fn from_db(value:&str)->Option<Self>{
        Some(match value {
            "owner"=>Self::Owner,
            "admin"=>Self::Admin,
            "operator"=>Self::Operator,
            "approver"=>Self::Approver,
            "viewer"=>Self::Viewer,
            _=>return None
        })
    }
}
pub fn permits(role:Role,permission:Permission)->bool {
    use Permission::*;
    use Role::*;
    match permission {
        ReadSettlement | ReadAudit => true,
        CreateDraft | SubmitAuthorizedIntent =>
            matches!(role,Owner|Admin|Operator),
        ApproveIntent => matches!(role,Owner|Admin|Approver),
        ManageMembers => matches!(role,Owner|Admin),
        ManageCorridors => matches!(role,Owner|Admin),
    }
}

/// A permission-only precondition, NOT a signed payment approval.
/// A real caller still must verify the member's identity, active DB role,
/// quote digest, proof validity and an independently approved settlement.
pub fn check_distinct_approver(
    role:Role,initiator_subject:&str,approver_subject:&str,
)->Result<(),AccessError>{
    if initiator_subject.is_empty() || approver_subject.is_empty() ||
       initiator_subject.len()>256 || approver_subject.len()>256 {
        return Err(AccessError::InvalidSubject);
    }
    if !permits(role,Permission::ApproveIntent) {
        return Err(AccessError::Forbidden);
    }
    if initiator_subject==approver_subject {
        return Err(AccessError::SameInitiator);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn role_policy_is_deny_by_default_for_sensitive_operations(){
        assert!(!permits(Role::Viewer,Permission::SubmitAuthorizedIntent));
        assert!(!permits(Role::Viewer,Permission::ApproveIntent));
        assert!(!permits(Role::Operator,Permission::ApproveIntent));
        assert!(!permits(Role::Approver,Permission::SubmitAuthorizedIntent));
        assert!(permits(Role::Operator,Permission::CreateDraft));
        assert!(permits(Role::Approver,Permission::ApproveIntent));
        assert!(permits(Role::Admin,Permission::ManageMembers));
        assert!(!permits(Role::Viewer,Permission::ManageCorridors));
        assert_eq!(Role::from_db("root"),None);
        assert_eq!(Role::from_db("owner"),Some(Role::Owner));
    }
    #[test]
    fn a_payment_initiator_cannot_self_approve(){
        assert_eq!(check_distinct_approver(Role::Admin,"member-123","member-123"),
          Err(AccessError::SameInitiator));
        assert_eq!(check_distinct_approver(Role::Operator,"member-a","member-b"),
          Err(AccessError::Forbidden));
        assert_eq!(check_distinct_approver(Role::Approver,"member-a","member-b"),Ok(()));
        assert_eq!(check_distinct_approver(Role::Owner,"","member-b"),
          Err(AccessError::InvalidSubject));
    }
}
