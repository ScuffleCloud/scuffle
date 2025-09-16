use core_db_types::models::{Organization, OrganizationInvitation, OrganizationMember, Policy, Project, Role, ServiceAccount, ServiceAccountToken};

use crate::macros::impl_cedar_identity;

impl_cedar_identity!(Organization);

impl_cedar_identity!(Project);

impl_cedar_identity!(Policy);

impl_cedar_identity!(Role);

impl crate::CedarEntity for OrganizationMember {
    const ENTITY_TYPE: &'static str = "OrganizationMember";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(format!(
            "{}:{}",
            self.organization_id.to_string_unprefixed(),
            self.user_id.to_string_unprefixed()
        ))
    }

    // async fn parents(&self, _global: &Arc<G>) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
    //     Ok(std::iter::once(CedarEntity::<G>::entity_uid(&self.organization_id)).collect())
    // }
}

impl crate::CedarEntity for ServiceAccount {
    const ENTITY_TYPE: &'static str = "ServiceAccount";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

    // async fn parents(&self, _global: &Arc<G>) -> Result<HashSet<cedar_policy::EntityUid>, tonic::Status> {
    //     let mut parents = HashSet::new();
    //     parents.insert(CedarEntity::<G>::entity_uid(&self.organization_id));
    //     if let Some(project_id) = &self.project_id {
    //         parents.insert(CedarEntity::<G>::entity_uid(project_id));
    //     }
    //     Ok(parents)
    // }
}

impl_cedar_identity!(ServiceAccountToken);

impl crate::CedarEntity for OrganizationInvitation {
    const ENTITY_TYPE: &'static str = "OrganizationInvitation";

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(self.id.to_string_unprefixed())
    }

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
