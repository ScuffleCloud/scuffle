// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "external_provider"))]
    pub struct ExternalProvider;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "mfa_factor_type"))]
    pub struct MfaFactorType;
}

diesel::table! {
    devices (id) {
        id -> Uuid,
        #[max_length = 255]
        fingerprint -> Varchar,
    }
}

diesel::table! {
    email_invites (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        organization_id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        token -> Varchar,
        expiry -> Timestamptz,
    }
}

diesel::table! {
    email_registration_requests (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        token -> Varchar,
        expiry -> Timestamptz,
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
    organization_members (organization_id, user_id) {
        organization_id -> Uuid,
        user_id -> Uuid,
        policies -> Jsonb,
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
    projects (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        organization_id -> Uuid,
    }
}

diesel::table! {
    service_account_tokens (id) {
        id -> Uuid,
        active -> Bool,
        service_account_id -> Uuid,
        #[max_length = 255]
        token -> Varchar,
        policies -> Nullable<Jsonb>,
        expiry -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    service_accounts (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        organization_id -> Uuid,
        project_id -> Nullable<Uuid>,
        policies -> Nullable<Jsonb>,
    }
}

diesel::table! {
    session_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        device_id -> Uuid,
        last_used -> Timestamptz,
        last_ip -> Cidr,
        expiry -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ExternalProvider;

    user_connections (id) {
        id -> Uuid,
        user_id -> Uuid,
        provider -> ExternalProvider,
        #[max_length = 255]
        external_id -> Varchar,
        #[max_length = 255]
        access_token -> Varchar,
        #[max_length = 255]
        refresh_token -> Varchar,
    }
}

diesel::table! {
    user_emails (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        primary -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password_hash -> Nullable<Varchar>,
    }
}

diesel::joinable!(email_invites -> organizations (organization_id));
diesel::joinable!(email_invites -> users (user_id));
diesel::joinable!(email_registration_requests -> users (user_id));
diesel::joinable!(mfa_factors -> users (user_id));
diesel::joinable!(organization_members -> organizations (organization_id));
diesel::joinable!(organizations -> users (owner_id));
diesel::joinable!(projects -> organizations (organization_id));
diesel::joinable!(service_account_tokens -> service_accounts (service_account_id));
diesel::joinable!(service_accounts -> organizations (organization_id));
diesel::joinable!(service_accounts -> projects (project_id));
diesel::joinable!(session_tokens -> devices (device_id));
diesel::joinable!(session_tokens -> users (user_id));
diesel::joinable!(user_connections -> users (user_id));
diesel::joinable!(user_emails -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    email_invites,
    email_registration_requests,
    mfa_factors,
    organization_members,
    organizations,
    projects,
    service_account_tokens,
    service_accounts,
    session_tokens,
    user_connections,
    user_emails,
    users,
);
