CREATE TABLE IF NOT EXISTS invite_links (
    id BIGSERIAL PRIMARY KEY,
    invite_link_owner UUID NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    usages_current INT NOT NULL DEFAULT 0,
    usages_maximum INT NOT NULL DEFAULT 1,
    invite VARCHAR(16) NOT NULL,
    invalid BOOLEAN NOT NULL,
    UNIQUE (invalid, invite)
);

CREATE TABLE IF NOT EXISTS invitations (
    invite_id BIGINT NOT NULL REFERENCES invite_links (id),
    uaid_inviter UUID NOT NULL REFERENCES actors (uaid) ON DELETE CASCADE,
    uaid_invited UUID NOT NULL REFERENCES actors (uaid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reputation (
    uaid UUID REFERENCES actors (uaid) ON DELETE CASCADE PRIMARY KEY,
    score INT NOT NULL DEFAULT 0
);

ALTER TABLE actors ADD COLUMN invites_available INT NOT NULL DEFAULT 0;
