CREATE TABLE IF NOT EXISTS api_keys (
    id SERIAL PRIMARY KEY,
    token VARCHAR(255) UNIQUE NOT NULL,
    CONSTRAINT token_length CHECK (length(token) >= 24 AND length(token) <= 255)
);

CREATE TABLE IF NOT EXISTS user_tokens (
    token_hash VARCHAR(255) PRIMARY KEY,
    cert_id BIGINT NOT NULL REFERENCES idcsr (id),
    uaid UUID NOT NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    valid_not_after TIMESTAMP NULL
);
