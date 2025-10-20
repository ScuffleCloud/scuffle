use std::borrow::Cow;

use chrono::{DateTime, Utc};

id::impl_id!(pub UserId, "usr_");

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: UserId,
    pub preferred_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password_hash: Option<String>,
    pub primary_email: String,
    pub avatar_url: Option<String>,
    pub mfa_recovery_codes_last_used_at: Option<DateTime<Utc>>,
    pub mfa_recovery_codes_regenerated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser<'a> {
    #[builder(default)]
    pub id: UserId,
    pub preferred_name: Option<Cow<'a, str>>,
    pub first_name: Option<Cow<'a, str>>,
    pub last_name: Option<Cow<'a, str>>,
    pub password_hash: Option<Cow<'a, str>>,
    pub primary_email: Option<Cow<'a, str>>,
    pub avatar_url: Option<Cow<'a, str>>,
    pub mfa_recovery_codes_last_used_at: Option<DateTime<Utc>>,
    pub mfa_recovery_codes_regenerated_at: Option<DateTime<Utc>>,
}
