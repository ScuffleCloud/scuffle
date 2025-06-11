-- User Management

CREATE TABLE "users" (
	"id" UUID NOT NULL UNIQUE,
	"name" VARCHAR(255) NOT NULL,
    "primary_email" UUID NOT NULL,
	"password_hash" VARCHAR(255), -- Nullable for users who register via external providers
	PRIMARY KEY("id")
);

ALTER TABLE "users"
ADD FOREIGN KEY("primary_email") REFERENCES "user_emails"("id");

CREATE TABLE "user_emails" (
	"id" UUID NOT NULL,
	"user_id" UUID NOT NULL,
	"email" VARCHAR(255) NOT NULL UNIQUE,
	PRIMARY KEY("id")
);

ALTER TABLE "user_emails"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

CREATE TYPE "external_provider" AS ENUM (
	'google'
);

CREATE TABLE "user_connections" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID NOT NULL,
	"provider" EXTERNAL_PROVIDER NOT NULL,
	"external_id" VARCHAR(255) NOT NULL,
	"access_token" VARCHAR(255) NOT NULL,
	"refresh_token" VARCHAR(255) NOT NULL,
	PRIMARY KEY("id")
);

CREATE UNIQUE INDEX "user_connections_index_0"
ON "user_connections" ("provider", "external_id");

CREATE UNIQUE INDEX "user_connections_index_1"
ON "user_connections" ("user_id", "provider");

ALTER TABLE "user_connections"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

CREATE TABLE "devices" (
	"id" UUID NOT NULL UNIQUE,
	"fingerprint" VARCHAR(255) NOT NULL,
	PRIMARY KEY("id")
);

CREATE TABLE "session_tokens" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID NOT NULL,
	"device_id" UUID NOT NULL,
    "last_used" TIMESTAMPTZ NOT NULL,
    "last_ip" CIDR NOT NULL,
	"expiry" TIMESTAMPTZ,
	PRIMARY KEY("id")
);

CREATE UNIQUE INDEX "session_tokens_index_0"
ON "session_tokens" ("user_id", "device_id");

ALTER TABLE "session_tokens"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

ALTER TABLE "session_tokens"
ADD FOREIGN KEY("device_id") REFERENCES "devices"("id");

--- MFA

CREATE TYPE "mfa_factor_type" AS ENUM (
	'totp',
	'webauthn' -- Passkeys are WebAuthn
);

CREATE TABLE "mfa_factors" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID NOT NULL,
	"type" MFA_FACTOR_TYPE NOT NULL,
	"secret" VARCHAR(255) NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "mfa_factors"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

--- Organizations and Projects

CREATE TABLE "organizations" (
	"id" UUID NOT NULL UNIQUE,
	"name" VARCHAR(255) NOT NULL,
	"owner_id" UUID NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "organizations"
ADD FOREIGN KEY("owner_id") REFERENCES "users"("id");

CREATE TABLE "organization_members" (
	"organization_id" UUID NOT NULL,
	"user_id" UUID NOT NULL UNIQUE,
	"policies" JSONB NOT NULL,
	PRIMARY KEY("organization_id", "user_id")
);

ALTER TABLE "organization_members"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

CREATE TABLE "projects" (
	"id" UUID NOT NULL UNIQUE,
	"name" VARCHAR(255) NOT NULL,
	"organization_id" UUID NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "projects"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

CREATE TABLE "service_accounts" (
	"id" UUID NOT NULL UNIQUE,
	"name" VARCHAR(255) NOT NULL,
	"organization_id" UUID NOT NULL,
	"project_id" UUID,
	"policies" JSONB,
	PRIMARY KEY("id")
);

ALTER TABLE "service_accounts"
ADD FOREIGN KEY("project_id") REFERENCES "projects"("id");

ALTER TABLE "service_accounts"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

CREATE TABLE "service_account_tokens" (
	"id" UUID NOT NULL UNIQUE,
	"active" BOOLEAN NOT NULL,
	"service_account_id" UUID NOT NULL,
	"token" VARCHAR(255) NOT NULL,
	"policies" JSONB,
	"expiry" TIMESTAMPTZ,
	PRIMARY KEY("id")
);

ALTER TABLE "service_account_tokens"
ADD FOREIGN KEY("service_account_id") REFERENCES "service_accounts"("id");

-- This is used for user registration requests via email and adding new email addresses to existing accounts.
-- When user_id is set, it indicates that the request is for an existing user to add a new email address.
CREATE TABLE "email_registration_requests" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID,
	"email" VARCHAR(255) NOT NULL,
	"token" VARCHAR(255) NOT NULL,
	"expiry" TIMESTAMPTZ NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "email_registration_requests"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

-- This is used for inviting users to an organization via email.
-- When user_id is set, it indicates that the invite is for an existing user to join the organization and can
-- only be used for that selected user.
CREATE TABLE "email_invites" (
    "id" UUID NOT NULL UNIQUE,
	"user_id" UUID,
	"organization_id" UUID NOT NULL,
	"email" VARCHAR(255) NOT NULL,
	"token" VARCHAR(255) NOT NULL,
	"expiry" TIMESTAMPTZ NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "email_invites"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

ALTER TABLE "email_invites"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");
