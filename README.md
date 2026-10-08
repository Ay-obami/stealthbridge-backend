# StealthBridge Backend

**Confidential payments. Without borders.** A Rust-first, multi-tenant corridor orchestration service for StealthBridge Business and Send.

> **Status:** Sprint 0 executable **read-only testnet API scaffold**, not a working payments product. No real funds, quote provider, wallet signer, relayer, or database integration is active.

## Boundaries
| Component | Responsibility | Current |
| --- | --- | --- |
| API | Health, capability/route discovery, request validation | Read-only demo |
| Settlement engine | Strict state transitions, idempotency & recovery | Pure Rust model and unit tests |
| Ledger reconciler | Stellar finality and event ingestion | Design only |
| Privacy adapters | Confidential Tokens / SPP | Feasibility pending |
| FX/payout | Signed quotes, off-ramp callbacks | Design only |
| Persistence | Tenant-scoped transactional journal | Design only |

## Run locally
Install a supported Rust toolchain then run:
\`\`\`sh
cargo test
cargo run
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/v1/capabilities
\`\`\`
\`POST /v1/settlements\` intentionally responds with **501**. Binding defaults to loopback, and there is no external service or credential setup.

## API and contracts
- [OpenAPI v0.1](api/openapi.yaml)
- [Settlement Architecture](docs/ARCHITECTURE.md)
- [Developer Reference Register](docs/DEVELOPER-REFERENCES.md)
- [Windmill Rust skill](.agents/skills/rust-backend/SKILL.md) — vendored upstream, adapted principles only

## Repositories
- [Frontend](https://github.com/stealthbridge-labs/stealthbridge-frontend)
- [Contracts](https://github.com/stealthbridge-labs/stealthbridge-contracts)
- [SDK](https://github.com/stealthbridge-labs/stealthbridge-sdk)

## Documentation policy
Never claim an integration is verified until an independent testnet transaction, privacy disclosure matrix, and reproducible evidence exist. Clear distinction between docs/proposed features, local mocked states and deployed contracts is mandatory.

## License
No open-source license selected yet; we must decide on licensing before Drips onboarding.
