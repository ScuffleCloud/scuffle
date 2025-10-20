use core_pb::v1::organization_service_server::OrganizationService;
use core_pb::v1::{
    OrganizationCreateRequest, OrganizationCreateResponse, OrganizationGetRequest, OrganizationGetResponse,
    OrganizationListRequest, OrganizationListResponse, OrganizationUpdateRequest, OrganizationUpdateResponse,
};
use tonic::async_trait;

#[async_trait]
impl<G: core_traits::Global> OrganizationService for crate::services::CoreSvc<G> {
    async fn create(
        &self,
        request: tonic::Request<OrganizationCreateRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationCreateResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn get(
        &self,
        request: tonic::Request<OrganizationGetRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationGetResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update(
        &self,
        request: tonic::Request<OrganizationUpdateRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationUpdateResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list(
        &self,
        request: tonic::Request<OrganizationListRequest>,
    ) -> tonic::Result<tonic::Response<OrganizationListResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

