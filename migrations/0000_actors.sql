CREATE TYPE actor_type AS ENUM ('local', 'foreign');

CREATE TABLE IF NOT EXISTS actors (
    uaid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type ACTOR_TYPE NOT NULL
);

CREATE TABLE IF NOT EXISTS local_actors (
    -- unique actor id
    uaid UUID PRIMARY KEY REFERENCES actors (uaid),
    local_name TEXT UNIQUE NOT NULL,
    deactivated BOOLEAN NOT NULL DEFAULT false,
    joined TIMESTAMP NOT NULL DEFAULT now()
);

COMMENT ON TABLE local_actors IS 'Actors from this home server.';

CREATE TABLE IF NOT EXISTS name_history (
    uaid UUID NOT NULL REFERENCES local_actors (uaid) ON DELETE CASCADE,
    time TIMESTAMP NOT NULL DEFAULT now(),
    local_name_old TEXT NOT NULL,
    local_name_new TEXT NOT NULL,
    UNIQUE (time, local_name_old),
    UNIQUE (time, local_name_new)
);
