//! Testnet-only durable ledger-head cursor for an opt-in read-only observer.
//! A stored ledger head is not a payment event, payout, or private proof.
use serde::Serialize;
use sqlx::{PgPool, Row};

const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

#[derive(Debug, Clone)]
pub struct VerifiedHead {
    pub passphrase: String,
    pub ledger_sequence: u64,
    pub ledger_hash: String,
    pub ledger_closed_at_unix: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct LedgerCheckpoint {
    pub ledger_sequence: u64,
    pub ledger_hash: String,
    pub ledger_closed_at_unix: String,
    pub source: &'static str,
}
#[derive(Debug, PartialEq, Eq)]
pub enum CheckpointChange { Inserted, Advanced, Unchanged, Stale }
#[derive(Debug)]
pub enum LedgerError { InvalidHead, ConflictingHash, Storage(sqlx::Error) }
impl From<sqlx::Error> for LedgerError {
    fn from(error: sqlx::Error) -> Self { Self::Storage(error) }
}
impl VerifiedHead {
    pub fn validate(&self) -> Result<(), LedgerError> {
        if self.passphrase != TESTNET_PASSPHRASE || self.ledger_sequence == 0 ||
            self.ledger_sequence > i64::MAX as u64 ||
            self.ledger_hash.len() != 64 ||
            !self.ledger_hash.bytes().all(|b| b.is_ascii_hexdigit()) ||
            self.ledger_closed_at_unix.is_empty() ||
            self.ledger_closed_at_unix.len() > 20 ||
            !self.ledger_closed_at_unix.bytes().all(|b| b.is_ascii_digit()) {
            return Err(LedgerError::InvalidHead);
        }
        Ok(())
    }
}

/// Transactional monotonic checkpointing; a same-sequence hash disagreement is
/// always surfaced for operator investigation, never silently overwritten.
pub async fn record_head(pool: &PgPool, head: &VerifiedHead) -> Result<CheckpointChange, LedgerError> {
    head.validate()?;
    let mut tx=pool.begin().await?;
    let seq=head.ledger_sequence as i64;
    let hash=head.ledger_hash.to_ascii_lowercase();
    let inserted=sqlx::query(
        "INSERT INTO stellar_ledger_observer (network,source,ledger_sequence,ledger_hash,close_time_unix) \
         VALUES ('testnet','stellar-rpc',$1,$2,$3) \
         ON CONFLICT (network,source) DO NOTHING RETURNING ledger_sequence"
    ).bind(seq).bind(&hash).bind(&head.ledger_closed_at_unix)
        .fetch_optional(&mut *tx).await?;
    if inserted.is_some() {
        tx.commit().await?;
        return Ok(CheckpointChange::Inserted);
    }
    let row=sqlx::query(
        "SELECT ledger_sequence,ledger_hash FROM stellar_ledger_observer \
         WHERE network='testnet' AND source='stellar-rpc' FOR UPDATE"
    ).fetch_one(&mut *tx).await?;
    let existing_seq:i64=row.try_get("ledger_sequence")?;
    let existing_hash:String=row.try_get("ledger_hash")?;
    if seq==existing_seq && hash!=existing_hash {
        return Err(LedgerError::ConflictingHash);
    }
    if seq<existing_seq {return Ok(CheckpointChange::Stale);}
    if seq==existing_seq {return Ok(CheckpointChange::Unchanged);}
    sqlx::query(
        "UPDATE stellar_ledger_observer SET ledger_sequence=$1,ledger_hash=$2,close_time_unix=$3, \
         observed_at=now() WHERE network='testnet' AND source='stellar-rpc'"
    ).bind(seq).bind(&hash).bind(&head.ledger_closed_at_unix)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(CheckpointChange::Advanced)
}

/// Last persisted read-only observation; absence is not silently replaced with
/// latest network data or a simulated ledger.
pub async fn last_head(pool:&PgPool)->Result<Option<LedgerCheckpoint>,sqlx::Error>{
    let row=sqlx::query(
        "SELECT ledger_sequence,ledger_hash,close_time_unix FROM stellar_ledger_observer \
         WHERE network='testnet' AND source='stellar-rpc'"
    ).fetch_optional(pool).await?;
    match row {
        Some(row)=>{
            let ledger_sequence:i64=row.try_get("ledger_sequence")?;
            Ok(Some(LedgerCheckpoint {
                ledger_sequence: ledger_sequence as u64,
                ledger_hash:row.try_get("ledger_hash")?,
                ledger_closed_at_unix:row.try_get("close_time_unix")?,
                source:"stellar-rpc",
            }))
        }
        None=>Ok(None),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->VerifiedHead {
        VerifiedHead {
            passphrase:TESTNET_PASSPHRASE.to_owned(),
            ledger_sequence:42,
            ledger_hash:"0".repeat(64),
            ledger_closed_at_unix:"1760000000".to_owned(),
        }
    }
    #[test]
    fn fails_closed_on_network_or_untrustworthy_hash() {
        assert!(fixture().validate().is_ok());
        let mut bad=fixture();
        bad.passphrase="Public Global Stellar Network ; September 2015".to_owned();
        assert!(matches!(bad.validate(),Err(LedgerError::InvalidHead)));
        bad=fixture(); bad.ledger_hash="g".repeat(64);
        assert!(matches!(bad.validate(),Err(LedgerError::InvalidHead)));
        bad=fixture(); bad.ledger_sequence=0;
        assert!(matches!(bad.validate(),Err(LedgerError::InvalidHead)));
        bad=fixture(); bad.ledger_closed_at_unix="yesterday".to_owned();
        assert!(matches!(bad.validate(),Err(LedgerError::InvalidHead)));
    }
}
