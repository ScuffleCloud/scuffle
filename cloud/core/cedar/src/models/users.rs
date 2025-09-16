use core_db_types::models::{NewUserEmailRequest, User, UserEmail, UserGoogleAccount};
use core_traits::OptionExt;

use crate::macros::{cedar_entity, cedar_entity_id};
use crate::{CedarIdentifiable, JsonEntityUid};

cedar_entity_id!(User);

impl crate::CedarEntity for User {
    async fn parents(
        &self,
        global: &impl core_traits::Global,
    ) -> Result<impl IntoIterator<Item = JsonEntityUid>, tonic::Status> {
        Ok(global
            .organization_member_by_user_id_loader()
            .load(self.id)
            .await
            .ok()
            .into_tonic_internal_err("failed to query organization members")?
            .into_iter()
            .flatten()
            .map(|m| m.organization_id)
            .map(|id| id.entity_uid()))
    }
}

impl crate::CedarIdentifiable for UserEmail {
    const ENTITY_TYPE: &'static str = "UserEmail";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.email)
    }
}

impl crate::CedarEntity for UserEmail {}

cedar_entity!(NewUserEmailRequest);

impl crate::CedarIdentifiable for UserGoogleAccount {
    const ENTITY_TYPE: &'static str = "UserGoogleAccount";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.sub)
    }
}

impl crate::CedarEntity for UserGoogleAccount {}
