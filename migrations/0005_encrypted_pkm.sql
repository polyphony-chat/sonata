CREATE TABLE IF NOT EXISTS encrypted_pkm (
    id BIGSERIAL PRIMARY KEY,
    uaid UUID REFERENCES local_actors (uaid) ON DELETE CASCADE,
    key_data TEXT NOT NULL,
    encryption_algorithms VARCHAR(255) [5] NOT NULL,
    UNIQUE (uaid, key_data)
);

COMMENT ON TABLE encrypted_pkm IS 'Encrypted private key material, uploaded by local_actors';
