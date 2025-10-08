CREATE TYPE "device_algorithm" AS ENUM (
    'RSA_OAEP_SHA256' -- RSA with OAEP padding and SHA-256 hashing
);

-- SHA-256 of the device public key
CREATE DOMAIN "sha256" AS BYTEA CHECK (octet_length(VALUE) = 32);

-- Devices are a way to identify a device.
CREATE TABLE "devices" (
    -- A sha256 of the device public key
    "fingerprint" sha256 PRIMARY KEY,
    -- The algorithm used to generate the fingerprint
    "algorithm" device_algorithm NOT NULL,
    -- The device public key
    "public_key_data" BYTEA NOT NULL,
    -- The last time the device was active
    "last_active_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- The time the device was created
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UserSessions are a way to identify a user on a device.
CREATE TABLE "user_sessions" (
    -- user this session is for
    "user_id" UUID NOT NULL,
    -- device this session is for
    "device_fingerprint" sha256 NOT NULL,
    -- the token
    "token" VARCHAR(32) NOT NULL,
    -- When the session expires
    "token_expires_at" TIMESTAMPTZ NOT NULL,
    -- When the session can no longer be refreshed
    "refresh_expires_at" TIMESTAMPTZ NOT NULL,
    -- The last time the session was last logged in / refreshed
    "last_login_at" TIMESTAMPTZ NOT NULL,
    -- When this session was first created
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY("user_id", "device_fingerprint")
);

CREATE INDEX ON "user_sessions"("device_fingerprint");
CREATE UNIQUE INDEX ON "user_sessions"("user_id", "device_fingerprint");

ALTER TABLE "user_sessions"
ADD FOREIGN KEY("device_fingerprint") REFERENCES "devices"("fingerprint")
ON DELETE CASCADE;

ALTER TABLE "user_sessions"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

-- Magic links are a login method for users.
CREATE TABLE "magic_link_requests" (
    "id" UUID PRIMARY KEY,
    -- the user this magic link is for
    "user_id" UUID NOT NULL,
    -- email address this was sent to
    "email" VARCHAR(255) NOT NULL,
    -- 5 character code
    "code" VARCHAR(5) NOT NULL,
    -- when the magic link expires
    "expires_at" TIMESTAMPTZ NOT NULL
);

CREATE INDEX ON "magic_link_requests"("user_id");

ALTER TABLE "magic_link_requests"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;
