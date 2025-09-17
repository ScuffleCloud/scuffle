use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use cedar_policy::{Decision, Entities, EntityId, PolicySet};
use core_cedar::{CedarEntity, CedarIdentifiable, EntityTypeName, entity_type_name};
use core_db_types::models::UserSession;
use core_traits::ResultExt;
use tonic_types::{ErrorDetails, StatusExt};

fn static_policies() -> &'static PolicySet {
    const STATIC_POLICIES_STR: &str = include_str!("../static_policies.cedar");
    static STATIC_POLICIES: OnceLock<PolicySet> = OnceLock::new();

    STATIC_POLICIES.get_or_init(|| PolicySet::from_str(STATIC_POLICIES_STR).expect("failed to parse static policies"))
}

#[derive(Debug, serde::Serialize)]
pub struct Unauthenticated;

impl CedarIdentifiable for Unauthenticated {
    const ENTITY_TYPE: EntityTypeName = entity_type_name!("Unauthenticated");

    fn entity_id(&self) -> EntityId {
        EntityId::new("unauthenticated")
    }
}

impl CedarEntity for Unauthenticated {}

#[derive(Debug, Clone, Copy, derive_more::Display, serde::Serialize)]
pub enum Action {
    /// Login to an existing account with email and password.
    #[display("login_with_email_password")]
    LoginWithEmailPassword,
    #[display("request_magic_link")]
    RequestMagicLink,
    #[display("login_with_magic_link")]
    LoginWithMagicLink,
    /// Login to an existing account with Google OAuth2.
    #[display("login_with_google")]
    LoginWithGoogle,
    #[display("login_with_webauthn")]
    LoginWithWebauthn,
    #[display("get_user")]
    GetUser,
    #[display("update_user")]
    UpdateUser,
    #[display("list_user_emails")]
    ListUserEmails,
    #[display("create_user_email")]
    CreateUserEmail,
    #[display("delete_user_email")]
    DeleteUserEmail,

    #[display("create_webauthn_credential")]
    CreateWebauthnCredential,
    #[display("complete_create_webauthn_credential")]
    CompleteCreateWebauthnCredential,
    #[display("create_webauthn_challenge")]
    CreateWebauthnChallenge,
    #[display("delete_webauthn_credential")]
    DeleteWebauthnCredential,
    #[display("list_webauthn_credentials")]
    ListWebauthnCredentials,

    #[display("create_totp_credential")]
    CreateTotpCredential,
    #[display("complete_create_totp_credential")]
    CompleteCreateTotpCredential,
    #[display("delete_totp_credential")]
    DeleteTotpCredential,
    #[display("list_totp_credentials")]
    ListTotpCredentials,

    #[display("regenerate_recovery_codes")]
    RegenerateRecoveryCodes,
    #[display("delete_user")]
    DeleteUser,

    // UserSessionRequest related
    #[display("create_user_session_request")]
    CreateUserSessionRequest,
    #[display("get_user_session_request")]
    GetUserSessionRequest,
    #[display("approve_user_session_request")]
    ApproveUserSessionRequest,
    #[display("complete_user_session_request")]
    CompleteUserSessionRequest,

    // UserSession related
    #[display("validate_mfa_for_user_session")]
    ValidateMfaForUserSession,
    #[display("refresh_user_session")]
    RefreshUserSession,
    #[display("invalidate_user_session")]
    InvalidateUserSession,

    // Organization related
    #[display("create_organization")]
    CreateOrganization,
    #[display("get_organization")]
    GetOrganization,
    #[display("update_organization")]
    UpdateOrganization,
    #[display("list_organization_members")]
    ListOrganizationMembers,
    #[display("list_organizations_by_user")]
    ListOrganizationsByUser,
    #[display("create_project")]
    CreateProject,
    #[display("list_projects")]
    ListProjects,

    // OrganizationInvitation related
    #[display("create_organization_invitation")]
    CreateOrganizationInvitation,
    #[display("list_organization_invitations_by_user")]
    ListOrganizationInvitationsByUser,
    #[display("list_organization_invitations_by_organization")]
    ListOrganizationInvitationsByOrganization,
    #[display("get_organization_invitation")]
    GetOrganizationInvitation,
    #[display("accept_organization_invitation")]
    AcceptOrganizationInvitation,
    #[display("decline_organization_invitation")]
    DeclineOrganizationInvitation,
}

impl CedarIdentifiable for Action {
    const ENTITY_TYPE: EntityTypeName = entity_type_name!("Action");

    fn entity_id(&self) -> EntityId {
        EntityId::new(self.to_string())
    }
}

impl CedarEntity for Action {}

/// A general resource that is used whenever there is no specific resource for a request. (e.g. user login)
#[derive(serde::Serialize)]
pub struct CoreApplication;

impl CedarIdentifiable for CoreApplication {
    const ENTITY_TYPE: EntityTypeName = entity_type_name!("Application");

    fn entity_id(&self) -> EntityId {
        EntityId::new("core")
    }
}

impl CedarEntity for CoreApplication {}

pub(crate) async fn is_authorized<G: core_traits::Global>(
    global: &Arc<G>,
    user_session: Option<&UserSession>,
    principal: &impl CedarEntity,
    action: &impl CedarEntity,
    resource: &impl CedarEntity,
) -> Result<(), tonic::Status> {
    let mut context = serde_json::Map::new();
    if let Some(session) = user_session {
        context.insert(
            "user_session_mfa_pending".to_string(),
            serde_json::Value::Bool(session.mfa_pending),
        );
    }

    let context = cedar_policy::Context::from_json_value(serde_json::Value::Object(context), None)
        .into_tonic_internal_err("failed to create cedar context")?;

    let r = cedar_policy::Request::new(
        principal.entity_uid().into(),
        action.entity_uid().into(),
        resource.entity_uid().into(),
        context,
        None,
    )
    .into_tonic_internal_err("failed to validate cedar request")?;

    let entities = vec![
        principal.to_entity(global.as_ref()).await?,
        action.to_entity(global.as_ref()).await?,
        resource.to_entity(global.as_ref()).await?,
    ];

    let entities = Entities::empty()
        .add_entities(entities, None)
        .into_tonic_internal_err("failed to create cedar entities")?;

    match cedar_policy::Authorizer::new()
        .is_authorized(&r, static_policies(), &entities)
        .decision()
    {
        Decision::Allow => Ok(()),
        Decision::Deny => {
            tracing::warn!(request = ?r, "authorization denied");
            let message = format!(
                "{} is not authorized to perform {} on {}",
                r.principal().expect("is always known"),
                r.action().expect("is always known"),
                r.resource().expect("is always known")
            );

            Err(tonic::Status::with_error_details(
                tonic::Code::PermissionDenied,
                "you are not authorized to perform this action",
                ErrorDetails::with_debug_info(vec![], message),
            ))
        }
    }
}
