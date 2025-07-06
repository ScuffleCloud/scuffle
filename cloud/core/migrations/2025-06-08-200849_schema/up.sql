-- User Management

-- Real users
CREATE TABLE "users" (
    "id" UUID PRIMARY KEY,
    "preferred_name" VARCHAR(255),
    "first_name" VARCHAR(255),
    "last_name" VARCHAR(255),
    "password_hash" VARCHAR(255), -- Nullable for users who register via external providers
    "primary_email" VARCHAR(255)
);

-- User emails
-- There can be multiple emails per user, but only one can be primary.
CREATE TABLE "user_emails" (
    "email" VARCHAR(255) PRIMARY KEY, -- should be normalized (to ascii lowercase?)
    "user_id" UUID NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE "users"
ADD FOREIGN KEY("primary_email") REFERENCES "user_emails"("email")
DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX ON "users"("primary_email");

ALTER TABLE "user_emails"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE
DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX ON "user_emails"("user_id");

-- User connections to Google.
--
-- https://developers.google.com/people/api/rest/v1/people/get
-- https://developers.google.com/people/api/rest/v1/people#Person
-- Person.memberships.metadata.source
-- Also: Person.metadata.sources.profileMetadata.userTypes == GOOGLE_APPS_USER
CREATE TABLE "user_google_accounts" (
    "sub" VARCHAR(255) PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "access_token" VARCHAR(255) NOT NULL,
    "access_token_expires_at" TIMESTAMPTZ NOT NULL,
    -- workspace and role within the workspace...
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE "user_google_accounts"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "user_google_accounts"("user_id");

-- This table contains one-time code user session requests which precede a full user session.
-- This is usually used for the CLI application.
CREATE TABLE "user_session_requests" (
    "id" UUID PRIMARY KEY,
    "device_name" VARCHAR(255) NOT NULL, -- Name of the device which is requesting the session
    "device_ip" INET NOT NULL,
    "code" VARCHAR(6) NOT NULL UNIQUE, -- 6 digit code
    "approved_by" UUID, -- Only set if this request was approved by a user.
    "expires_at" TIMESTAMPTZ NOT NULL
);

ALTER TABLE "user_session_requests"
ADD FOREIGN KEY("approved_by") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "user_session_requests"("code");

-- Used for magic link login.
CREATE TABLE "magic_link_user_session_requests" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "code" BYTEA NOT NULL UNIQUE,
    "expires_at" TIMESTAMPTZ NOT NULL
);

ALTER TABLE "magic_link_user_session_requests"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "magic_link_user_session_requests"("code");

CREATE TYPE "device_algorithm" AS ENUM (
    'RSA_OAEP_SHA256' -- RSA with OAEP padding and SHA-256 hashing
);

-- A combination of device and user.
-- "last_used_at" and "last_ip" is updated every time this session is reactivated.
CREATE TABLE "user_sessions" (
    "user_id" UUID NOT NULL,
    "device_fingerprint" BYTEA NOT NULL UNIQUE,
    "device_algorithm" DEVICE_ALGORITHM NOT NULL,
    "device_pk_data" BYTEA NOT NULL, -- PKCS#8 DER SPKI
    "last_used_at" TIMESTAMPTZ NOT NULL,
    "last_ip" INET NOT NULL,
    "token_id" UUID UNIQUE,
    "token" BYTEA,
    "token_expires_at" TIMESTAMPTZ,
    "expires_at" TIMESTAMPTZ NOT NULL,
    "mfa_pending" BOOLEAN NOT NULL,
    PRIMARY KEY("user_id", "device_fingerprint")
);

ALTER TABLE "user_sessions"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "user_sessions"("device_fingerprint")
WHERE "token_id" IS NOT NULL;

CREATE INDEX ON "user_sessions"("token_id");

--- MFA (Multi-Factor Authentication)

CREATE TABLE "mfa_totps" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "secret" BYTEA NOT NULL
);

ALTER TABLE "mfa_totps"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "mfa_totps"("user_id");

CREATE TABLE "mfa_webauthn_pks" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "credential_id" BYTEA NOT NULL,
    "spki_data" BYTEA NOT NULL, -- PKCS#8 DER SPKI
    "current_challenge" BYTEA,
    "current_challenge_expires_at" TIMESTAMPTZ
);

ALTER TABLE "mfa_webauthn_pks"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

CREATE INDEX ON "mfa_webauthn_pks"("user_id");

CREATE INDEX ON "mfa_webauthn_pks"("credential_id");

--- Organizations and Projects

CREATE TABLE "organizations" (
    "id" UUID PRIMARY KEY,
    "google_customer_id" VARCHAR(255) UNIQUE,
    "google_hosted_domain" VARCHAR(255) UNIQUE,
    "name" VARCHAR(255) NOT NULL,
    "owner_id" UUID NOT NULL
);

-- Explicitly not cascading on delete.
ALTER TABLE "organizations"
ADD FOREIGN KEY("owner_id") REFERENCES "users"("id");

CREATE INDEX ON "organizations"("google_customer_id");
CREATE INDEX ON "organizations"("google_hosted_domain");

CREATE TABLE "projects" (
    "id" UUID PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "organization_id" UUID NOT NULL
);

ALTER TABLE "projects"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

