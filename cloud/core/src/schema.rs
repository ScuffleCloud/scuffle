// @generated automatically by Diesel CLI.

pub(crate) mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "device_algorithm"))]
    pub struct DeviceAlgorithm;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "webauthn_algorithm"))]
    pub struct WebauthnAlgorithm;
}

diesel::table! {
    email_registration_requests (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 255]
        email -> Varchar,
        code -> Bytea,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    mfa_totps (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        secret -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WebauthnAlgorithm;

    mfa_webauthn_pks (id) {
        id -> Uuid,
        user_id -> Uuid,
        algorithm -> WebauthnAlgorithm,
        pk_id -> Bytea,
        pk_data -> Bytea,
        current_challenge -> Nullable<Bytea>,
        current_challenge_expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    organization_invitations (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        organization_id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        invited_by_id -> Uuid,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    organization_member_policies (organization_id, user_id, policy_id) {
        organization_id -> Uuid,
        user_id -> Uuid,
        policy_id -> Uuid,
    }
}

diesel::table! {
    organization_members (organization_id, user_id) {
        organization_id -> Uuid,
        user_id -> Uuid,
        invited_by_id -> Nullable<Uuid>,
        inline_policy -> Nullable<Jsonb>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        owner_id -> Uuid,
    }
}

diesel::table! {
    policies (id) {
        id -> Uuid,
        organization_id -> Uuid,
        project_id -> Nullable<Uuid>,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        policy -> Jsonb,
    }
}

diesel::table! {
    projects (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        organization_id -> Uuid,
    }
}

diesel::table! {
    role_policies (role_id, policy_id) {
        role_id -> Uuid,
        policy_id -> Uuid,
    }
}

diesel::table! {
    roles (id) {
        id -> Uuid,
        organization_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        inline_policy -> Nullable<Jsonb>,
    }
}

diesel::table! {
    service_account_policies (service_account_id, policy_id) {
        service_account_id -> Uuid,
        policy_id -> Uuid,
    }
}

diesel::table! {
    service_account_tokens (id) {
        id -> Uuid,
        active -> Bool,
        service_account_id -> Uuid,
        token -> Bytea,
        inline_policy -> Nullable<Jsonb>,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    service_accounts (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        organization_id -> Uuid,
        project_id -> Nullable<Uuid>,
        inline_policy -> Nullable<Jsonb>,
    }
}

diesel::table! {
    user_emails (email) {
        #[max_length = 255]
        email -> Varchar,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_google_accounts (id) {
        #[max_length = 255]
        id -> Varchar,
        user_id -> Uuid,
        #[max_length = 255]
        access_token -> Varchar,
        #[max_length = 255]
        refresh_token -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_session_requests (id) {
        id -> Uuid,
        #[max_length = 255]
        device_name -> Varchar,
        device_ip -> Inet,
        #[max_length = 6]
        code -> Varchar,
        approved_by -> Nullable<Uuid>,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DeviceAlgorithm;

    user_sessions (user_id, device_fingerprint) {
        user_id -> Uuid,
        #[max_length = 256]
        device_fingerprint -> Bit,
        device_algorithm -> DeviceAlgorithm,
        device_pk_data -> Bytea,
        last_used_at -> Timestamptz,
        last_ip -> Inet,
        token_id -> Nullable<Uuid>,
        token -> Nullable<Bytea>,
        token_expires_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        preferred_name -> Nullable<Varchar>,
        #[max_length = 255]
        first_name -> Nullable<Varchar>,
        #[max_length = 255]
        last_name -> Nullable<Varchar>,
        #[max_length = 255]
        password_hash -> Nullable<Varchar>,
        #[max_length = 255]
        primary_email -> Varchar,
    }
}

diesel::joinable!(email_registration_requests -> users (user_id));
diesel::joinable!(mfa_totps -> users (user_id));
diesel::joinable!(mfa_webauthn_pks -> users (user_id));
diesel::joinable!(organization_invitations -> organizations (organization_id));
diesel::joinable!(organization_member_policies -> policies (policy_id));
diesel::joinable!(organization_members -> organizations (organization_id));
diesel::joinable!(organizations -> users (owner_id));
diesel::joinable!(policies -> organizations (organization_id));
diesel::joinable!(policies -> projects (project_id));
diesel::joinable!(projects -> organizations (organization_id));
diesel::joinable!(role_policies -> policies (policy_id));
diesel::joinable!(role_policies -> roles (role_id));
diesel::joinable!(roles -> organizations (organization_id));
diesel::joinable!(service_account_policies -> policies (policy_id));
diesel::joinable!(service_account_policies -> service_accounts (service_account_id));
diesel::joinable!(service_account_tokens -> service_accounts (service_account_id));
diesel::joinable!(service_accounts -> organizations (organization_id));
diesel::joinable!(service_accounts -> projects (project_id));
diesel::joinable!(user_google_accounts -> users (user_id));
diesel::joinable!(user_session_requests -> users (approved_by));
diesel::joinable!(user_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    email_registration_requests,
    mfa_totps,
    mfa_webauthn_pks,
    organization_invitations,
    organization_member_policies,
    organization_members,
    organizations,
    policies,
    projects,
    role_policies,
    roles,
    service_account_policies,
    service_account_tokens,
    service_accounts,
    user_emails,
    user_google_accounts,
    user_session_requests,
    user_sessions,
    users,
);
