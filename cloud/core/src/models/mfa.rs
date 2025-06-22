use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::id::{Id, PrefixedId};
use crate::models::crypto::CryptoAlgorithm;
use crate::models::users::{User, UserId};

pub type MfaTotpId = Id<MfaTotp>;

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

pub type MfaWebauthnPkId = Id<MfaWebauthnPk>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_webauthn_pks)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnPk {
    pub id: MfaWebauthnPkId,
    pub user_id: UserId,
    pub algorithm: CryptoAlgorithm,
    pub pk_id: Vec<u8>,
    pub pk_data: Vec<u8>,
    pub current_challenge: Option<Vec<u8>>,
    pub current_challenge_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PrefixedId for MfaWebauthnPk {
    const PREFIX: &'static str = "webauthn";
}
