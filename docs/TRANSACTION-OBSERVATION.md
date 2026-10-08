# Public Transaction Observation

GET /v1/transactions/{hash} validates exactly 64 hex bytes and checks the configured RPC identifies the Stellar **Testnet** passphrase before invoking getTransaction.

Its response is a deliberate restricted projection: transaction hash, SUCCESS/FAILED inclusion status, included ledger, RPC latest ledger, close-time Unix string and data source. The service **does not pass through** envelopeXdr, transaction result XDR, contract events or fee metadata.

A NOT_FOUND response maps to 404, including for transactions beyond the configured RPC retention window. A successful on-chain transaction does not establish that a fiat payout, issuer redemption, confidential transfer or bank reconciliation occurred.

Source: https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getTransaction

Security follow-ups: API edge rate limiting, RPC failover, request observability without correlation identifiers in logs, injected/mock-RPC integration tests, source-specific health metrics, off-chain watchlist privacy policy and network latency budgets.
