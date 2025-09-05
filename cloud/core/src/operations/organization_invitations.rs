use diesel::query_dsl::methods::{FilterDsl, FindDsl, SelectDsl};
use diesel::{ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::Action;
use crate::http_ext::RequestExt;
use crate::models::{
    Organization, OrganizationId, OrganizationInvitation, OrganizationInvitationId, OrganizationMember, User, UserId,
};
use crate::operations::{NoopOperationDriver, Operation, TransactionOperationDriver};
use crate::schema::{organization_invitations, organization_members, user_emails};
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, common};

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationInvitationRequest> {
    type Driver<'a> = TransactionOperationDriver<'a>;
    type Principal = User;
    type Resource = OrganizationInvitation;
    type Response = pb::scufflecloud::core::v1::OrganizationInvitation;

    const ACTION: Action = Action::CreateOrganizationInvitation;

    async fn load_principal(
        &mut self,
        _driver: &mut Self::Driver<'_>,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut Self::Driver<'_>) -> Result<Self::Resource, tonic::Status> {
        let session = self.session_or_err()?;

        let organization_id: OrganizationId = self
            .get_ref()
            .organization_id
            .parse()
            .into_tonic_internal_err("failed to parse id")?;

        let invited_user = user_emails::dsl::user_emails
            .find(&self.get_ref().email)
            .select(user_emails::dsl::user_id)
            .get_result::<UserId>(&mut driver.conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to query user email")?;

        Ok(OrganizationInvitation {
            id: OrganizationInvitationId::new(),
            user_id: invited_user,
            organization_id,
            email: self.get_ref().email.clone(),
            invited_by_id: session.user_id,
            expires_at: self
                .get_ref()
                .expires_in_s
                .map(|s| chrono::Utc::now() + chrono::Duration::seconds(s as i64)),
        })
    }

    async fn execute(
        self,
        driver: &mut Self::Driver<'_>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        diesel::insert_into(organization_invitations::dsl::organization_invitations)
            .values(&resource)
            .execute(&mut driver.conn)
            .await
            .into_tonic_internal_err("failed to insert organization invitation")?;

        Ok(resource.into())
    }
}

impl<G: CoreConfig> Operation<G>
    for tonic::Request<pb::scufflecloud::core::v1::ListOrganizationInvitationsByOrganizationRequest>
{
    type Driver<'a> = NoopOperationDriver;
    type Principal = User;
    type Resource = Organization;
    type Response = pb::scufflecloud::core::v1::OrganizationInvitationList;

    const ACTION: Action = Action::ListOrganizationInvitationsByOrganization;

    async fn load_principal(
        &mut self,
        _driver: &mut Self::Driver<'_>,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut Self::Driver<'_>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let organization_id: OrganizationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_organization_by_id(global, organization_id).await
    }

    async fn execute(
        self,
        _driver: &mut Self::Driver<'_>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let invitations = organization_invitations::dsl::organization_invitations
            .filter(organization_invitations::dsl::organization_id.eq(resource.id))
            .load::<OrganizationInvitation>(&mut db)
            .await
            .into_tonic_internal_err("failed to query organization invitations")?;

        Ok(pb::scufflecloud::core::v1::OrganizationInvitationList {
            invitations: invitations.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListOrgnizationInvitesByUserRequest> {
    type Driver<'a> = NoopOperationDriver;
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::OrganizationInvitationList;

    const ACTION: Action = Action::ListOrganizationInvitationsByUser;

    async fn load_principal(
        &mut self,
        _driver: &mut Self::Driver<'_>,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut Self::Driver<'_>,) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_user_by_id(global, user_id).await
    }

    async fn execute(
        self,
        _driver: &mut Self::Driver<'_>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let invitations = organization_invitations::dsl::organization_invitations
            .filter(organization_invitations::dsl::user_id.eq(resource.id))
            .load::<OrganizationInvitation>(&mut db)
            .await
            .into_tonic_internal_err("failed to query organization invitations")?;

        Ok(pb::scufflecloud::core::v1::OrganizationInvitationList {
            invitations: invitations.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::GetOrganizationInvitationRequest> {
    type Driver<'a> = NoopOperationDriver;
    type Principal = User;
    type Resource = OrganizationInvitation;
    type Response = pb::scufflecloud::core::v1::OrganizationInvitation;

    const ACTION: Action = Action::GetOrganizationInvitation;

    async fn load_principal(
        &mut self,
        _driver: &mut Self::Driver<'_>,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut Self::Driver<'_>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let id: OrganizationInvitationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        organization_invitations::dsl::organization_invitations
            .find(id)
            .first::<OrganizationInvitation>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to query organization invitation")?
            .into_tonic_not_found("organization invitation not found")
    }

    async fn execute(
        self,
        _driver: &mut Self::Driver<'_>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        Ok(resource.into())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::AcceptOrganizationInvitationRequest> {
    type Driver<'a> = TransactionOperationDriver<'a>;
    type Principal = User;
    type Resource = OrganizationInvitation;
    type Response = pb::scufflecloud::core::v1::OrganizationMember;

    const ACTION: Action = Action::AcceptOrganizationInvitation;

    async fn load_principal(
        &mut self,
        _driver: &mut Self::Driver<'_>,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut Self::Driver<'_>) -> Result<Self::Resource, tonic::Status> {
        let id: OrganizationInvitationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        organization_invitations::dsl::organization_invitations
            .find(id)
            .first::<OrganizationInvitation>(&mut driver.conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to query organization invitation")?
            .into_tonic_not_found("organization invitation not found")
    }

    async fn execute(
        self,
        driver: &mut Self::Driver<'_>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let Some(user_id) = resource.user_id else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::FailedPrecondition,
                "register first to accept this organization invitation",
                ErrorDetails::new(),
            ));
        };

        let organization_member = OrganizationMember {
            organization_id: resource.organization_id,
            user_id,
            invited_by_id: Some(resource.invited_by_id),
            inline_policy: None,
            created_at: chrono::Utc::now(),
        };

        diesel::insert_into(organization_members::dsl::organization_members)
            .values(&organization_member)
            .execute(&mut driver.conn)
            .await
            .into_tonic_internal_err("failed to insert organization member")?;

        Ok(organization_member.into())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::DeclineOrganizationInvitationRequest> {
    type Driver<'a> = TransactionOperationDriver<'a>;
    type Principal = User;
    type Resource = OrganizationInvitation;
    type Response = ();

    const ACTION: Action = Action::DeclineOrganizationInvitation;

    async fn load_principal(
        &mut self,
        _driver: &mut Self::Driver<'_>,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut Self::Driver<'_>) -> Result<Self::Resource, tonic::Status> {
        let id: OrganizationInvitationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        organization_invitations::dsl::organization_invitations
            .find(id)
            .first::<OrganizationInvitation>(&mut driver.conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to query organization invitation")?
            .into_tonic_not_found("organization invitation not found")
    }

    async fn execute(
        self,
        driver: &mut Self::Driver<'_>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        diesel::delete(organization_invitations::dsl::organization_invitations)
            .filter(organization_invitations::dsl::id.eq(resource.id))
            .execute(&mut driver.conn)
            .await
            .into_tonic_internal_err("failed to delete organization invitation")?;

        Ok(())
    }
}
