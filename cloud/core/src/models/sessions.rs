use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::id::{Id, PrefixedId};
use crate::models::crypto::CryptoAlgorithm;
use crate::models::users::{User, UserId};

pub type UserSessionRequestId = Id<UserSessionRequest>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::user_session_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSessionRequest {
    pub id: UserSessionRequestId,
    pub device_name: String,
    pub device_ip: ipnetwork::IpNetwork,
    pub code: String,
    pub approved_by: Option<UserId>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for UserSessionRequest {
    const PREFIX: &'static str = "session_req";
}

pub struct UserSessionToken;

impl PrefixedId for UserSessionToken {
    const PREFIX: &'static str = "user_session_token";
}

pub type UserSessionTokenId = Id<UserSessionToken>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(primary_key(user_id, device_fingerprint))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSession {
    pub user_id: UserId,
    pub device_fingerprint: Vec<u8>,
    pub device_algorithm: CryptoAlgorithm,
    pub device_pk_data: Vec<u8>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
    pub last_ip: ipnetwork::IpNetwork,
    pub token_id: Option<UserSessionTokenId>,
    pub token: Option<String>,
    pub token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub type EmailRegistrationRequestId = Id<EmailRegistrationRequest>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = crate::schema::email_registration_requests)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EmailRegistrationRequest {
    pub id: EmailRegistrationRequestId,
    pub user_id: Option<UserId>,
    pub email: String,
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for EmailRegistrationRequest {
    const PREFIX: &'static str = "email_reg_req";
}
