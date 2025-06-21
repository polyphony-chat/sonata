CREATE TABLE IF NOT EXISTS actors (
    -- unique actor id
    uaid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_name TEXT UNIQUE NOT NULL,
    deactivated BOOLEAN NOT NULL DEFAULT false,
    joined TIMESTAMP NOT NULL DEFAULT now()
);

COMMENT ON TABLE actors IS 'Actors from this home server.';

CREATE TABLE IF NOT EXISTS name_history (
    uaid UUID NOT NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    time TIMESTAMP NOT NULL DEFAULT now(),
    local_name_old TEXT NOT NULL,
    local_name_new TEXT NOT NULL,
    UNIQUE (time, local_name_old),
    UNIQUE (time, local_name_new)
);
