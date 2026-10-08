-- Infrastructure schema only. There is NO publicly accessible intent-creation
-- endpoint, custody handler, fiat execution or payment transaction in this phase.
CREATE TABLE IF NOT EXISTS settlement_intents (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL,
    corridor_id UUID NOT NULL REFERENCES corridors(id),
    idempotency_key VARCHAR(128) NOT NULL CHECK (length(idempotency_key) BETWEEN 16 AND 128),
    request_sha256 CHAR(64) NOT NULL CHECK (request_sha256 ~ '^[0-9a-f]{64}$'),
    state VARCHAR(32) NOT NULL DEFAULT 'draft' CHECK (state IN (
        'draft','quoted','authorized','submitted','chain_finalized','payout_pending',
        'payout_completed','expired','rejected','chain_failed','payout_failed',
        'refund_pending','refunded','manual_review'
    )),
    version BIGINT NOT NULL DEFAULT 0 CHECK (version >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, idempotency_key),
    UNIQUE (organization_id, id)
);
CREATE INDEX IF NOT EXISTS settlement_intents_by_org_state
    ON settlement_intents (organization_id, state, created_at DESC);

CREATE TABLE IF NOT EXISTS settlement_transition_log (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    organization_id UUID NOT NULL,
    settlement_id UUID NOT NULL,
    previous_state VARCHAR(32),
    next_state VARCHAR(32) NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (organization_id, settlement_id) REFERENCES settlement_intents (organization_id, id),
    UNIQUE (organization_id, settlement_id, version)
);
COMMENT ON TABLE settlement_transition_log IS 'Append-only intent state history; application DB role must not have update or delete privileges.';
