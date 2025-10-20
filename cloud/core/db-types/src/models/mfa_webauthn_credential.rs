use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::{UserId, sha256::Sha256};

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnCredential {
    pub id: Sha256,
    pub user_id: UserId,
    pub name: Option<String>,
    pub credential: serde_json::Value,
    pub counter: i64,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMfaWebauthnCredential<'a> {
    #[builder(default)]
    pub id: Sha256,
    pub user_id: UserId,
    pub name: Option<Cow<'a, str>>,
    pub credential: serde_json::Value,
    pub counter: i64,
    pub last_used_at: Option<DateTime<Utc>>,
}
