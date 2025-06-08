-- User Management

CREATE TABLE "users" (
	"id" UUID NOT NULL UNIQUE,
	"name" VARCHAR(255) NOT NULL,
	"password" VARCHAR(255), -- Nullable for users who register via external providers
	PRIMARY KEY("id")
);

CREATE TABLE "user_emails" (
	"id" UUID NOT NULL,
	"user_id" UUID NOT NULL,
	"email" VARCHAR(255) NOT NULL,
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

CREATE UNIQUE INDEX "user_connection_index_0"
ON "user_connections" ("provider", "external_id");

CREATE UNIQUE INDEX "user_connection_index_1"
ON "user_connections" ("user_id", "provider");

ALTER TABLE "user_connections"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

-- "Magic Link" login requests via email.
-- This can be used by users who don't want to use a password.
CREATE TABLE "email_login_requests" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID NOT NULL,
	"token" VARCHAR(255) NOT NULL UNIQUE,
	"expiry" TIMESTAMPTZ NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "email_login_requests"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

CREATE TABLE "sessions" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID NOT NULL,
	"expiry" TIMESTAMPTZ,
	PRIMARY KEY("id")
);

ALTER TABLE "sessions"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

--- MFA

CREATE TYPE "mfa_factor_type" AS ENUM (
	'totp',
	'passkey'
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

CREATE TYPE "resource_owner_type" AS ENUM (
	'user',
	'service_account'
);

CREATE TABLE "api_tokens" (
	"id" UUID NOT NULL UNIQUE,
	"active" BOOLEAN NOT NULL,
	"resource_owner_type" RESOURCE_OWNER_TYPE NOT NULL,
	"resource_owner_id" UUID NOT NULL,
	"token" VARCHAR(255) NOT NULL,
	"policies" JSONB,
	"expiry" TIMESTAMPTZ,
	PRIMARY KEY("id")
);

-- This is used for user registration requests via email, email invites and adding new email addresses to existing accounts.
-- When organization_id is set, it indicates that the request is an invite from an organization.
-- When user_id is set, it indicates that the request is for an existing user to add a new email address.
-- Both should not be set.
CREATE TABLE "email_registration_requests" (
	"id" UUID NOT NULL UNIQUE,
	"user_id" UUID,
	"organization_id" UUID,
	"email" VARCHAR(255) NOT NULL,
	"token" VARCHAR(255) NOT NULL,
	"expiry" TIMESTAMPTZ NOT NULL,
	PRIMARY KEY("id")
);

ALTER TABLE "email_registration_requests"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

ALTER TABLE "email_registration_requests"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");
