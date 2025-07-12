CREATE TYPE service AS (
    service_id VARCHAR (64),
    service_url TEXT,
    is_primary BOOLEAN
);

CREATE TABLE IF NOT EXISTS actor_service_mapping (
    id BIGSERIAL PRIMARY KEY,
    uaid UUID UNIQUE REFERENCES local_actors (uaid) ON DELETE CASCADE,
    services SERVICE [] NULL
);
