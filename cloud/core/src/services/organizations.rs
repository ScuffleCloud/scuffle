use crate::CoreConfig;
use crate::operations::Operation;
use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::organizations_service_server::OrganizationsService for CoreSvc<G> {
    async fn create_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn get_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetOrganizationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn update_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UpdateOrganizationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_organization_members(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListOrganizationMembersRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationMembersList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_organizations_by_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListOrganizationsByUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationsList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }
}
