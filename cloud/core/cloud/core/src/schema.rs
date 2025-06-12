// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "mfa_factor_type"))]
    pub struct MfaFactorType;
}

diesel::table! {
    email_registration_requests (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        token -> Varchar,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MfaFactorType;

    mfa_factors (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[sql_name = "type"]
        type_ -> MfaFactorType,
        #[max_length = 255]
        secret -> Varchar,
    }
}

diesel::table! {
    organization_invites (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        organization_id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        invited_by_id -> Uuid,
        expiries_at -> Nullable<Timestamptz>,
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
        #[max_length = 255]
        token -> Varchar,
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
    user_sessions (user_id, device_id) {
        user_id -> Uuid,
        device_id -> Uuid,
        #[max_length = 255]
        device_public_key -> Varchar,
        last_used_at -> Timestamptz,
        last_ip -> Inet,
        #[max_length = 255]
        token -> Nullable<Varchar>,
        token_expires_at -> Nullable<Timestamptz>,
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
diesel::joinable!(mfa_factors -> users (user_id));
diesel::joinable!(organization_invites -> organizations (organization_id));
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
diesel::joinable!(user_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    email_registration_requests,
    mfa_factors,
    organization_invites,
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
    user_sessions,
    users,
);
