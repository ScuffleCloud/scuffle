use crate::services::CoreSvc;

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

    async fn list_organization_members(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationMemberList>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn list_organizations_by_user(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationList>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
