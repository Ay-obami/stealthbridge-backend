-- Infrastructure schema extension for settlement financial persistence and outbox.
-- No amount is represented as floating point; minor units are stored as 64-bit integers.

ALTER TABLE settlement_intents
    ADD COLUMN IF NOT EXISTS amount_minor_units BIGINT CHECK (amount_minor_units IS NULL OR amount_minor_units >= 0),
    ADD COLUMN IF NOT EXISTS asset_code VARCHAR(32);

CREATE TABLE IF NOT EXISTS settlement_outbox (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    organization_id UUID NOT NULL,
    settlement_id UUID NOT NULL,
    event_type VARCHAR(64) NOT NULL CHECK (length(event_type) > 0),
    payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ,
    FOREIGN KEY (organization_id, settlement_id) REFERENCES settlement_intents (organization_id, id),
    CONSTRAINT outbox_processing_time CHECK (processed_at IS NULL OR processed_at >= created_at)
);

CREATE INDEX IF NOT EXISTS settlement_outbox_pending_idx
    ON settlement_outbox (organization_id, created_at) WHERE processed = FALSE;

COMMENT ON TABLE settlement_outbox IS
  'Transactional outbox for asynchronous settlement workflow events; emitted atomically with state changes.';
