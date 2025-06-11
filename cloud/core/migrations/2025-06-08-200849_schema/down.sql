DROP TABLE IF EXISTS "email_invites" CASCADE;
DROP TABLE IF EXISTS "email_registration_requests" CASCADE;
DROP TABLE IF EXISTS "service_account_tokens" CASCADE;
DROP TABLE IF EXISTS "service_accounts" CASCADE;
DROP TABLE IF EXISTS "projects" CASCADE;
DROP TABLE IF EXISTS "organization_members" CASCADE;
DROP TABLE IF EXISTS "organizations" CASCADE;

DROP TABLE IF EXISTS "mfa_factors" CASCADE;
DROP TYPE IF EXISTS "mfa_factor_type" CASCADE;

DROP INDEX IF EXISTS "session_tokens_index_0" CASCADE;
DROP TABLE IF EXISTS "session_tokens" CASCADE;
DROP TABLE IF EXISTS "devices" CASCADE;
DROP INDEX IF EXISTS "user_connections_index_1" CASCADE;
DROP INDEX IF EXISTS "user_connections_index_0" CASCADE;
DROP TABLE IF EXISTS "user_connections" CASCADE;
DROP TYPE IF EXISTS "external_provider" CASCADE;
DROP INDEX IF EXISTS "user_emails_index_0" CASCADE;
DROP TABLE IF EXISTS "user_emails" CASCADE;
DROP TABLE IF EXISTS "users" CASCADE;
