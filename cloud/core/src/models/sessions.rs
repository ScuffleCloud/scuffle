use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use super::impl_enum;
use crate::chrono_ext::ChronoDateTimeExt;
use crate::id::{Id, PrefixedId};
use crate::models::users::{User, UserId};

impl_enum!(DeviceAlgorithm, crate::schema::sql_types::DeviceAlgorithm, {
    RsaOaepSha256 => b"RSA_OAEP_SHA256",
});

impl From<pb::scufflecloud::core::v1::DeviceAlgorithm> for DeviceAlgorithm {
    fn from(value: pb::scufflecloud::core::v1::DeviceAlgorithm) -> Self {
        match value {
            pb::scufflecloud::core::v1::DeviceAlgorithm::RsaOaepSha256 => DeviceAlgorithm::RsaOaepSha256,
        }
    }
}

pub(crate) type UserSessionRequestId = Id<UserSessionRequest>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
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

impl From<UserSessionRequest> for pb::scufflecloud::core::v1::UserSessionRequest {
    fn from(value: UserSessionRequest) -> Self {
        pb::scufflecloud::core::v1::UserSessionRequest {
            id: value.id.to_string(),
            name: value.device_name,
            ip: value.device_ip.to_string(),
            approved_by: value.approved_by.map(|id| id.to_string()),
            expires_at: Some(value.expires_at.to_prost_timestamp_utc()),
        }
    }
}

pub(crate) type MagicLinkUserSessionRequestId = Id<MagicLinkUserSessionRequest>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::magic_link_user_session_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MagicLinkUserSessionRequest {
    pub id: MagicLinkUserSessionRequestId,
    pub user_id: UserId,
    pub code: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for MagicLinkUserSessionRequest {
    const PREFIX: &'static str = "magic_link";
}

#[derive(Debug)]
pub struct UserSessionToken;

impl PrefixedId for UserSessionToken {
    const PREFIX: &'static str = "user_session_token";
}

pub(crate) type UserSessionTokenId = Id<UserSessionToken>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, Clone)]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(primary_key(user_id, device_fingerprint))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSession {
    pub user_id: UserId,
    pub device_fingerprint: Vec<u8>,
    pub device_algorithm: DeviceAlgorithm,
    pub device_pk_data: Vec<u8>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
    pub last_ip: ipnetwork::IpNetwork,
    pub token_id: Option<UserSessionTokenId>,
    pub token: Option<Vec<u8>>,
    pub token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub(crate) type EmailRegistrationRequestId = Id<EmailRegistrationRequest>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::email_registration_requests)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EmailRegistrationRequest {
    pub id: EmailRegistrationRequestId,
    pub user_id: Option<UserId>,
    pub email: String,
    pub code: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for EmailRegistrationRequest {
    const PREFIX: &'static str = "email_reg_req";
}
