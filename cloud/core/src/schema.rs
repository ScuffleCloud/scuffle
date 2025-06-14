// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "external_provider"))]
    pub struct ExternalProvider;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "mfa_factor_type"))]
    pub struct MfaFactorType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "resource_owner_type"))]
    pub struct ResourceOwnerType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ResourceOwnerType;

    api_tokens (id) {
        id -> Uuid,
        active -> Bool,
        resource_owner_type -> ResourceOwnerType,
        resource_owner_id -> Uuid,
        #[max_length = 255]
        token -> Varchar,
        policies -> Nullable<Jsonb>,
        expiry -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    email_login_requests (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token -> Varchar,
        expiry -> Timestamptz,
    }
}

diesel::table! {
    email_registration_requests (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        organization_id -> Nullable<Uuid>,
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
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
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
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password -> Nullable<Varchar>,
    }
}

diesel::joinable!(email_login_requests -> users (user_id));
diesel::joinable!(email_registration_requests -> organizations (organization_id));
diesel::joinable!(email_registration_requests -> users (user_id));
diesel::joinable!(mfa_factors -> users (user_id));
diesel::joinable!(organization_members -> organizations (organization_id));
diesel::joinable!(organizations -> users (owner_id));
diesel::joinable!(projects -> organizations (organization_id));
diesel::joinable!(service_accounts -> organizations (organization_id));
diesel::joinable!(service_accounts -> projects (project_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(user_connections -> users (user_id));
diesel::joinable!(user_emails -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_tokens,
    email_login_requests,
    email_registration_requests,
    mfa_factors,
    organization_members,
    organizations,
    projects,
    service_accounts,
    sessions,
    user_connections,
    user_emails,
    users,
);
