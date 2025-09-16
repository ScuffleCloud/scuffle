use std::collections::HashSet;

use core_db_types::models::{
    Organization, OrganizationInvitation, OrganizationMember, Policy, Project, Role, ServiceAccount, ServiceAccountToken,
};

use crate::CedarIdentifiable;
use crate::macros::{cedar_entity, cedar_entity_id};

cedar_entity!(Organization);

cedar_entity!(Project);

cedar_entity!(Policy);

cedar_entity!(Role);

impl crate::CedarIdentifiable for OrganizationMember {
    const ENTITY_TYPE: &'static str = "OrganizationMember";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(format!(
            "{}:{}",
            self.organization_id.to_string_unprefixed(),
            self.user_id.to_string_unprefixed()
        ))
    }
}

impl crate::CedarEntity for OrganizationMember {
    async fn parents(&self, _: &impl core_traits::Global) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
        Ok(std::iter::once(self.organization_id.entity_uid()).collect())
    }
}

cedar_entity_id!(ServiceAccount);

impl crate::CedarEntity for ServiceAccount {
    async fn parents(&self, _global: &impl core_traits::Global) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
        Ok(std::iter::once(self.organization_id.entity_uid())
            .chain(self.project_id.map(|id| id.entity_uid()))
            .collect())
    }
}

cedar_entity!(ServiceAccountToken);

cedar_entity_id!(OrganizationInvitation);

impl crate::CedarEntity for OrganizationInvitation {
    // async fn additional_attributes(
    //     &self,
    //     global: &Arc<G>,
    // ) -> Result<serde_json::value::Map<String, serde_json::Value>, tonic::Status> {
    //     let organization = common::get_organization_by_id(global, self.organization_id).await?;
    //     let organization_attr = organization.attributes(global).await?;

    //     Ok([("organization".to_string(), serde_json::Value::Object(organization_attr))]
    //         .into_iter()
    //         .collect())
    // }
}
