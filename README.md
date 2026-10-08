# StealthBridge Backend

Real-time, **read-only** Stellar Testnet observation API plus an unseeded, operator-managed corridor catalog. Rust, Axum, PostgreSQL, Stellar RPC.

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
- [Service architecture](docs/ARCHITECTURE.md)

*Status:* no confidential transfers, stablecoin issuance, FX rates, wallet signing, fiat payouts or proof generation wired yet. Do not process real money. Network is restricted to Testnet passphrase.

## Other repositories
[Frontend](https://github.com/stealthbridge-labs/stealthbridge-frontend) · [Contracts](https://github.com/stealthbridge-labs/stealthbridge-contracts) · [SDK](https://github.com/stealthbridge-labs/stealthbridge-sdk)
