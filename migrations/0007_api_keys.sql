CREATE TABLE IF NOT EXISTS api_keys (
    id SERIAL PRIMARY KEY,
    token VARCHAR(255) UNIQUE NOT NULL,
    CONSTRAINT token_length CHECK (length(token) >= 24 AND length(token) <= 255)
);
