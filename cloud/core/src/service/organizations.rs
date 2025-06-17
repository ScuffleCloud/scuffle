use crate::service::CoreSvc;

#[async_trait::async_trait]
impl pb::scufflecloud::core::v1::organizations_service_server::OrganizationsService for CoreSvc {
    async fn create_organization(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_organization(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_organization_members(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationMembers>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
