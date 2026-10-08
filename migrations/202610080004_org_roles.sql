-- Organization and role data foundation only. This schema does NOT
-- authenticate wallet signatures, grant custody or activate settlement APIs.
-- No records or supposed customers are seeded.
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY,
    display_name VARCHAR(120) NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 120),
    status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','active','suspended')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS organization_members (
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    subject VARCHAR(256) NOT NULL CHECK (length(subject) BETWEEN 8 AND 256),
    role VARCHAR(16) NOT NULL CHECK (role IN ('owner','admin','operator','approver','viewer')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    PRIMARY KEY (organization_id,subject),
    CONSTRAINT member_revocation_time CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);
CREATE INDEX IF NOT EXISTS organization_members_active
    ON organization_members (organization_id, role) WHERE revoked_at IS NULL;
COMMENT ON TABLE organizations IS
 'Tenant metadata; status is not a legal/compliance verification or payment approval';
COMMENT ON TABLE organization_members IS
 'Role assignments only; subjects must be authenticated and tenant-bound by a separately reviewed implementation';
