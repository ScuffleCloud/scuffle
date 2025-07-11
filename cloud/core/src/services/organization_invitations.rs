use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::Action;
use crate::common::TxError;
use crate::http_ext::RequestExt;
use crate::models::{OrganizationId, OrganizationInvitation, OrganizationInvitationId, OrganizationMember, UserId};
use crate::schema::{organization_invitations, organization_members, user_emails};
use crate::services::CoreSvc;
use crate::std_ext::ResultExt;
use crate::{CoreConfig, common};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::organization_invitations_service_server::OrganizationInvitationsService
    for CoreSvc<G>
{
    async fn create_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateOrganizationInvitationRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();

        let session = ext.session_or_err()?;

        let organization_id: OrganizationId = payload
            .organization_id
            .parse()
            .into_tonic_internal_err("failed to parse id")?;

        let email = common::normalize_email(&payload.email);

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let invitation = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let invited_user = user_emails::dsl::user_emails
                        .find(&email)
                        .select(user_emails::dsl::user_id)
                        .get_result::<UserId>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query user email")?;

                    let invitation = OrganizationInvitation {
                        id: OrganizationInvitationId::new(),
                        user_id: invited_user,
                        organization_id,
                        email,
                        invited_by_id: session.user_id,
                        expires_at: payload
                            .expires_in_s
                            .map(|s| chrono::Utc::now() + chrono::Duration::seconds(s as i64)),
                    };

                    session.is_authorized(global, session.user_id, Action::CreateOrganizationInvitation, &invitation)?;

                    diesel::insert_into(organization_invitations::dsl::organization_invitations)
                        .values(&invitation)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert organization invitation")?;

                    Ok(invitation)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(invitation.into()))
    }

    async fn list_organization_invitations_by_organization(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::OrganizationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitationList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let organization_id: OrganizationId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        session.is_authorized(
            global,
            session.user_id,
            Action::ListOrganizationInvitationsByOrganization,
            organization_id,
        )?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let invitations = organization_invitations::dsl::organization_invitations
            .filter(organization_invitations::dsl::organization_id.eq(organization_id))
            .load::<OrganizationInvitation>(&mut *db)
            .await
            .into_tonic_internal_err("failed to query organization invitations")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::OrganizationInvitationList {
            invitations: invitations.into_iter().map(Into::into).collect(),
        }))
    }

    async fn list_orgnization_invites_by_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitationList>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let user_id: UserId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        session.is_authorized(global, session.user_id, Action::ListOrganizationInvitationsByUser, user_id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let invitations = organization_invitations::dsl::organization_invitations
            .filter(organization_invitations::dsl::user_id.eq(user_id))
            .load::<OrganizationInvitation>(&mut *db)
            .await
            .into_tonic_internal_err("failed to query organization invitations")?;

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::OrganizationInvitationList {
            invitations: invitations.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::OrganizationInvitationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationInvitation>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let id: OrganizationInvitationId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        session.is_authorized(global, session.user_id, Action::GetOrganizationInvitation, id)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let invitation = organization_invitations::dsl::organization_invitations
            .find(id)
            .get_result::<OrganizationInvitation>(&mut *db)
            .await
            .optional()
            .into_tonic_internal_err("failed to query organization invitations")?
            .ok_or_else(|| {
                tonic::Status::with_error_details(
                    tonic::Code::NotFound,
                    "organization invitation not found",
                    ErrorDetails::new(),
                )
            })?;

        Ok(tonic::Response::new(invitation.into()))
    }

    async fn accept_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::OrganizationInvitationByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::OrganizationMember>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let id: OrganizationInvitationId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let member = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let invitation = diesel::delete(organization_invitations::dsl::organization_invitations)
                        .filter(organization_invitations::dsl::id.eq(id))
                        .returning(OrganizationInvitation::as_select())
                        .get_result::<OrganizationInvitation>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to delete organization invitation")?
                        .ok_or_else(|| {
                            tonic::Status::with_error_details(
                                tonic::Code::NotFound,
                                "organization invitation not found",
                                ErrorDetails::new(),
                            )
                        })?;

                    let Some(user_id) = invitation.user_id else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::FailedPrecondition,
                            "register first to accept this organization invitation",
                            ErrorDetails::new(),
                        )
                        .into());
                    };

                    session.is_authorized(global, session.user_id, Action::ApproveUserSessionRequest, &invitation)?;

                    let organization_member = OrganizationMember {
                        organization_id: invitation.organization_id,
                        user_id,
                        invited_by_id: Some(invitation.invited_by_id),
                        inline_policy: None,
                        created_at: chrono::Utc::now(),
                    };

                    diesel::insert_into(organization_members::dsl::organization_members)
                        .values(&organization_member)
                        .execute(conn)
                        .await
                        .into_tonic_internal_err("failed to insert organization member")?;

                    Ok(organization_member)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(member.into()))
    }

    async fn decline_organization_invitation(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::OrganizationInvitationByIdRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let id: OrganizationInvitationId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        db.transaction::<_, TxError, _>(move |conn| {
            async move {
                let invitation = diesel::delete(organization_invitations::dsl::organization_invitations)
                    .filter(organization_invitations::dsl::id.eq(id))
                    .returning(OrganizationInvitation::as_select())
                    .get_result::<OrganizationInvitation>(conn)
                    .await
                    .optional()
                    .into_tonic_internal_err("failed to delete organization invitation")?
                    .ok_or_else(|| {
                        tonic::Status::with_error_details(
                            tonic::Code::NotFound,
                            "organization invitation not found",
                            ErrorDetails::new(),
                        )
                    })?;

                session.is_authorized(global, session.user_id, Action::DeclineOrganizationInvitation, &invitation)?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(tonic::Response::new(()))
    }
}
