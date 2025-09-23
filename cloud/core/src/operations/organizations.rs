use core_db_types::models::{Organization, OrganizationId, OrganizationMember, Project, ProjectId, User, UserId};
use core_db_types::schema::{organization_members, organizations, projects};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{RequestExt, ResultExt};

use crate::cedar::Action;
use crate::common;
use crate::http_ext::CoreRequestExt;
use crate::operations::{Operation, OperationDriver};

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationRequest> {
    type Principal = User;
    type Resource = Organization;
    type Response = pb::scufflecloud::core::v1::Organization;

    const ACTION: Action = Action::CreateOrganization;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;

        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let session = self.session_or_err()?;

        Ok(Organization {
            id: OrganizationId::new(),
            google_customer_id: None,
            google_hosted_domain: None,
            name: self.get_ref().name.clone(),
            owner_id: session.user_id,
        })
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        diesel::insert_into(organizations::dsl::organizations)
            .values(&resource)
            .execute(&mut db)
            .await
            .into_tonic_internal_err("failed to create organization")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::GetOrganizationRequest> {
    type Principal = User;
    type Resource = Organization;
    type Response = pb::scufflecloud::core::v1::Organization;

    const ACTION: Action = Action::GetOrganization;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let id: OrganizationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_organization_by_id(global, id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::UpdateOrganizationRequest> {
    type Principal = User;
    type Resource = Organization;
    type Response = pb::scufflecloud::core::v1::Organization;

    const ACTION: Action = Action::UpdateOrganization;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let id: OrganizationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        let conn = driver.conn().await?;

        common::get_organization_by_id_in_tx(conn, id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        mut resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let payload = self.into_inner();

        let owner_update_id = payload
            .owner
            .map(|owner| {
                owner
                    .owner_id
                    .parse::<UserId>()
                    .into_tonic_err_with_field_violation("owner_id", "invalid ID")
            })
            .transpose()?;

        let conn = driver.conn().await?;

        if let Some(owner_update_id) = owner_update_id {
            resource = diesel::update(organizations::dsl::organizations)
                .filter(organizations::dsl::id.eq(resource.id))
                .set(organizations::dsl::owner_id.eq(&owner_update_id))
                .returning(Organization::as_returning())
                .get_result::<Organization>(conn)
                .await
                .into_tonic_internal_err("failed to update organization owner")?;
        }

        if let Some(name) = &payload.name {
            resource = diesel::update(organizations::dsl::organizations)
                .filter(organizations::dsl::id.eq(resource.id))
                .set(organizations::dsl::name.eq(&name.name))
                .returning(Organization::as_returning())
                .get_result::<Organization>(conn)
                .await
                .into_tonic_internal_err("failed to update organization name")?;
        }

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListOrganizationMembersRequest> {
    type Principal = User;
    type Resource = Organization;
    type Response = pb::scufflecloud::core::v1::OrganizationMembersList;

    const ACTION: Action = Action::ListOrganizationMembers;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let id: OrganizationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_organization_by_id(global, id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let members = organization_members::dsl::organization_members
            .filter(organization_members::dsl::organization_id.eq(resource.id))
            .load::<OrganizationMember>(&mut db)
            .await
            .into_tonic_internal_err("failed to load organization members")?;

        Ok(pb::scufflecloud::core::v1::OrganizationMembersList {
            members: members.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListOrganizationsByUserRequest> {
    type Principal = User;
    type Resource = User;
    type Response = pb::scufflecloud::core::v1::OrganizationsList;

    const ACTION: Action = Action::ListOrganizationsByUser;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let id: UserId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_user_by_id(global, id).await
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let organizations = organization_members::dsl::organization_members
            .filter(organization_members::dsl::user_id.eq(resource.id))
            .inner_join(organizations::dsl::organizations)
            .select(Organization::as_select())
            .load::<Organization>(&mut db)
            .await
            .into_tonic_internal_err("failed to load organizations")?;

        Ok(pb::scufflecloud::core::v1::OrganizationsList {
            organizations: organizations.into_iter().map(Into::into).collect(),
        })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateProjectRequest> {
    type Principal = User;
    type Resource = Project;
    type Response = pb::scufflecloud::core::v1::Project;

    const ACTION: Action = Action::CreateProject;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let organization_id: OrganizationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        Ok(Project {
            id: ProjectId::new(),
            name: self.get_ref().name.clone(),
            organization_id,
        })
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;

        diesel::insert_into(projects::dsl::projects)
            .values(&resource)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to create project")?;

        Ok(resource.into())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListProjectsRequest> {
    type Principal = User;
    type Resource = Organization;
    type Response = pb::scufflecloud::core::v1::ProjectsList;

    const ACTION: Action = Action::ListProjects;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let id: OrganizationId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        common::get_organization_by_id(global, id).await
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let conn = driver.conn().await?;

        let projects = projects::dsl::projects
            .filter(projects::dsl::organization_id.eq(resource.id))
            .load::<Project>(conn)
            .await
            .into_tonic_internal_err("failed to load projects")?;

        Ok(pb::scufflecloud::core::v1::ProjectsList {
            projects: projects.into_iter().map(Into::into).collect(),
        })
    }
}
