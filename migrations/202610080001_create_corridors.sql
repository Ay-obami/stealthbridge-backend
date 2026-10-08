-- No placeholder accounts, assets, or seeded "demo" corridors.
-- External onboarding/configuration is required before any corridor appears.
CREATE TABLE IF NOT EXISTS corridors (
    id UUID PRIMARY KEY,
    origin_country VARCHAR(2) NOT NULL CHECK (origin_country ~ '^[A-Z]{2}$'),
    destination_country VARCHAR(2) NOT NULL CHECK (destination_country ~ '^[A-Z]{2}$'),
    asset_code TEXT NOT NULL,
    asset_issuer TEXT,
    privacy_rail TEXT NOT NULL CHECK (privacy_rail IN ('confidential-token', 'private-payments')),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT distinct_countries CHECK (origin_country <> destination_country)
);
CREATE INDEX IF NOT EXISTS corridors_enabled_idx
 ON corridors (origin_country, destination_country) WHERE enabled = TRUE;
COMMENT ON TABLE corridors IS 'Operator-configured corridors; none are enabled by default. Not a proof of live liquidity or fiat payout.';
