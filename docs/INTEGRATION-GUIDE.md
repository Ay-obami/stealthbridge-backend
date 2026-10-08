# StealthBridge Backend — Integration and Operating Guide

## 1. Purpose and trust boundaries

This Rust/Axum service provides reliable public Testnet observation and an **internal, tenant-scoped settlement journal**. It does **not** hold wallet keys, custody assets, issue tokens, quote fiat FX, or execute a payout. Never infer financial readiness from a successful HTTP response or passing CI.

```text
Frontend / TypeScript SDK
      │ HTTPS read-only metadata
      ▼
Rust API (network passphrase enforced)
      ├── Stellar RPC: getNetwork, getLatestLedger, getTransaction
      │    └─ Projection only: ledger/hash/status, no raw XDR or contract events
      └── PostgreSQL: enabled corridors and internal settlement state journal
           ├─ tenant-scoped idempotent intents
           └─ append-only transition log (not a double-entry ledger)
```

## 2. Current API surface

| Route | Input | Result | Operational caveat |
|---|---|---|---|
| `GET /health` | None | 200 process liveness | Not chain readiness |
| `GET /ready` | None | 200 ready / 503 degraded | Requires both RPC and DB; payment flags remain disabled |
| `GET /v1/network` | None | Live protocol/ledger and passphrase | 502 on bad/unavailable RPC |
| `GET /v1/capabilities` | None | Explicit boolean capability flags | Privacy and payment flags remain false |
| `GET /v1/corridors` | None | Enabled database records, maybe empty | 503 without PostgreSQL |
| `GET /v1/corridors/{id}` | UUID | Single enabled operator record | 400 invalid, 404 missing, 503 database unavailable |
| `GET /v1/transactions/{hash}` | 64 hex | Public execution status and ledger | 404 includes old/nonretained tx; no fiat payout claim |
| `POST /v1/settlements` | — | 501 | No fund submission allowed |

All public reads return live observation or persisted configuration, **never synthetic rates/partners/success messages**. See [OpenAPI](../api/openapi.yaml) for response schemas.

## 3. Local startup and migrations

Configure `STELLAR_RPC_URL=https://soroban-testnet.stellar.org` and optionally `DATABASE_URL` with an operator-managed PostgreSQL instance. Run migrations using an approved SQLx migration process; startup itself does not mutate the database.

```sh
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run
curl -i http://127.0.0.1:8080/health
curl -i http://127.0.0.1:8080/ready
curl -i http://127.0.0.1:8080/v1/network
```

A database is intentionally not seeded with example corridors. An unconfigured instance may serve `/v1/network` while `/ready` and `/v1/corridors` correctly report degraded/unavailable.

## 4. Financial domain foundations

`src/amount.rs` provides exact checked i128 minor-unit arithmetic tied to asset identity and decimals. Identity and scale must be validated externally against an actual issuer or on-chain token. No floating-point amounts should enter durable settlement logic.

`src/settlement.rs` holds the pure transition rules. `src/store.rs` holds organization-scoped idempotent registration with payload digests, row locking, revision checks and an atomic append-only history. These functions are deliberately not exposed as public HTTP writes. A future caller must verify tenant identity, wallet authorization, quote signatures, proof validity, issuer permissions, compliance eligibility, and external provider obligations first.

## 5. Privacy and operational safety

- Never return raw transaction XDR or contract events through the public observation API.
- Do not log secret inputs, payment witnesses or personal financial identifiers.
- Deploy behind TLS, request rate limits, managed secrets and database credentials with minimum privileges.
- The current readiness endpoint is *dependency status*, not a settlement certification.
- CI's PostgreSQL service uses synthetic isolated test data; these are **not production corridor registrations**.

## 6. Release sequence

First verify API/SDK schema agreement and remote endpoint data, then authenticated organization identity, then proof/asset compatibility, then signed quotes and structured settlement attempts, then provider-specific payout reconciliation. Review security, licensing and local compliance requirements before anything fund-moving. The [backend roadmap](../ROADMAP.md) describes full exit criteria.

## RPC hardening (concurrent-reader protection)

The read-only Stellar upstream client has an 8-second HTTP timeout, **16 simultaneous RPC request permits**, and a **2 MiB maximum streamed response**. Responses are not deserialized until their bounded content is collected. Unexpected JSON-RPC versions, mismatched response IDs, error objects, null results and untrusted ledger hashes fail closed as upstream errors. Correctness tests cover malformed response envelopes.

The configured RPC URL must be HTTPS without embedded credentials, fragment or query string. Readiness probes execute network and database checks concurrently, with a two-second DB timeout. These are service-capacity safeguards, **not per-user/IP rate limits**; public deployment still needs an ingress limiter, trusted proxy policy and monitoring.

## Bounded corridor keyset discovery

\`GET /v1/corridors/page?limit=25&after=<UUID>\` returns \`{items, next_cursor}\` from **real enabled operator records** in PostgreSQL. Limits range 1–100, default 25; malformed UUID or out-of-range count returns 400. The query fetches one extra row to determine whether a cursor should be returned, so the service never reads the full table to produce a page. Stable UUID ordering avoids OFFSET scans at larger tables. A missing database returns 503, an empty configured database gives an empty page with null cursor, and no partner/FX details are synthesized.

Page boundaries are not a long-running database snapshot: concurrent operator enable/disable changes can affect later pages. Cursors must be treated as opaque pagination tokens; a future authenticated and signed cursor scheme will be needed if customer-specific filters appear. \`/v1/corridors\` is retained for backwards compatibility with existing read-only clients; new integrations should use the bounded endpoint.
