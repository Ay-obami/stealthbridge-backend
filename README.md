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
