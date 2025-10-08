use core_db_types::models::{
    Organization, OrganizationId, OrganizationInvitation, OrganizationInvitationId, OrganizationMember, Policy, PolicyId,
    Project, ProjectId, Role, RoleId, ServiceAccount, ServiceAccountId, ServiceAccountToken, ServiceAccountTokenId,
};
use ext_traits::OptionExt;

use crate::macros::{cedar_entity, cedar_entity_id};
use crate::{CedarIdentifiable, EntityTypeName, JsonEntityUid, entity_type_name};

cedar_entity!(Organization, OrganizationId);

cedar_entity!(Project, ProjectId);

cedar_entity!(Policy, PolicyId);

cedar_entity!(Role, RoleId);

impl crate::CedarIdentifiable for OrganizationMember {
    const ENTITY_TYPE: EntityTypeName = entity_type_name!("OrganizationMember");

    fn entity_id(&self) -> cedar_policy::EntityId {
        cedar_policy::EntityId::new(format!("{}:{}", self.organization_id.ulid(), self.user_id.ulid()))
    }
}

impl crate::CedarEntity for OrganizationMember {
    async fn parents(&self, _: &impl core_traits::Global) -> Result<impl IntoIterator<Item = JsonEntityUid>, tonic::Status> {
        Ok(std::iter::once(self.organization_id.entity_uid()))
    }
}

cedar_entity_id!(ServiceAccount, ServiceAccountId);

impl crate::CedarEntity for ServiceAccount {
    async fn parents(&self, _: &impl core_traits::Global) -> Result<impl IntoIterator<Item = JsonEntityUid>, tonic::Status> {
        Ok(std::iter::once(self.organization_id.entity_uid()).chain(self.project_id.map(|id| id.entity_uid())))
    }
}

cedar_entity!(ServiceAccountToken, ServiceAccountTokenId);

cedar_entity_id!(OrganizationInvitation, OrganizationInvitationId);

impl crate::CedarEntity for OrganizationInvitation {
    async fn additional_attributes(
        &self,
        global: &impl core_traits::Global,
    ) -> Result<impl serde::Serialize, tonic::Status> {
        #[derive(serde_derive::Serialize)]
        struct AdditionalAttrs {
            organization: Organization,
        }

        Ok(AdditionalAttrs {
            organization: global
                .organization_loader()
                .load(self.organization_id)
                .await
                .ok()
                .into_tonic_internal_err("failed to query organization")?
                .into_tonic_not_found("organization not found")?,
        })
    }
}
