# Organizational identity and authorization foundations

StealthBridge now defines minimal internal organization/member storage and a pure Rust role-permission matrix. It is **not a production sign-in system** and does not make payments available.

## Schema

The \`organizations\` table stores an opaque UUID, non-sensitive display name, and pending/active/suspended state. \`organization_members\` ties opaque subject identifiers to roles through a composite tenant key, bounded identifier length, and optional revocation timestamp. No customer records, login credentials, wallet seed, financial amounts, counterparties, or KYC documents are seeded.

A real sign-in implementation must independently prove subject ownership (e.g. a verified wallet challenge bound to nonce, domain, passphrase and expiry), associate the verified subject with an active member row in **the selected tenant**, check organization status, and apply database isolation. A request-provided organization ID or role string never constitutes permission.

## Role policy

| Role | Read | Create draft | Approve (another initiator) | Submit previously authorized intent | Manage members/corridors |
|:--|:--:|:--:|:--:|:--:|:--:|
| Owner | Yes | Yes | Yes | Yes | Yes |
| Admin | Yes | Yes | Yes | Yes | Yes |
| Operator | Yes | Yes | No | Yes | No |
| Approver | Yes | No | Yes | No | No |
| Viewer | Yes | No | No | No | No |

The pure \`check_distinct_approver\` helper rejects a matching initiator/approver and invalid subject strings. It **only checks preconditions**: a real command would additionally need a signed authorization tied to immutable quote/digest, externally verified organization membership, policy limits, network, proof and asynchronous settlement state.

## Not implemented yet

- Wallet/WebAuthn challenge-response and session issuance.
- Database row-level security, trusted database principals and audited service roles.
- Quorum/multi-signature approval records, replay protection and approval expiry.
- Public authenticated organization routes.
- Verified legal entity onboarding or fiat financial service eligibility.

Until these are implemented and independently tested, \`POST /v1/settlements\` remains disabled and no role policy may be used to bypass it. The next meaningful integration is a tenant-scoped authenticated identity verifier, not a claim that the roles already grant real authority.

## PostgreSQL integrity tests

`tests/organization_schema.rs` uses an isolated CI PostgreSQL service to verify scoped primary keys, valid/invalid roles, subject length checks, foreign-key enforcement, revocation timestamp constraints, active membership isolation and restricted tenant deletion. It inserts only synthetic, explicitly test-labeled records and cleans them up. Passing these tests **does not establish wallet identity verification, database row-level security or permission to move funds**.


## DB-enforced membership guard

\`authz::authorize_member(pool, organization_uuid, verified_subject, permission)\` now queries active organization status and non-revoked membership for the **exact** tenant, maps only recognized roles, then checks the permission matrix. It denies unknown tenants, suspended organizations, revoked members and unauthorized actions. PostgreSQL failures remain distinct errors, never implicit permissions.

**Important:** This internal read-only guard presupposes the subject's identity has already been cryptographically proven by a trusted auth layer. No public session/signature authentication exists yet. For future financial mutations the membership row must be locked and checked in the **same transaction** as the intended state change, with immutable signed approvals bound to the request digest, expiry, signer and tenant. This function alone does not meet that requirement.

CI uses a disposable PostgreSQL database to verify isolation, revocation, inactive organizations and permission boundaries.
