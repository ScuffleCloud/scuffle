use std::time::SystemTime;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use super::impl_enum;
use crate::models::users::{User, UserId};

impl_enum!(DeviceAlgorithm, crate::schema::sql_types::DeviceAlgorithm, {
    RsaOaepSha256 => b"RSA_OAEP_SHA256",
});

impl From<DeviceAlgorithm> for pb::scufflecloud::core::v1::DeviceAlgorithm {
    fn from(value: DeviceAlgorithm) -> Self {
        match value {
            DeviceAlgorithm::RsaOaepSha256 => pb::scufflecloud::core::v1::DeviceAlgorithm::RsaOaepSha256,
        }
    }
}

impl From<pb::scufflecloud::core::v1::DeviceAlgorithm> for DeviceAlgorithm {
    fn from(value: pb::scufflecloud::core::v1::DeviceAlgorithm) -> Self {
        match value {
            pb::scufflecloud::core::v1::DeviceAlgorithm::RsaOaepSha256 => DeviceAlgorithm::RsaOaepSha256,
        }
    }
}

id::impl_id!(pub UserSessionRequestId, "sr_");

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::user_session_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSessionRequest {
    pub id: UserSessionRequestId,
    pub device_name: String,
    pub device_ip: ipnetwork::IpNetwork,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<UserId>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserSessionRequest> for pb::scufflecloud::core::v1::UserSessionRequest {
    fn from(value: UserSessionRequest) -> Self {
        pb::scufflecloud::core::v1::UserSessionRequest {
            id: value.id.to_string(),
            name: value.device_name,
            ip: value.device_ip.to_string(),
            approved_by: value.approved_by.map(|id| id.to_string()),
            expires_at: Some(SystemTime::from(value.expires_at).into()),
        }
    }
}

id::impl_id!(pub MagicLinkRequestId, "ml_");

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::magic_link_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MagicLinkRequest {
    pub id: MagicLinkRequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    pub email: String,
    pub code: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

id::impl_id!(pub UserSessionTokenId, "st_");

/// Does not represent a real database table as it is always part of a [`UserSession`].
#[derive(Debug, serde::Serialize)]
pub struct UserSessionToken {
    pub id: UserSessionTokenId,
    pub token: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(
    Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, Clone, serde_derive::Serialize,
)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<UserSessionTokenId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub mfa_pending: bool,
}

impl UserSession {
    pub fn into_pb(
        self,
        last_ip: pb::scufflecloud::core::v1::IpAddressInfo,
        last_user_agent: Option<pb::scufflecloud::core::v1::UserAgent>,
    ) -> pb::scufflecloud::core::v1::UserSession {
        pb::scufflecloud::core::v1::UserSession {
            user_id: self.user_id.to_string(),
            device_fingerprint: self.device_fingerprint,
            last_used_at: Some(SystemTime::from(self.last_used_at).into()),
            last_ip: Some(last_ip),
            last_user_agent,
            token_id: self.token_id.map(|id| id.to_string()),
            token_expires_at: self.token_expires_at.map(|t| SystemTime::from(t).into()),
            expires_at: Some(SystemTime::from(self.expires_at).into()),
            mfa_pending: self.mfa_pending,
        }
    }
}
