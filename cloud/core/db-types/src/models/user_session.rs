use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::id::UserId;
use crate::models::sha256::Sha256;

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(user_id, device_fingerprint))]
pub struct UserSession {
    pub user_id: UserId,
    pub device_fingerprint: Sha256,
    pub token: String,
    pub token_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub last_login_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(user_id, device_fingerprint))]
pub struct NewUserSession<'a> {
    #[builder(start_fn)]
    pub user_id: UserId,
    #[builder(start_fn)]
    pub device_fingerprint: Sha256,
    pub token: Cow<'a, str>,
    pub token_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    #[builder(default = chrono::Utc::now())]
    pub last_login_at: DateTime<Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
}
