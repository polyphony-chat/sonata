CREATE TABLE IF NOT EXISTS keytrials (
    id BIGSERIAL PRIMARY KEY,
    for_id_cert_serial_number UUID NOT NULL REFERENCES idcsr (serial_number) ON DELETE CASCADE,
    expires TIMESTAMP NOT NULL,
    trial VARCHAR(256) NOT NULL
);

CREATE TABLE IF NOT EXISTS keytrials_completed (
    keytrial_id BIGINT PRIMARY KEY REFERENCES keytrials (id) ON DELETE CASCADE,
    signature TEXT UNIQUE NOT NULL
);
