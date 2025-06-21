CREATE TABLE IF NOT EXISTS resources (
    id BIGSERIAL PRIMARY KEY,
    link_to TEXT NULL,
    bytes BYTEA NULL,
    CONSTRAINT link_or_bytes CHECK ((bytes IS NOT NULL AND link_to IS NULL) OR (bytes IS NULL AND link_to IS NOT NULL))
);

COMMENT ON TABLE resources IS 'RawR resources. Either has the server act as a cdn by directly storing and distributing the resource, or acts as a proxy, returning the link to the actual file.';

CREATE TABLE IF NOT EXISTS resource_access_properties (
    id BIGINT PRIMARY KEY REFERENCES resources (id) ON DELETE CASCADE,
    private BOOLEAN NOT NULL,
    public BOOLEAN NOT NULL,
    CONSTRAINT not_public_and_private CHECK (
        (private::INT + public::INT) <= 1
    ),
    allowlist TEXT [] NULL,
    denylist TEXT [] NULL
);

CREATE TABLE IF NOT EXISTS resource_information (
    id BIGINT UNIQUE REFERENCES resource_access_properties (id) ON DELETE CASCADE,
    size BIGINT NOT NULL,
    resource_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    owner_uaid UUID NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL,
    b3sum VARCHAR(64) NULL,
    PRIMARY KEY (owner_uaid, resource_id)
);
