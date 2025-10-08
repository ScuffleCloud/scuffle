use pb::scufflecloud::core::v1::organization_invitations_service_server::OrganizationInvitationsService;
use pb::scufflecloud::core::v1::{
    AcceptOrganizationInvitationRequest, CreateOrganizationInvitationRequest, DeclineOrganizationInvitationRequest,
    GetOrganizationInvitationRequest, ListOrganizationInvitationsByOrganizationRequest, ListOrgnizationInvitesByUserRequest,
    OrganizationInvitation, OrganizationInvitationList, OrganizationMember,
};

use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: core_traits::Global> OrganizationInvitationsService for CoreSvc<G> {
    async fn create_organization_invitation(
        &self,
        req: tonic::Request<CreateOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<OrganizationInvitation>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_organization_invitations_by_organization(
        &self,
        req: tonic::Request<ListOrganizationInvitationsByOrganizationRequest>,
    ) -> Result<tonic::Response<OrganizationInvitationList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_orgnization_invites_by_user(
        &self,
        req: tonic::Request<ListOrgnizationInvitesByUserRequest>,
    ) -> Result<tonic::Response<OrganizationInvitationList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn get_organization_invitation(
        &self,
        req: tonic::Request<GetOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<OrganizationInvitation>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn accept_organization_invitation(
        &self,
        req: tonic::Request<AcceptOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<OrganizationMember>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn decline_organization_invitation(
        &self,
        req: tonic::Request<DeclineOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
