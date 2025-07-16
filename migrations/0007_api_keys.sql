CREATE TABLE IF NOT EXISTS api_keys (
    id SERIAL PRIMARY KEY,
    token VARCHAR(255) UNIQUE NOT NULL,
    CONSTRAINT token_length CHECK (length(token) >= 24 AND length(token) <= 255)
);

CREATE TABLE IF NOT EXISTS user_tokens (
    token_hash VARCHAR(255) PRIMARY KEY,
    uaid UUID NOT NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    valid_not_after TIMESTAMP NULL,
    cert_id BIGINT NULL REFERENCES idcert (idcsr_id),
    UNIQUE NULLS NOT DISTINCT (uaid, cert_id)
);

COMMENT ON TABLE user_tokens IS 'User access token hashes. Cleans up expired tokens on each insert operation of this table. Use view filtering to exclude expired tokens on queries.';

-- We could use the postgres cron extension, but if it is possible and feasible, I'd like to stick
-- with a "vanilla" postgres setup.
CREATE OR REPLACE FUNCTION cleanup_expired_tokens()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM user_tokens
    WHERE valid_not_after IS NOT NULL
      AND valid_not_after < NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cleanup_expired_tokens_trigger
AFTER INSERT ON user_tokens
EXECUTE FUNCTION cleanup_expired_tokens();
