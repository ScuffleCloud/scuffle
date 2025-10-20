CREATE TABLE "mfa_totp_credentials" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "name" VARCHAR(255),
    "url" VARCHAR(1024) NOT NULL,
    "last_used_at" TIMESTAMPTZ
);

ALTER TABLE "mfa_totp_credentials"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE UNIQUE INDEX ON "mfa_totp_credentials"("user_id");


CREATE TABLE "mfa_webauthn_credentials" (
    -- A sha256 of the credential ID (https://docs.rs/webauthn-rs/latest/webauthn_rs/prelude/struct.Passkey.html#method.cred_id)
    "id" sha256 PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "name" VARCHAR(255),
    -- contains the Passkey (https://docs.rs/webauthn-rs/latest/webauthn_rs/prelude/struct.Passkey.html)
    "credential" JSONB NOT NULL,
    "counter" BIGINT NOT NULL DEFAULT 0 CHECK (counter >= 0),
    "last_used_at" TIMESTAMPTZ
);

ALTER TABLE "mfa_webauthn_credentials"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "mfa_webauthn_credentials"("user_id");


CREATE TABLE "mfa_recovery_codes" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID NOT NULL,
    -- ARGON2 hashed 12 character alphanumeric code
    "code_hash" VARCHAR(255) NOT NULL
);

ALTER TABLE "mfa_recovery_codes"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "mfa_recovery_codes"("user_id");
