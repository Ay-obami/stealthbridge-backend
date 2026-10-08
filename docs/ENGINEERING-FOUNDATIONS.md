# Backend engineering foundations

## Exact financial values
Rust `AssetAmount` stores a strictly scoped asset identity and **i128 minor units**. Parsing unsigned decimal strings rejects ambiguous locale notation, exponent notation, negative transfers, decimal scale overflow, mismatched token identities and checked overflow. This is an internal domain helper, not a quote provider or source of asset metadata. No arbitrary currency precision is assumed.

## Readiness semantics
Process liveness at `/health` can succeed when dependencies are unavailable. `/ready` probes the actual Testnet RPC and PostgreSQL if configured, returning 200 only when both are reachable. A missing `DATABASE_URL` is reported as `database=not-configured`; an unsuccessful probe of a configured database is `database=unavailable`. Both are degraded 503 responses. A ready read-only API still cannot submit payments; the readiness projection always says payments are disabled.

Set `STEALTHBRIDGE_METRICS_TOKEN` only in an internal deployment to enable `/internal/metrics`. Scrape it with `Authorization: Bearer <token>` over the service's private network or an authenticated gateway. Leave it unset to disable the endpoint. Metrics use fixed names and aggregate counts; they do not include wallet, transaction, tenant, or customer identifiers. RPC age is omitted until a valid observation has been recorded.

## Next implementation gates
Tenant authentication, signed-and-expiring quotes, verified account/asset metadata, approval policy, privacy proof validation, regulatory partner onboarding, unique provider callbacks, reconciliation and incident response. Do not enable settlement routes until these are independently tested and authorized.

## Testing
`cargo test --all-targets` covers exact-value domain rules and PostgreSQL journal integration in CI. Source-level builds do not substitute for live deployment verification, threat-model review or an external security audit.
