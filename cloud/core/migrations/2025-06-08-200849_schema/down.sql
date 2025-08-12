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

DROP TABLE IF EXISTS "mfa_recovery_codes" CASCADE;
DROP TABLE IF EXISTS "mfa_webauthn_credentials" CASCADE;
DROP TABLE IF EXISTS "mfa_webauthn_reg_sessions" CASCADE;
DROP TABLE IF EXISTS "mfa_webauthn_auth_sessions" CASCADE;
DROP TABLE IF EXISTS "mfa_totp_credentials" CASCADE;
DROP TABLE IF EXISTS "mfa_totp_reg_sessions" CASCADE;

DROP TABLE IF EXISTS "user_sessions" CASCADE;
DROP TYPE IF EXISTS "device_algorithm" CASCADE;
DROP TABLE IF EXISTS "magic_link_user_session_requests" CASCADE;
DROP TABLE IF EXISTS "user_session_requests" CASCADE;
DROP TABLE IF EXISTS "user_google_accounts" CASCADE;
DROP TABLE IF EXISTS "user_emails" CASCADE;
DROP TABLE IF EXISTS "users" CASCADE;
