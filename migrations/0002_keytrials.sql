CREATE TABLE IF NOT EXISTS keytrials (
    id BIGSERIAL PRIMARY KEY,
    for_id_cert_id BIGINT NOT NULL REFERENCES idcert (idcsr_id) ON DELETE CASCADE,
    expires TIMESTAMP NOT NULL,
    trial VARCHAR(256) UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS keytrials_completed (
    keytrial_id BIGINT PRIMARY KEY REFERENCES keytrials (id) ON DELETE CASCADE,
    signature TEXT UNIQUE NOT NULL
);
