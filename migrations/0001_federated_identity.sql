CREATE TABLE IF NOT EXISTS algorithm_identifiers (
    id SERIAL PRIMARY KEY,
    algorithm_identifier TEXT UNIQUE NOT NULL,
    common_name TEXT UNIQUE NULL,
    parameters TEXT NULL
);

COMMENT ON TABLE algorithm_identifiers IS 'PKCS #10 Algorithm Identifiers for signature and public key algorithms.';

CREATE TABLE IF NOT EXISTS public_keys (
    id BIGSERIAL PRIMARY KEY,
    uaid UUID NOT NULL REFERENCES actors (uaid),
    pubkey TEXT UNIQUE NOT NULL,
    UNIQUE (uaid, pubkey),
    algorithm_identifier INT NOT NULL REFERENCES algorithm_identifiers (id) ON DELETE CASCADE
);

COMMENT ON TABLE public_keys IS 'Public keys of both actors, cached actors and home servers, including this home server.';

CREATE TABLE IF NOT EXISTS subjects (
    uaid UUID PRIMARY KEY REFERENCES actors (uaid),
    domain_components TEXT [] NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

COMMENT ON TABLE subjects IS 'Subjects.';

CREATE TABLE IF NOT EXISTS issuers (
    id BIGSERIAL PRIMARY KEY,
    domain_components TEXT [] NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

COMMENT ON TABLE issuers IS 'Issuers. Deduplicates issuer entries. Especially helpful, if the domain of this home server changes.';

CREATE TABLE IF NOT EXISTS idcsr (
    id BIGSERIAL PRIMARY KEY,
    serial_number NUMERIC(49, 0) UNIQUE NOT NULL,
    uaid UUID NOT NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    actor_public_key_id BIGINT UNIQUE NOT NULL REFERENCES public_keys (id) ON DELETE CASCADE,
    actor_signature TEXT UNIQUE NOT NULL,
    session_id VARCHAR(32) NOT NULL,
    valid_not_before TIMESTAMP NULL,
    valid_not_after TIMESTAMP NULL,
    extensions TEXT NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

COMMENT ON TABLE idcsr IS 'ID-CSRs.';
COMMENT ON COLUMN idcsr.serial_number IS 'To be generated via a CSPRNG. Serial numbers must not be used for cryptographic purposes';

CREATE TABLE IF NOT EXISTS invalidated_certs (
    id BIGSERIAL PRIMARY KEY,
    serial_number NUMERIC(49, 0) UNIQUE NOT NULL REFERENCES idcsr (serial_number) ON DELETE CASCADE,
    cert_id BIGINT UNIQUE NOT NULL REFERENCES idcsr (id) ON DELETE CASCADE,
    invalidated_at TIMESTAMP NOT NULL
);

COMMENT ON TABLE invalidated_certs IS 'Stores information about all invalidated certificates that were created by this home server';

ALTER TABLE idcsr ADD COLUMN invalidation_info BIGINT NULL;
ALTER TABLE idcsr ADD UNIQUE (session_id, invalidation_info, valid_not_before, valid_not_after);

CREATE TABLE IF NOT EXISTS idcert (
    idcsr_id BIGINT PRIMARY KEY REFERENCES idcsr (id),
    issuer_info_id BIGINT NOT NULL REFERENCES issuers (id) ON DELETE CASCADE,
    valid_not_before TIMESTAMP NOT NULL,
    valid_not_after TIMESTAMP NOT NULL,
    home_server_public_key_id BIGINT NOT NULL REFERENCES public_keys (id) ON DELETE CASCADE,
    home_server_signature TEXT UNIQUE NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

COMMENT ON TABLE idcert IS 'ID-Certs.';

CREATE TABLE IF NOT EXISTS idcert_cached (
    idcert_id BIGINT PRIMARY KEY REFERENCES idcert (idcsr_id),
    cache_not_valid_before TIMESTAMP NOT NULL,
    cache_not_valid_after TIMESTAMP NOT NULL,
    cache_signature TEXT UNIQUE NOT NULL
);

COMMENT ON TABLE idcert_cached IS 'ID-Certs issued by this home server with additional cache information.';

CREATE TABLE IF NOT EXISTS foreign_cached_idcerts (
    federation_id TEXT NOT NULL,
    session_id VARCHAR(32) NOT NULL,
    PRIMARY KEY (federation_id, session_id),
    invalidated_at TIMESTAMP NULL,
    cache_not_valid_before TIMESTAMP NOT NULL,
    cache_not_valid_after TIMESTAMP NOT NULL,
    cache_signature TEXT UNIQUE NOT NULL,
    home_server_signature_public_key BIGINT NULL REFERENCES public_keys (id) ON DELETE CASCADE,
    idcert_pem TEXT NULL
);

COMMENT ON TABLE foreign_cached_idcerts IS 'ID-Certs that have been cached, but issued by other home servers.';
