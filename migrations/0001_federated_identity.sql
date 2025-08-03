CREATE TABLE IF NOT EXISTS algorithm_identifiers (
    id serial PRIMARY KEY,
    algorithm_identifier text UNIQUE NOT NULL,
    common_name text UNIQUE NULL,
    parameters_der_encoded smallint [] NULL
);

COMMENT ON TABLE algorithm_identifiers IS 'PKCS #10 Algorithm Identifiers for signature and public key algorithms.';

CREATE TABLE IF NOT EXISTS public_keys (
    id bigserial PRIMARY KEY,
    uaid uuid NULL REFERENCES local_actors (uaid),
    pubkey text UNIQUE NOT NULL,
    UNIQUE (uaid, pubkey),
    algorithm_identifier int NOT NULL REFERENCES algorithm_identifiers (id) ON DELETE CASCADE
);

COMMENT ON TABLE public_keys IS 'Public keys of both actors, cached actors and home servers, including this home server.';

CREATE TABLE IF NOT EXISTS idcsr (
    id bigserial PRIMARY KEY,
    serial_number numeric(49, 0) UNIQUE NOT NULL,
    uaid uuid NULL REFERENCES local_actors (uaid) ON DELETE CASCADE,
    subject_public_key_id bigint UNIQUE NOT NULL REFERENCES public_keys (id) ON DELETE CASCADE,
    subject_signature text UNIQUE NOT NULL,
    session_id varchar(32) NOT NULL,
    valid_not_before timestamp NULL,
    valid_not_after timestamp NULL,
    extensions text NOT NULL,
    pem_encoded text UNIQUE NOT NULL
);

COMMENT ON TABLE idcsr IS 'ID-CSRs.';
COMMENT ON COLUMN idcsr.serial_number IS 'To be generated via a CSPRNG. Serial numbers must not be used for cryptographic purposes';

CREATE TABLE IF NOT EXISTS issuers (
    id bigserial PRIMARY KEY,
    domain_components text [] UNIQUE NOT NULL
);

COMMENT ON TABLE issuers IS 'Issuers. Deduplicates issuer entries. Especially helpful, if the domain of this home server changes.';

CREATE TABLE IF NOT EXISTS invalidated_certs (
    id bigserial PRIMARY KEY,
    serial_number numeric(49, 0) UNIQUE NOT NULL REFERENCES idcsr (serial_number) ON DELETE CASCADE,
    cert_id bigint UNIQUE NOT NULL REFERENCES idcsr (id) ON DELETE CASCADE,
    invalidated_at timestamp NOT NULL
);

COMMENT ON TABLE invalidated_certs IS 'Stores information about all invalidated certificates that were created by this home server';

ALTER TABLE idcsr ADD COLUMN invalidation_info bigint NULL;
ALTER TABLE idcsr ADD UNIQUE (session_id, invalidation_info, valid_not_before, valid_not_after);

CREATE TABLE IF NOT EXISTS idcert (
    idcsr_id bigint PRIMARY KEY REFERENCES idcsr (id),
    issuer_info_id bigint NOT NULL REFERENCES issuers (id) ON DELETE CASCADE,
    valid_not_before timestamp NOT NULL,
    valid_not_after timestamp NOT NULL,
    home_server_public_key_id bigint NOT NULL REFERENCES public_keys (id) ON DELETE CASCADE,
    home_server_signature text UNIQUE NOT NULL,
    pem_encoded text UNIQUE NOT NULL
);

COMMENT ON TABLE idcert IS 'ID-Certs.';

CREATE TABLE IF NOT EXISTS idcert_cached (
    idcert_id bigint PRIMARY KEY REFERENCES idcert (idcsr_id),
    cache_not_valid_before timestamp NOT NULL,
    cache_not_valid_after timestamp NOT NULL,
    cache_signature text UNIQUE NOT NULL
);

COMMENT ON TABLE idcert_cached IS 'ID-Certs issued by this home server with additional cache information.';

CREATE TABLE IF NOT EXISTS foreign_cached_idcerts (
    federation_id text NOT NULL,
    session_id varchar(32) NOT NULL,
    PRIMARY KEY (federation_id, session_id),
    invalidated_at timestamp NULL,
    cache_not_valid_before timestamp NOT NULL,
    cache_not_valid_after timestamp NOT NULL,
    cache_signature text UNIQUE NOT NULL,
    home_server_signature_public_key bigint NULL REFERENCES public_keys (id) ON DELETE CASCADE,
    idcert_pem text NULL
);

COMMENT ON TABLE foreign_cached_idcerts IS 'ID-Certs that have been cached, but issued by other home servers.';
