//! Transactional, organization-scoped journal. No public write routes or
//! financial amounts are exposed. Call this only from future authenticated
//! orchestration code after explicit policy and proof verification.

use crate::settlement::{transition, SettlementState};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug)]
pub enum JournalError {
    InvalidInput,
    NotFound,
    PayloadConflict,
    VersionConflict,
    InvalidTransition,
    Storage(sqlx::Error),
}
impl From<sqlx::Error> for JournalError {
    fn from(value: sqlx::Error) -> Self { Self::Storage(value) }
}

pub struct CreateResult { pub id: Uuid, pub created: bool }

pub fn validated_key_and_digest(key: &str, digest: &str) -> bool {
    (16..=128).contains(&key.len())
        && key.bytes().all(|b| b.is_ascii_graphic())
        && digest.len() == 64
        && digest.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Concurrent calls with the same organization/key converge to one intent.
/// A different digest for an existing key is a hard conflict, never a second intent.
pub async fn register_intent(
    pool: &PgPool, organization_id: Uuid, corridor_id: Uuid,
    key: &str, request_sha256: &str,
) -> Result<CreateResult, JournalError> {
    if !validated_key_and_digest(key, request_sha256) { return Err(JournalError::InvalidInput); }
    let mut tx = pool.begin().await?;
    let candidate = Uuid::new_v4();
    let row = sqlx::query(
        "INSERT INTO settlement_intents \
         (id, organization_id, corridor_id, idempotency_key, request_sha256) \
         VALUES ($1,$2,$3,$4,$5) \
         ON CONFLICT (organization_id,idempotency_key) DO NOTHING RETURNING id"
    )
    .bind(candidate).bind(organization_id).bind(corridor_id)
    .bind(key).bind(request_sha256)
    .fetch_optional(&mut *tx).await?;
    let created = row.is_some();
    let found = sqlx::query(
        "SELECT id, request_sha256, corridor_id FROM settlement_intents \
         WHERE organization_id=$1 AND idempotency_key=$2 FOR UPDATE"
    ).bind(organization_id).bind(key).fetch_one(&mut *tx).await?;
    let id: Uuid = found.try_get("id")?;
    let existing_digest: String = found.try_get("request_sha256")?;
    let existing_corridor: Uuid = found.try_get("corridor_id")?;
    if existing_digest != request_sha256 || existing_corridor != corridor_id {
        return Err(JournalError::PayloadConflict);
    }
    if created {
        sqlx::query("INSERT INTO settlement_transition_log \
            (organization_id, settlement_id, previous_state, next_state, version) \
            VALUES ($1,$2,NULL,'draft',0)")
            .bind(organization_id).bind(id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(CreateResult {id,created})
}

pub async fn advance_intent(
    pool: &PgPool, organization_id: Uuid, id: Uuid,
    expected_version: i64, next: SettlementState,
) -> Result<i64, JournalError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        "SELECT state, version FROM settlement_intents \
         WHERE organization_id=$1 AND id=$2 FOR UPDATE"
    ).bind(organization_id).bind(id).fetch_optional(&mut *tx).await?
        .ok_or(JournalError::NotFound)?;
    let state_name: String = row.try_get("state")?;
    let version: i64 = row.try_get("version")?;
    if version != expected_version { return Err(JournalError::VersionConflict); }
    let current = SettlementState::parse(&state_name).ok_or(JournalError::InvalidInput)?;
    transition(current,next).map_err(|_| JournalError::InvalidTransition)?;
    let next_version = version.checked_add(1).ok_or(JournalError::VersionConflict)?;
    sqlx::query("UPDATE settlement_intents SET state=$1,version=$2,updated_at=now() \
        WHERE organization_id=$3 AND id=$4")
        .bind(next.as_str()).bind(next_version).bind(organization_id).bind(id)
        .execute(&mut *tx).await?;
    sqlx::query("INSERT INTO settlement_transition_log \
        (organization_id, settlement_id, previous_state, next_state, version) \
        VALUES ($1,$2,$3,$4,$5)")
        .bind(organization_id).bind(id).bind(current.as_str()).bind(next.as_str())
        .bind(next_version).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(next_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_requirements_prevent_ambiguous_idempotency() {
        assert!(validated_key_and_digest("org-scope-request-123",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"));
        assert!(!validated_key_and_digest("short","0".repeat(64).as_str()));
        assert!(!validated_key_and_digest("org-scope-request-123","A".repeat(64).as_str()));
    }
}
