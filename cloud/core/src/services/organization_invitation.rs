use core_pb::v1::organization_invitation_service_server::OrganizationInvitationService;
use core_pb::v1::{
    OrganizationInvitationAcceptRequest, OrganizationInvitationAcceptResponse, OrganizationInvitationCreateRequest,
    OrganizationInvitationCreateResponse, OrganizationInvitationDeclineRequest, OrganizationInvitationDeclineResponse,
    OrganizationInvitationGetRequest, OrganizationInvitationGetResponse, OrganizationInvitationListRequest,
    OrganizationInvitationListResponse, OrganizationInvitationRevokeRequest, OrganizationInvitationRevokeResponse,
};
use tonic::async_trait;

#[async_trait]
impl<G: core_traits::Global> OrganizationInvitationService for crate::services::CoreSvc<G> {
    async fn create(
        &self,
        request: tonic::Request<OrganizationInvitationCreateRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationInvitationCreateResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list(
        &self,
        request: tonic::Request<OrganizationInvitationListRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationInvitationListResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn get(
        &self,
        request: tonic::Request<OrganizationInvitationGetRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationInvitationGetResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn accept(
        &self,
        request: tonic::Request<OrganizationInvitationAcceptRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationInvitationAcceptResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn decline(
        &self,
        request: tonic::Request<OrganizationInvitationDeclineRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationInvitationDeclineResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn revoke(
        &self,
        request: tonic::Request<OrganizationInvitationRevokeRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationInvitationRevokeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
