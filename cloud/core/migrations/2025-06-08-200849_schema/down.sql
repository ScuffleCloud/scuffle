DROP TABLE IF EXISTS "organization_invitations" CASCADE;
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

DROP TABLE IF EXISTS "mfa_webauthn_pks" CASCADE;
DROP TABLE IF EXISTS "mfa_totps" CASCADE;

DROP TABLE IF EXISTS "user_sessions" CASCADE;
DROP TABLE IF EXISTS "crypto_algorithm" CASCADE;
DROP TABLE IF EXISTS "user_session_requests" CASCADE;
DROP TABLE IF EXISTS "user_google_accounts" CASCADE;
DROP TABLE IF EXISTS "user_emails" CASCADE;
DROP TABLE IF EXISTS "users" CASCADE;
