-- Signed, tenant-scoped notification inbox. No raw bodies, credentials or KYC.
CREATE TABLE IF NOT EXISTS provider_event_inbox(
 organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 provider VARCHAR(128) NOT NULL CHECK (provider ~ '^[A-Za-z0-9:_-]{8,128}$'),
 event_id VARCHAR(128) NOT NULL CHECK (event_id ~ '^[A-Za-z0-9:_-]{8,128}$'),
 event_type VARCHAR(128) NOT NULL CHECK (event_type ~ '^[A-Za-z0-9:._-]{3,128}$'),
 payload_sha256 CHAR(64) NOT NULL CHECK (payload_sha256 ~ '^[a-f0-9]{64}$'),
 received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 PRIMARY KEY(organization_id,provider,event_id)
);
COMMENT ON TABLE provider_event_inbox IS
 'Authenticated notification metadata only; no financial payout or settlement changes';
