<div align="center"><img src="assets/stealthbridge-logo.svg" alt="StealthBridge — Confidential payments. Without borders." width="760" /></div>

# StealthBridge Backend

**Engineering roadmap:** [View the repository-specific plan](ROADMAP.md).

Real-time, **read-only** Stellar Testnet observation API, a real transaction hash lookup, a tenant-scoped internal settlement journal, and an unseeded operator-managed corridor catalog. Rust, Axum, PostgreSQL, Stellar RPC.

**Live data, not demo records.** `GET /v1/network` makes actual `getNetwork` and `getLatestLedger` RPC requests. `GET /v1/corridors` reads enabled corridors from PostgreSQL; if no database is configured it clearly returns 503 instead of making them up. `POST /v1/settlements` is disabled (501) until cryptographic proof and custody requirements are satisfied.

## Run
Requires Rust and dependencies. No account secret or key required for RPC reads.

```sh
cargo test
cargo run
curl http://127.0.0.1:8080/v1/network
```

For real corridor records, set a PostgreSQL `DATABASE_URL` and apply `migrations/` through your approved deployment environment. No example corridor data is inserted.

## Architecture
- [API specification](api/openapi.yaml)
- [Deployment and environment](docs/DEPLOYMENT.md)
- [Settlement state machine](src/settlement.rs)
- [Transactional journal](docs/SETTLEMENT-JOURNAL.md)
- [Public transaction observation](docs/TRANSACTION-OBSERVATION.md)
- [Service architecture](docs/ARCHITECTURE.md)

*Status:* no confidential transfers, stablecoin issuance, FX rates, wallet signing, fiat payouts or proof generation wired yet. Do not process real money. Network is restricted to Testnet passphrase.

## Other repositories
[Frontend](https://github.com/stealthbridge-labs/stealthbridge-frontend) · [Contracts](https://github.com/stealthbridge-labs/stealthbridge-contracts) · [SDK](https://github.com/stealthbridge-labs/stealthbridge-sdk)

## Exact asset amounts and readiness

- `src/amount.rs` provides checked fixed-precision `i128` minor-unit arithmetic, explicit asset/network identity and strict decimal parsing. It uses no floating-point financial math. Asset decimals must come from verified issuer/chain metadata; this module does not discover or trust a stablecoin by itself.
- `GET /health` reports process liveness only.
- `GET /ready` probes actual Stellar RPC **and** configured PostgreSQL with bounded upstream calls; 200 requires both, otherwise 503 with a non-sensitive degraded status. It **always** reports `payments=disabled`, which is different from network readiness.
- Backend internal settlement state transitions are still not public payment APIs and require policy, authentication and privacy-proof verification before money movement.

See [service readiness and exact asset value notes](docs/ENGINEERING-FOUNDATIONS.md).

### Corridor lookup

`GET /v1/corridors/{id}` selects one **enabled** operator-configured record by UUID with a parameterized PostgreSQL query. Invalid UUIDs return 400, unavailable/disabled records 404 and a missing database 503. No country, asset identity, rate, issuer relationship or payout availability is synthesized. The SDK exposes the same typed read.

## Detailed integration guide

[Deployment and API integration reference](docs/INTEGRATION-GUIDE.md) documents every read-only route, failure status, database boundary and release precondition.

## Bounded corridor keyset discovery

\`GET /v1/corridors/page?limit=25&after=<UUID>\` returns \`{items, next_cursor}\` from **real enabled operator records** in PostgreSQL. Limits range 1–100, default 25; malformed UUID or out-of-range count returns 400. The query fetches one extra row to determine whether a cursor should be returned, so the service never reads the full table to produce a page. Stable UUID ordering avoids OFFSET scans at larger tables. A missing database returns 503, an empty configured database gives an empty page with null cursor, and no partner/FX details are synthesized.

Page boundaries are not a long-running database snapshot: concurrent operator enable/disable changes can affect later pages. Cursors must be treated as opaque pagination tokens; a future authenticated and signed cursor scheme will be needed if customer-specific filters appear. \`/v1/corridors\` is retained for backwards compatibility with existing read-only clients; new integrations should use the bounded endpoint.

## Opt-in durable Stellar observer

Set \`STEALTHBRIDGE_ENABLE_LEDGER_OBSERVER=true\` **only** on a designated backend worker with a real PostgreSQL database. This starts a 15-second read-only Stellar Testnet ledger-head poller; each response must contain the exact Testnet passphrase, positive ledger sequence, a 64-character hexadecimal ledger hash and a valid close-time. A transactionally monotonic cursor is persisted in \`stellar_ledger_observer\`. A same-sequence hash conflict is rejected and flagged; stale responses cannot rewind the cursor. **No transaction XDR, wallet identity, customer or payment data is stored.**

\`GET /v1/observer\` exposes the last persisted public ledger checkpoint; 404 means no observation has been recorded, and 503 means no database service. This record **may be stale** and is not a settlement receipt, indexer backlog, account balance or regulated payout confirmation.

Operate **one designated observer per environment**; multiple replicas can safely contend on row locks but cause needless RPC load. Production worker leases, chain history backfill, event indexing, failure metrics and replay protection remain future work. The opt-in worker never signs or submits transactions.

## Tenant access-control foundations

The backend now includes a role and separation-of-duties model with a minimal organization/member schema. These are **internal building blocks**, not authentication endpoints or financial permissions. See [organization authorization and missing controls](docs/ORGANIZATION-AUTHORIZATION.md).

### Bounded ledger freshness

The readiness probe verifies not just a valid Testnet RPC network passphrase and database but a ledger **closed within 180 seconds of the current server clock**. An old ledger, invalid timestamp, or timestamp more than 30 seconds ahead of the server clock makes \`GET /ready\` report degraded (503), even if \`getNetwork\` and \`getLatestLedger\` return HTTP success. \`GET /v1/network\` still reports actual metadata without changing or fabricating it.

Keep host clocks synchronized; slow networks can require explicit operator investigation. A fresh ledger is only an infrastructure-readiness condition and **does not enable payments**.

## Future provider webhook inbox

A provider-neutral authenticated-notification verifier and PostgreSQL idempotent inbox are available as internal building blocks. They do not enable provider integrations, public callback endpoints, payouts or settlement transitions. See src/webhook.rs and its unit/integration tests.
