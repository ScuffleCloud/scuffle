use crate::CoreConfig;
use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::organization_invitations_service_server::OrganizationInvitationsService
    for CoreSvc<G>
{
    async fn create_organization_invitation(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn list_organization_invitations_by_organization(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitationList>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn list_orgnization_invites_by_user(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitationList>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_organization_invitation(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationInvitationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn accept_organization_invitation(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationInvitationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn decline_organization_invitation(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationInvitationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
