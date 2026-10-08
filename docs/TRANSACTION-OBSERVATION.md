# Public Transaction Observation

GET /v1/transactions/{hash} validates exactly 64 hex bytes and checks the configured RPC identifies the Stellar **Testnet** passphrase before invoking getTransaction.

Its response is a deliberate restricted projection: transaction hash, SUCCESS/FAILED inclusion status, included ledger, RPC latest ledger, close-time Unix string and data source. The service **does not pass through** envelopeXdr, transaction result XDR, contract events or fee metadata.

A NOT_FOUND response maps to 404, including for transactions beyond the configured RPC retention window. A successful on-chain transaction does not establish that a fiat payout, issuer redemption, confidential transfer or bank reconciliation occurred.

Source: https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getTransaction

Security follow-ups: API edge rate limiting, RPC failover, request observability without correlation identifiers in logs, injected/mock-RPC integration tests, source-specific health metrics, off-chain watchlist privacy policy and network latency budgets.

## RPC hardening (concurrent-reader protection)

The read-only Stellar upstream client has an 8-second HTTP timeout, **16 simultaneous RPC request permits**, and a **2 MiB maximum streamed response**. Responses are not deserialized until their bounded content is collected. Unexpected JSON-RPC versions, mismatched response IDs, error objects, null results and untrusted ledger hashes fail closed as upstream errors. Correctness tests cover malformed response envelopes.

The configured RPC URL must be HTTPS without embedded credentials, fragment or query string. Readiness probes execute network and database checks concurrently, with a two-second DB timeout. These are service-capacity safeguards, **not per-user/IP rate limits**; public deployment still needs an ingress limiter, trusted proxy policy and monitoring.
