CREATE TABLE IF NOT EXISTS algorithm_identifiers (
    id SERIAL PRIMARY KEY,
    algorithm_identifier TEXT UNIQUE,
    common_name TEXT UNIQUE NULL,
    parameters TEXT NULL
);

CREATE TABLE IF NOT EXISTS public_keys (
    id BIGSERIAL PRIMARY KEY,
    pubkey TEXT UNIQUE,
    algorithm_identifier INT REFERENCES algorithm_identifiers (id) NOT NULL
);

CREATE TABLE IF NOT EXISTS subjects (
    id BIGSERIAL PRIMARY KEY,
    domain_components TEXT [] NOT NULL,
    federation_id TEXT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS issuers (
    id BIGSERIAL PRIMARY KEY,
    domain_components TEXT [] NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS idcsr (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_key_id BIGINT UNIQUE REFERENCES public_keys (id) NOT NULL,
    subject_info_id BIGINT REFERENCES subjects (id) NOT NULL,
    session_id VARCHAR(32) NOT NULL,
    valid_not_before TIMESTAMP NULL,
    valid_not_after TIMESTAMP NULL,
    extensions TEXT NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS idcert (
    idcsr_id UUID PRIMARY KEY REFERENCES idcsr (id),
    issuer_info_id BIGINT REFERENCES issuers (id) NOT NULL,
    valid_not_before TIMESTAMP NOT NULL,
    valid_not_after TIMESTAMP NOT NULL,
    signature_algorithm_identifier INT REFERENCES algorithm_identifiers (id),
    signature TEXT UNIQUE NOT NULL,
    pem_encoded TEXT UNIQUE NOT NULL
);
