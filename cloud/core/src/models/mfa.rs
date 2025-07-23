use diesel::Selectable;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

use crate::cedar::CedarEntity;
use crate::id::{Id, PrefixedId};
use crate::models::users::{User, UserId};

pub(crate) type MfaTotpId = Id<MfaTotp>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_totps)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTotp {
    pub id: MfaTotpId,
    pub user_id: UserId,
    pub secret: Vec<u8>,
}

impl PrefixedId for MfaTotp {
    const PREFIX: &'static str = "mft";
}

impl CedarEntity for MfaTotp {
    const ENTITY_TYPE: &'static str = "MfaTotp";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    fn attributes(&self) -> std::collections::HashMap<String, cedar_policy::RestrictedExpression> {
        self.id.attributes()
    }
}

pub(crate) type MfaWebauthnCredentialId = Id<MfaWebauthnCredential>;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Associations, Debug)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebauthnCredential {
    pub id: MfaWebauthnCredentialId,
    pub user_id: UserId,
    pub credential_id: Vec<u8>,
    pub spki_data: Vec<u8>,
}

impl PrefixedId for MfaWebauthnCredential {
    const PREFIX: &'static str = "mfw";
}

impl CedarEntity for MfaWebauthnCredential {
    const ENTITY_TYPE: &'static str = "MfaWebauthnPk";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    fn attributes(&self) -> std::collections::HashMap<String, cedar_policy::RestrictedExpression> {
        self.id.attributes()
    }
}

impl From<MfaWebauthnCredential> for pb::scufflecloud::core::v1::MfaWebauthnCredential {
    fn from(value: MfaWebauthnCredential) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.to_string(),
            credential_id: value.credential_id,
        }
    }
}
