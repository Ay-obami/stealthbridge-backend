//! Verified-notification inbox foundations, NOT a live provider callback API.
//! The tenant/provider/secret are trusted operator configuration, not values
//! supplied by unauthenticated HTTP body fields.
use hmac::{Hmac,Mac};
use sha2::{Sha256,Digest};
use serde::Deserialize;
use sqlx::{PgPool,Row};
use uuid::Uuid;

type HmacSha256=Hmac<Sha256>;
const MAX_BYTES:usize=65_536;
const MAX_SKEW:i64=300;
const DOMAIN:&[u8]=b"stealthbridge-webhook-v1\n";

#[derive(Debug,PartialEq,Eq)]
pub enum VerifyError{InvalidPayload,BadSignature,Expired,WeakKey}
#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope{event_id:String,event_type:String}
#[derive(Clone,Debug)]
pub struct VerifiedEvent{
 pub event_id:String,
 pub event_type:String,
 pub payload_sha256:String,
}
fn id_ok(s:&str)->bool{
 (8..=128).contains(&s.len()) &&
  s.bytes().all(|b|b.is_ascii_alphanumeric() || b":_-".contains(&b))
}
fn type_ok(s:&str)->bool{
 (3..=128).contains(&s.len()) &&
  s.bytes().all(|b|b.is_ascii_alphanumeric() || b":._-".contains(&b))
}
pub fn verify_event(
 raw:&[u8], timestamp:&str, signature_hex:&str,
 secret:&[u8], now_unix:i64,
)->Result<VerifiedEvent,VerifyError>{
 if secret.len()<32{return Err(VerifyError::WeakKey);}
 if raw.is_empty() || raw.len()>MAX_BYTES || timestamp.is_empty() ||
    timestamp.len()>20 || !timestamp.bytes().all(|b|b.is_ascii_digit()) ||
    signature_hex.len()!=64 || !signature_hex.bytes().all(|b|b.is_ascii_hexdigit()){
    return Err(VerifyError::InvalidPayload);
 }
 let timestamp_sec=timestamp.parse::<i64>().map_err(|_|VerifyError::InvalidPayload)?;
 let skew=now_unix.checked_sub(timestamp_sec).ok_or(VerifyError::Expired)?;
 if !(-MAX_SKEW..=MAX_SKEW).contains(&skew){return Err(VerifyError::Expired);}
 let signature=hex::decode(signature_hex).map_err(|_|VerifyError::BadSignature)?;
 let mut mac=HmacSha256::new_from_slice(secret).map_err(|_|VerifyError::WeakKey)?;
 mac.update(DOMAIN);
 mac.update(timestamp.as_bytes());
 mac.update(b"\n");
 mac.update(raw);
 mac.verify_slice(&signature).map_err(|_|VerifyError::BadSignature)?;
 // Payload deserialization occurs only after authentication.
 let data:Envelope=serde_json::from_slice(raw).map_err(|_|VerifyError::InvalidPayload)?;
 if !id_ok(&data.event_id)||!type_ok(&data.event_type){
    return Err(VerifyError::InvalidPayload);
 }
 Ok(VerifiedEvent{
    event_id:data.event_id,
    event_type:data.event_type,
    payload_sha256:format!("{:x}",Sha256::digest(raw)),
 })
}
#[derive(Debug)]
pub enum InboxError{Collision,InvalidIdentifier,Database(sqlx::Error)}
impl From<sqlx::Error> for InboxError{
 fn from(e:sqlx::Error)->Self{Self::Database(e)}
}
#[derive(Debug,PartialEq,Eq)]
pub enum InboxResult{Inserted,Duplicate}

/// Store only authenticated, tenant-scoped metadata and content digest.
/// Does not emit transactions, execute payouts or mutate settlement status.
pub async fn record_event(
 pool:&PgPool,tenant:Uuid,provider:&str,event:&VerifiedEvent,
)->Result<InboxResult,InboxError>{
 if !id_ok(provider)||!id_ok(&event.event_id)||!type_ok(&event.event_type)||
    event.payload_sha256.len()!=64 ||
    !event.payload_sha256.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){
    return Err(InboxError::InvalidIdentifier);
 }
 let mut tx=pool.begin().await?;
 let created=sqlx::query(
  "INSERT INTO provider_event_inbox(organization_id,provider,event_id,event_type,payload_sha256) \
   VALUES ($1,$2,$3,$4,$5) \
   ON CONFLICT(organization_id,provider,event_id) DO NOTHING RETURNING event_id"
 ).bind(tenant).bind(provider).bind(&event.event_id).bind(&event.event_type)
  .bind(&event.payload_sha256).fetch_optional(&mut *tx).await?;
 if created.is_some(){
    tx.commit().await?;
    return Ok(InboxResult::Inserted);
 }
 let prior=sqlx::query(
  "SELECT payload_sha256,event_type FROM provider_event_inbox \
   WHERE organization_id=$1 AND provider=$2 AND event_id=$3 FOR UPDATE"
 ).bind(tenant).bind(provider).bind(&event.event_id).fetch_one(&mut *tx).await?;
 let digest:String=prior.try_get("payload_sha256")?;
 let kind:String=prior.try_get("event_type")?;
 if digest!=event.payload_sha256||kind!=event.event_type{return Err(InboxError::Collision);}
 tx.commit().await?;
 Ok(InboxResult::Duplicate)
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn signature_tampering_and_expired_delivery_rejected(){
    let key=b"unit-test-only-configured-secret-must-not-be-reused";
    let body=br#"{"event_id":"fixture-event-001","event_type":"fixture.notice"}"#;
    let timestamp="1760000000";
    let mut mac=HmacSha256::new_from_slice(key).unwrap();
    mac.update(DOMAIN);mac.update(timestamp.as_bytes());mac.update(b"\n");mac.update(body);
    let signature=hex::encode(mac.finalize().into_bytes());
    let result=verify_event(body,timestamp,&signature,key,1_760_000_100).unwrap();
    assert_eq!(result.event_id,"fixture-event-001");
    assert_eq!(result.event_type,"fixture.notice");
    assert_eq!(result.payload_sha256.len(),64);
    assert!(matches!(verify_event(body,timestamp,&signature,key,1_760_000_400),Err(VerifyError::Expired)));
    assert!(matches!(verify_event(b"malicious",timestamp,&signature,key,1_760_000_100),Err(VerifyError::BadSignature)));
    assert!(matches!(verify_event(body,timestamp,&"0".repeat(64),key,1_760_000_100),Err(VerifyError::BadSignature)));
    assert!(matches!(verify_event(body,timestamp,&signature,b"short",1_760_000_100),Err(VerifyError::WeakKey)));
 }
}
