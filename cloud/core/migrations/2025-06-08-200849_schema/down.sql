DROP TABLE IF EXISTS "organization_invites" CASCADE;
DROP TABLE IF EXISTS "email_registration_requests" CASCADE;
DROP TABLE IF EXISTS "service_account_policies" CASCADE;
DROP TABLE IF EXISTS "service_account_tokens" CASCADE;
DROP TABLE IF EXISTS "service_accounts" CASCADE;
DROP TABLE IF EXISTS "organization_member_policies" CASCADE;
DROP TABLE IF EXISTS "organization_members" CASCADE;
DROP TABLE IF EXISTS "role_policies" CASCADE;
DROP TABLE IF EXISTS "roles" CASCADE;
DROP TABLE IF EXISTS "policies" CASCADE;
DROP TABLE IF EXISTS "projects" CASCADE;
DROP TABLE IF EXISTS "organizations" CASCADE;

DROP TABLE IF EXISTS "mfa_factors" CASCADE;
DROP TYPE IF EXISTS "mfa_factor_type" CASCADE;

DROP TABLE IF EXISTS "user_sessions" CASCADE;
DROP TABLE IF EXISTS "user_google_accounts" CASCADE;
DROP TABLE IF EXISTS "user_emails" CASCADE;
DROP TABLE IF EXISTS "users" CASCADE;