CREATE TABLE "policies" (
    "id" UUID PRIMARY KEY,
    "organization_id" UUID NOT NULL,
    "project_id" UUID,
    "name" VARCHAR(255) NOT NULL,
    "description" TEXT,
    "policy" VARCHAR(255) NOT NULL
);

ALTER TABLE "policies"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id")
ON DELETE CASCADE;

ALTER TABLE "policies"
ADD FOREIGN KEY("project_id") REFERENCES "projects"("id")
ON DELETE CASCADE;

CREATE INDEX ON "policies"("organization_id");
CREATE INDEX ON "policies"("project_id");

CREATE TABLE "roles" (
    "id" UUID PRIMARY KEY,
    "organization_id" UUID NOT NULL,
    "name" VARCHAR(255) NOT NULL,
    "description" TEXT,
    "inline_policy" VARCHAR(255)
);

ALTER TABLE "roles"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id")
ON DELETE CASCADE;

CREATE INDEX ON "roles"("organization_id");

CREATE TABLE "role_policies" (
    "role_id" UUID NOT NULL,
    "policy_id" UUID NOT NULL,
    PRIMARY KEY("role_id", "policy_id")
);

ALTER TABLE "role_policies"
ADD FOREIGN KEY("role_id") REFERENCES "roles"("id")
ON DELETE CASCADE;

ALTER TABLE "role_policies"
ADD FOREIGN KEY("policy_id") REFERENCES "policies"("id")
ON DELETE CASCADE;

CREATE INDEX ON "role_policies"("policy_id", "role_id");

CREATE TABLE "organization_members" (
    "organization_id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "invited_by_id" UUID,
    "inline_policy" VARCHAR(255),
    PRIMARY KEY("organization_id", "user_id")
);

ALTER TABLE "organization_members"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

ALTER TABLE "organization_members"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

ALTER TABLE "organization_members"
ADD FOREIGN KEY("invited_by_id") REFERENCES "users"("id")
ON DELETE SET NULL;

CREATE INDEX ON "organization_members"("user_id", "organization_id");

CREATE TABLE "organization_member_policies" (
    "organization_id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "policy_id" UUID NOT NULL,
    PRIMARY KEY("organization_id", "user_id", "policy_id")
);

ALTER TABLE "organization_member_policies"
ADD FOREIGN KEY("organization_id", "user_id") REFERENCES "organization_members"("organization_id", "user_id")
ON DELETE CASCADE;

ALTER TABLE "organization_member_policies"
ADD FOREIGN KEY("policy_id") REFERENCES "policies"("id")
ON DELETE CASCADE;

CREATE INDEX ON "organization_member_policies"("policy_id");

CREATE TABLE "service_accounts" (
    "id" UUID PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "organization_id" UUID NOT NULL,
    "project_id" UUID,
    "inline_policy" VARCHAR(255)
);

ALTER TABLE "service_accounts"
ADD FOREIGN KEY("project_id") REFERENCES "projects"("id");

ALTER TABLE "service_accounts"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

CREATE TABLE "service_account_tokens" (
    "id" UUID PRIMARY KEY,
    "active" BOOLEAN NOT NULL,
    "service_account_id" UUID NOT NULL,
    "token" BYTEA NOT NULL,
    "inline_policy" VARCHAR(255),
    "expires_at" TIMESTAMPTZ
);

ALTER TABLE "service_account_tokens"
ADD FOREIGN KEY("service_account_id") REFERENCES "service_accounts"("id");

-- Service account project id has to be the same as the project id of the policy.
CREATE TABLE "service_account_policies" (
    "service_account_id" UUID NOT NULL,
    "policy_id" UUID NOT NULL,
    PRIMARY KEY("service_account_id", "policy_id")
);

ALTER TABLE "service_account_policies"
ADD FOREIGN KEY("service_account_id") REFERENCES "service_accounts"("id")
ON DELETE CASCADE;

ALTER TABLE "service_account_policies"
ADD FOREIGN KEY("policy_id") REFERENCES "policies"("id")
ON DELETE CASCADE;

CREATE INDEX ON "service_account_policies"("policy_id", "service_account_id");

-- This is used for user registration requests via email and adding new email addresses to existing accounts.
-- When user_id is set, it indicates that the request is for an existing user to add a new email address.
CREATE TABLE "email_registration_requests" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID,
    "email" VARCHAR(255) NOT NULL,
    "code" BYTEA NOT NULL,
    "expires_at" TIMESTAMPTZ NOT NULL
);

CREATE INDEX ON "email_registration_requests"("code");

ALTER TABLE "email_registration_requests"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

-- This is used for inviting users to an organization.
-- When user_id is set, it indicates that the invite is for an existing user to join the organization and can
-- only be used for that selected user.
CREATE TABLE "organization_invitations" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID,
    "organization_id" UUID NOT NULL,
    "email" VARCHAR(255) NOT NULL,
    "invited_by_id" UUID NOT NULL,
    "expires_at" TIMESTAMPTZ
);

ALTER TABLE "organization_invitations"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

ALTER TABLE "organization_invitations"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id")
ON DELETE CASCADE;

ALTER TABLE "organization_invitations"
ADD FOREIGN KEY("invited_by_id") REFERENCES "users"("id")
ON DELETE CASCADE;
