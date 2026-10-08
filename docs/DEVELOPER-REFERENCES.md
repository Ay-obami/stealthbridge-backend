# Engineering reference register

## Primary
- https://skills.stellar.org/
- https://github.com/stellar/stellar-dev-skill/tree/main/skills/smart-contracts
- https://github.com/stellar/stellar-dev-skill/tree/main/skills/dapp
- https://github.com/stellar/stellar-dev-skill/tree/main/skills/zk-proofs
- https://developers.stellar.org/docs
- https://stellar.org/blog/developers/practical-confidential-stablecoins-an-issuer-controlled-architecture
- https://stellar.org/blog/developers/developer-preview-stellar-private-payments

## External pattern sources, not drop-in dependencies
- https://github.com/PugarHuda/tukar — product scope, testnet evidence, security documentation, explicit prototype limitations
- https://github.com/windmill-labs/windmill/tree/main/.agents/skills/rust-backend — upstream Rust patterns

## Windmill skill installation and adaptation
The upstream rust-backend SKILL.md was installed under \`.agents/skills/rust-backend/SKILL.md\` and read back. It includes Windmill-specific types (\`windmill_common\`), paths and telemetry; **do not import those into StealthBridge**. Transfer its applicable principles: explicit queries, parameterized SQL, transactions, no panics, bounded queues, CPU-bound work off runtime and structured Axum handlers.
The official CLI method for a developer environment is:
\`\`\`bash
npx skills add https://github.com/windmill-labs/windmill --skill rust-backend
\`\`\`
That CLI was not run remotely; the verbatim SKILL.md was vendored instead after source review.

## Pending integrations
OpenZeppelin Relayer: not configured, credentials withheld; DeFindex SDK: out of scope until real liquidity product requirement; CCTP: future adapter, not part of first Stellar-only corridor. All require separate security/design decisions.
