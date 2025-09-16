use core_db_types::models::{NewUserEmailRequest, User, UserEmail, UserGoogleAccount};

use crate::macros::impl_cedar_identity;

impl crate::CedarEntity for User {
    const ENTITY_TYPE: &'static str = "User";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    // async fn parents(&self, global: &Arc<G>) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
    //     let organization_ids = global
    //         .organization_member_by_user_id_loader()
    //         .load(self.id)
    //         .await
    //         .ok()
    //         .into_tonic_internal_err("failed to query organization members")?
    //         .into_tonic_not_found("user not found")?
    //         .into_iter()
    //         .map(|m| m.organization_id)
    //         .map(|id| CedarEntity::<G>::entity_uid(&id))
    //         .collect::<HashSet<_>>();

    //     Ok(organization_ids)
    // }
}


impl crate::CedarEntity for UserEmail {
    const ENTITY_TYPE: &'static str = "UserEmail";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.email)
    }
}

impl_cedar_identity!(NewUserEmailRequest);

impl crate::CedarEntity for UserGoogleAccount {
    const ENTITY_TYPE: &'static str = "UserGoogleAccount";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(&self.sub)
    }
}
