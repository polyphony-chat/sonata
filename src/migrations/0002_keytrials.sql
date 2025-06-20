CREATE TABLE IF NOT EXISTS keytrials (
    id BIGSERIAL PRIMARY KEY,
    for_id_cert UUID REFERENCES idcsr (id) NOT NULL,
    expires TIMESTAMP NOT NULL,
    trial VARCHAR(256) NOT NULL
);

CREATE TABLE IF NOT EXISTS keytrials_completed (
    keytrial_id BIGINT PRIMARY KEY REFERENCES keytrials (id),
    signature TEXT UNIQUE NOT NULL
);
