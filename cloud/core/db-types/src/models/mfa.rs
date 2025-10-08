use std::time::SystemTime;

use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::models::users::{User, UserId};

id::impl_id!(pub MfaTotpCredentialId, "mft_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::mfa_totp_credentials)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTotpCredential {
    pub id: MfaTotpCredentialId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub secret: Vec<u8>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
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

id::impl_id!(pub MfaWebauthnCredentialId, "mfw_");

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug, serde_derive::Serialize)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnCredential {
    pub id: MfaWebauthnCredentialId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub credential_id: Vec<u8>,
    #[serde(skip)] // cedar doesn't support json values
    pub credential: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter: Option<i64>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl From<MfaWebauthnCredential> for pb::scufflecloud::core::v1::WebauthnCredential {
    fn from(value: MfaWebauthnCredential) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.to_string(),
            name: value.name,
            last_used_at_utc: Some(SystemTime::from(value.last_used_at).into()),
            created_at: Some(value.id.datetime().into()),
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

id::impl_id!(pub MfaRecoveryCodeId, "mrc_");

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
