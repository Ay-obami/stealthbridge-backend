# Provider Callback Verification — Internal v1

StealthBridge now has a provider-neutral signed notification verifier and durable deduplication inbox. **It is not a payment connector, bank integration, payout confirmation service or public HTTP endpoint.** No provider is onboarded and no settlement transitions are permitted from this module.

## HMAC verification

An operator-managed integration must resolve the trusted tenant UUID, provider identifier, signing key and expected timestamp format **outside the untrusted callback body**. The internal v1 verifier authenticates exactly these bytes in order:

1. ASCII domain separator `stealthbridge-webhook-v1` followed by a newline
2. Decimal Unix timestamp string followed by a newline
3. The raw, unmodified HTTP body bytes

The SHA-256 HMAC is supplied as 64 hex characters and compared in constant time. The signing key must contain at least 32 bytes from a trusted secret store. The receiver rejects stale or future-dated timestamps outside ±300 seconds, payloads larger than 64 KiB, malformed event IDs/types, invalid signatures and malformed JSON. JSON parsing happens only after verification. **These rules describe our internal test protocol; a real provider may mandate a different signature format and must be integrated against its own authenticated specification.**

## Durable inbox

PostgreSQL migration `202610080005_provider_inbox.sql` adds a composite unique key on `(organization_id, provider, event_id)`. It saves only the event ID, type, SHA-256 of authenticated raw bytes and reception time, never raw customer data. Two notifications with the same scoped ID and same authenticated event hash are idempotent. Reuse of that ID with a different type or hash is an explicit collision; it is never treated as a new payout.

## Security boundaries

- An authenticated callback is not proof of settled fiat funds. It is merely a signed claim that requires later independent reconciliation.
- The module has no Axum write route, third-party credentials, partner name, provider token or ability to transition payment state.
- Tenant/member authentication, partner registration, key rotation, proxy/body handling, ordering, retry/replay policy, retention and audit must be separately reviewed.
- PostgreSQL fixtures and HMAC signatures in tests are synthetic security fixtures, not invented production events.

Run `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings`; migrations execute against the isolated CI PostgreSQL service. This does not replace an external security assessment.
