# Backend deployment — noncustodial observation service

## Environment
- `HOST=0.0.0.0` only inside a trusted reverse-proxy or PaaS network.
- `PORT` platform-provided listener port.
- `STELLAR_RPC_URL` HTTPS RPC for **Stellar Testnet only**.
- `DATABASE_URL` optional secure PostgreSQL; without it `GET /v1/corridors` responds 503 (no fabricated corridors).
- Do not set signing secrets; service has no signing code.

## Migration and startup
Run `sqlx migrate run` with operator-selected database credentials *as a separate approved deployment step*. The service does not create or mutate external resources by itself. Start using `cargo run --release`.

## Smoke tests
```bash
curl -fsS "$API_URL/health"
curl -fsS "$API_URL/v1/network"
curl -fsS "$API_URL/v1/capabilities"
curl -i "$API_URL/v1/corridors"
curl -i -X POST "$API_URL/v1/settlements"
```
Expected: `/v1/network` returns the actual ledger head and protocol. Settlements always 501 until private transfers work and are verified. A healthy process with broken RPC will still answer `/health` but `/v1/network` returns 502.

## Scale/security
Serve behind TLS, managed DDoS controls and request limits. Add service-level auth and tenant-aware settlement reads *before enabling write endpoints*. Do not publish KYC records or private witnesses to this API. Do not expose a database administrative operation publicly.
