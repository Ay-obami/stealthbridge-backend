# StealthBridge Backend

**Status: testnet architecture scaffold. Not a deployed payment service.**

Rust/Axum API, settlement state machine, Stellar indexer/reconciliation, mock FX quotes and simulated payout provider.

## Ownership
- `services/api`: OpenAPI HTTP layer and authentication
- `services/settlement-engine`: transition rules, idempotency and reconciliation
- `services/indexer`: ledger observation and confirmation reconciliation
- `integrations/mock-fx` and `integrations/mock-payout`: testnet-only simulations
- `api/openapi.yaml`: versioned API contract consumed by frontend

API and ledger finality are distinct from fiat payout completion. Do not store spend keys, seeds or ZK witnesses.

See https://github.com/stealthbridge-labs/stealthbridge-sdk and https://github.com/stealthbridge-labs/stealthbridge-contracts.
