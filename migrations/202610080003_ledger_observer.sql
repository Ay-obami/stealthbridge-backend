-- Opt-in, Testnet-only checkpoint. Not a custody ledger or proof of a payout.
CREATE TABLE IF NOT EXISTS stellar_ledger_observer (
    network TEXT NOT NULL CHECK (network = 'testnet'),
    source TEXT NOT NULL CHECK (source = 'stellar-rpc'),
    ledger_sequence BIGINT NOT NULL CHECK (ledger_sequence > 0),
    ledger_hash CHAR(64) NOT NULL CHECK (ledger_hash ~ '^[0-9a-f]{64}$'),
    close_time_unix VARCHAR(20) NOT NULL CHECK (close_time_unix ~ '^[0-9]+$'),
    observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (network, source)
);
COMMENT ON TABLE stellar_ledger_observer IS
  'Last verified Testnet ledger head observed from RPC; no wallet, transfer, or payout data';
