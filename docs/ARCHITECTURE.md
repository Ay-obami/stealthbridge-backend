# StealthBridge Backend Architecture v0.2

## Scope
A multi-tenant, corridor-oriented settlement orchestration layer for two distinct user experiences, **Business** (known counterparties; confidential amounts) and **Send** (private counterparty links). Not a privacy pool or custody provider itself.

## Services (future boundaries)
- **API:** authentication and tenant scope, wallet challenge verification, idempotency checks, rate limits, signatures, HTTP contracts.
- **Settlement engine:** workflow state machine with durable outbox and idempotent provider calls.
- **Ledger adapter/indexer:** observe Stellar RPC network events, finality, payment receipts and contract execution errors. Never assume events expose confidential amount.
- **Corridor policy:** asset / country / network compatibility and permissions; configurable by tenant and jurisdiction.
- **Privacy adapters:** two separate implementations (Confidential Tokens, SPP); not a common proof format.
- **FX & payouts:** provider quotes with expiration and minimum received, partner payout acknowledgements, eventually transaction-safe saga compensation.
- **Compliance:** maintain restricted partner references and screening outcomes off-chain, generate scoped assertions for protocol where supported.

## Tenancy and scale boundaries
\`\`\`
tenant -> organizations -> corridors -> settlements -> settlement_attempts
                                |                 -> chain_observations
                                -> quotes         -> payout_attempts
\`\`\`
Every read/write must be tenant-scoped; don't rely solely on filtering at the UI. Global contract state must not leak financial metadata across tenants.

## Exact lifecycle
draft → quoted → authorized → submitted → chain_finalized → payout_pending → payout_completed

Exceptional transitions include expired, rejected, chain_failed, payout_failed, refund_pending, refunded and manual_review. See \`src/settlement.rs\` for current local transition predicate. **It is a pure model**, not persistence or an executable settlement service.

## Invariants
1. No mutable payment quote after authorization (create a new revision).
2. No payout before sufficient chain finality AND required policy/FX controls.
3. Never infer payout success from a Stellar transaction hash.
4. Retry by idempotency key, tenant ID, and provider's stable payout ID.
5. Store secrets/witnesses in user's wallet or approved isolated secure environment, not app logs.
6. Separate public ledger evidence, confidential proof inputs, and regulator-access data stores.
7. A real-world settlement needs documented cancellation/refund semantics and partner contracts.

## Recommended data store (not yet implemented)
PostgreSQL with immutable settlement_status_history and partitioned large ledger_observation tables; transactional outbox for asynchronous jobs; deduped webhook inbox. Do not store fake balances or mark simulated quotes as live market rates.

## First scaling experiments
Index replay under duplicate events; concurrent same-idempotency submission; payout callback reordering; 100 parallel settlements in simulated mode; ledger RPC recovery and delayed-finality tests. Document actual latencies rather than claiming throughput.
