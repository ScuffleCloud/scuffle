use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use tonic_types::{ErrorDetails, StatusExt};

use crate::CoreConfig;
use crate::cedar::Action;
use crate::http_ext::RequestExt;
use crate::models::{Organization, OrganizationId, OrganizationMember, UserId};
use crate::schema::{organization_members, organizations};
use crate::services::CoreSvc;
use crate::std_ext::ResultExt;

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::organizations_service_server::OrganizationsService for CoreSvc<G> {
    async fn create_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let organization = Organization {
            id: OrganizationId::new(),
            google_customer_id: None,
            google_hosted_domain: None,
            name: payload.name,
            owner_id: session.user_id,
        };

        session.is_authorized(global, session.user_id, Action::CreateOrganization, &organization)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        diesel::insert_into(organizations::dsl::organizations)
            .values(&organization)
            .execute(&mut db)
            .await
            .into_tonic_internal_err("failed to create organization")?;

        Ok(tonic::Response::new(organization.into()))
    }

    async fn get_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::Organization>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let id: OrganizationId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        session.is_authorized(global, session.user_id, Action::GetOrganization, id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let organization = organizations::dsl::organizations
            .find(id)
            .first::<Organization>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to load organization")?
            .ok_or_else(|| {
                tonic::Status::with_error_details(tonic::Code::NotFound, "organization not found", ErrorDetails::new())
            })?;

        Ok(tonic::Response::new(organization.into()))
    }

    async fn list_organization_members(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationMemberList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let id: OrganizationId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        session.is_authorized(global, session.user_id, Action::ListOrganizationMembers, id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let members = organization_members::dsl::organization_members
            .filter(organization_members::dsl::organization_id.eq(id))
            .load::<OrganizationMember>(&mut db)
            .await
            .into_tonic_internal_err("failed to load organization members")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::OrganizationMemberList {
            members: members.into_iter().map(Into::into).collect(),
        }))
    }

    async fn list_organizations_by_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let id: UserId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        session.is_authorized(global, session.user_id, Action::ListOrganizationsByUser, id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let organizations = organization_members::dsl::organization_members
            .filter(organization_members::dsl::user_id.eq(id))
            .inner_join(organizations::dsl::organizations)
            .select(Organization::as_select())
            .load::<Organization>(&mut db)
            .await
            .into_tonic_internal_err("failed to load organizations")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::OrganizationList {
            organizations: organizations.into_iter().map(Into::into).collect(),
        }))
    }
}
