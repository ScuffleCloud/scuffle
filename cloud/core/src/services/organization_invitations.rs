use crate::operations::Operation;
use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: core_traits::Global>
    pb::scufflecloud::core::v1::organization_invitations_service_server::OrganizationInvitationsService for CoreSvc<G>
{
    async fn create_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_organization_invitations_by_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListOrganizationInvitationsByOrganizationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitationList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_orgnization_invites_by_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListOrgnizationInvitesByUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitationList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn get_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn accept_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::AcceptOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationMember>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn decline_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeclineOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }
}
