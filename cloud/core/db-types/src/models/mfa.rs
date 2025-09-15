use std::time::SystemTime;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::cedar::CedarEntity;
use crate::id::{Id, PrefixedId};
use crate::models::users::{User, UserId};

pub type MfaTotpCredentialId = Id<MfaTotpCredential>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::mfa_totp_credentials)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTotpCredential {
    pub id: MfaTotpCredentialId,
    pub user_id: UserId,
    pub name: String,
    pub secret: Vec<u8>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for MfaTotpCredential {
    const PREFIX: &'static str = "mft";
}

impl CedarEntity for MfaTotpCredential {
    const ENTITY_TYPE: &'static str = "MfaTotpCredential";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

impl From<MfaTotpCredential> for pb::scufflecloud::core::v1::TotpCredential {
    fn from(value: MfaTotpCredential) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.to_string(),
            name: value.name,
            last_used_at_utc: Some(SystemTime::from(value.last_used_at).into()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_totp_reg_sessions)]
#[diesel(primary_key(user_id))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTotpRegistrationSession {
    pub user_id: UserId,
    pub secret: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub type MfaWebauthnCredentialId = Id<MfaWebauthnCredential>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnCredential {
    pub id: MfaWebauthnCredentialId,
    pub user_id: UserId,
    pub name: String,
    pub credential_id: Vec<u8>,
    pub credential: serde_json::Value,
    pub counter: Option<i64>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl PrefixedId for MfaWebauthnCredential {
    const PREFIX: &'static str = "mfw";
}

impl CedarEntity for MfaWebauthnCredential {
    const ENTITY_TYPE: &'static str = "MfaWebauthnCredential";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }
}

impl From<MfaWebauthnCredential> for pb::scufflecloud::core::v1::WebauthnCredential {
    fn from(value: MfaWebauthnCredential) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.to_string(),
            name: value.name,
            last_used_at_utc: Some(SystemTime::from(value.last_used_at).into()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_webauthn_reg_sessions)]
#[diesel(primary_key(user_id))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnRegistrationSession {
    pub user_id: UserId,
    pub state: serde_json::Value,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_webauthn_auth_sessions)]
#[diesel(primary_key(user_id))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnAuthenticationSession {
    pub user_id: UserId,
    pub state: serde_json::Value,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub type MfaRecoveryCodeId = Id<MfaRecoveryCode>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_recovery_codes)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaRecoveryCode {
    pub id: MfaRecoveryCodeId,
    pub user_id: UserId,
    pub code_hash: String,
}

impl PrefixedId for MfaRecoveryCode {
    const PREFIX: &'static str = "mrc";
}
