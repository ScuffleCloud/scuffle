use pb::scufflecloud::core::v1::organizations_service_server::OrganizationsService;
use pb::scufflecloud::core::v1::{
    CreateOrganizationRequest, CreateProjectRequest, GetOrganizationRequest, ListOrganizationMembersRequest,
    ListOrganizationsByUserRequest, ListProjectsRequest, Organization, OrganizationMembersList, OrganizationsList, Project,
    ProjectsList, UpdateOrganizationRequest,
};

use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: core_traits::Global> OrganizationsService for CoreSvc<G> {
    async fn create_organization(
        &self,
        req: tonic::Request<CreateOrganizationRequest>,
    ) -> Result<tonic::Response<Organization>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn get_organization(
        &self,
        req: tonic::Request<GetOrganizationRequest>,
    ) -> Result<tonic::Response<Organization>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update_organization(
        &self,
        req: tonic::Request<UpdateOrganizationRequest>,
    ) -> Result<tonic::Response<Organization>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_organization_members(
        &self,
        req: tonic::Request<ListOrganizationMembersRequest>,
    ) -> Result<tonic::Response<OrganizationMembersList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_organizations_by_user(
        &self,
        req: tonic::Request<ListOrganizationsByUserRequest>,
    ) -> Result<tonic::Response<OrganizationsList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_project(
        &self,
        req: tonic::Request<CreateProjectRequest>,
    ) -> Result<tonic::Response<Project>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_projects(
        &self,
        req: tonic::Request<ListProjectsRequest>,
    ) -> Result<tonic::Response<ProjectsList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
