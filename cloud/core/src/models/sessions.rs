use base64::Engine;
use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use super::impl_enum;
use crate::cedar::CedarEntity;
use crate::chrono_ext::ChronoDateTimeExt;
use crate::id::{Id, PrefixedId};
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

pub(crate) type UserSessionRequestId = Id<UserSessionRequest>;

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, serde::Serialize)]
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
    const PREFIX: &'static str = "sr";
}

impl CedarEntity for UserSessionRequest {
    const ENTITY_TYPE: &'static str = "UserSessionRequest";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
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

#[derive(Debug, Queryable, Selectable, Insertable, Identifiable, AsChangeset, serde::Serialize)]
#[diesel(table_name = crate::schema::magic_link_user_session_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MagicLinkUserSessionRequest {
    pub id: MagicLinkUserSessionRequestId,
    pub user_id: UserId,
    pub code: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for MagicLinkUserSessionRequest {
    const PREFIX: &'static str = "ml";
}

impl CedarEntity for MagicLinkUserSessionRequest {
    const ENTITY_TYPE: &'static str = "MagicLinkUserSessionRequest";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

pub(crate) type UserSessionTokenId = Id<UserSessionToken>;

/// Does not represent a real database table as it is always part of a [`UserSession`].
#[derive(Debug, serde::Serialize)]
pub struct UserSessionToken {
    pub id: UserSessionTokenId,
    pub token: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for UserSessionToken {
    const PREFIX: &'static str = "st";
}

impl CedarEntity for UserSessionToken {
    const ENTITY_TYPE: &'static str = "UserSessionToken";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, Clone, serde::Serialize)]
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
    pub mfa_pending: bool,
}

impl CedarEntity for UserSession {
    const ENTITY_TYPE: &'static str = "UserSession";

    fn entity_id(&self) -> cedar_policy::EntityId {
        let user_id = (*self.user_id).to_string();
        let fingerprint = base64::prelude::BASE64_STANDARD.encode(&self.device_fingerprint);
        cedar_policy::EntityId::new(format!("{user_id}:{fingerprint}"))
    }
}

impl From<UserSession> for pb::scufflecloud::core::v1::UserSession {
    fn from(value: UserSession) -> Self {
        pb::scufflecloud::core::v1::UserSession {
            user_id: value.user_id.to_string(),
            device_fingerprint: value.device_fingerprint,
            last_used_at: Some(value.last_used_at.to_prost_timestamp_utc()),
            last_ip: value.last_ip.to_string(),
            token_id: value.token_id.map(|id| id.to_string()),
            token_expires_at: value.token_expires_at.map(|t| t.to_prost_timestamp_utc()),
            expires_at: Some(value.expires_at.to_prost_timestamp_utc()),
            mfa_pending: value.mfa_pending,
        }
    }
}

pub(crate) type EmailRegistrationRequestId = Id<EmailRegistrationRequest>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde::Serialize)]
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
    const PREFIX: &'static str = "er";
}

impl CedarEntity for EmailRegistrationRequest {
    const ENTITY_TYPE: &'static str = "EmailRegistrationRequest";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}
