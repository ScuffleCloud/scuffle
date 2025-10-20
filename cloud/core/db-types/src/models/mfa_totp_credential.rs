use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::models::UserId;

id::impl_id!(pub MfaTotpCredentialId, "mtt_");

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::mfa_totp_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTotpCredential {
    pub id: MfaTotpCredentialId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub url: String,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, diesel::Insertable, diesel::Identifiable, diesel::AsChangeset, bon::Builder)]
#[diesel(table_name = crate::schema::mfa_totp_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMfaTotpCredential<'a> {
    #[builder(default)]
    pub id: MfaTotpCredentialId,
    pub user_id: UserId,
    pub name: Option<Cow<'a, str>>,
    pub url: String,
    pub last_used_at: Option<DateTime<Utc>>,
}
