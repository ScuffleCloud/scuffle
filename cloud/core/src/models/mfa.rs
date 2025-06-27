use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use super::impl_enum;
use crate::id::{Id, PrefixedId};
use crate::models::users::{User, UserId};

impl_enum!(WebauthnAlgorithm, crate::schema::sql_types::WebauthnAlgorithm, {
    ED25519 => b"ED25519", // EdDSA using Ed25519 curve
    ESP256 => b"ESP256", // ECDSA using P-256 curve and SHA-256
    RS256 => b"RS256", // RSASSA-PKCS1-v1_5 using SHA-256
    ESP384 => b"ESP384", // ECDSA using P-384 curve and SHA-384
    ESP512 => b"ESP512" // ECDSA using P-521 curve and SHA-512
});

impl From<pb::scufflecloud::core::v1::WebauthnAlgorithm> for WebauthnAlgorithm {
    fn from(value: pb::scufflecloud::core::v1::WebauthnAlgorithm) -> Self {
        match value {
            pb::scufflecloud::core::v1::WebauthnAlgorithm::Ed25519 => WebauthnAlgorithm::ED25519,
            pb::scufflecloud::core::v1::WebauthnAlgorithm::Esp256 => WebauthnAlgorithm::ESP256,
            pb::scufflecloud::core::v1::WebauthnAlgorithm::Rs256 => WebauthnAlgorithm::RS256,
            pb::scufflecloud::core::v1::WebauthnAlgorithm::Esp384 => WebauthnAlgorithm::ESP384,
            pb::scufflecloud::core::v1::WebauthnAlgorithm::Esp512 => WebauthnAlgorithm::ESP512,
        }
    }
}

pub(crate) type MfaTotpId = Id<MfaTotp>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_totps)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTotp {
    pub id: MfaTotpId,
    pub user_id: UserId,
    pub secret: String,
}

impl PrefixedId for MfaTotp {
    const PREFIX: &'static str = "totp";
}

pub(crate) type MfaWebauthnPkId = Id<MfaWebauthnPk>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_webauthn_pks)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnPk {
    pub id: MfaWebauthnPkId,
    pub user_id: UserId,
    pub algorithm: WebauthnAlgorithm,
    pub pk_id: Vec<u8>,
    pub pk_data: Vec<u8>,
    pub current_challenge: Option<Vec<u8>>,
    pub current_challenge_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PrefixedId for MfaWebauthnPk {
    const PREFIX: &'static str = "webauthn";
}
