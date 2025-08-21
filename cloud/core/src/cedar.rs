use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use cedar_policy::{Decision, Entities, Entity, EntityId, EntityTypeName, EntityUid, PolicySet};
use tonic_types::{ErrorDetails, StatusExt};

use crate::CoreConfig;
use crate::id::{Id, PrefixedId};
use crate::models::UserSession;
use crate::std_ext::ResultExt;

fn static_policies() -> &'static PolicySet {
    const STATIC_POLICIES_STR: &str = include_str!("../static_policies.cedar");
    static STATIC_POLICIES: OnceLock<PolicySet> = OnceLock::new();

    STATIC_POLICIES.get_or_init(|| PolicySet::from_str(STATIC_POLICIES_STR).expect("failed to parse static policies"))
}

fn uid_to_json(uid: EntityUid) -> serde_json::Value {
    serde_json::json!({
        "type": uid.type_name().to_string(),
        "id": uid.id().unescaped(),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CedarEntityError {
    #[error("failed to serialize attributes: {0}")]
    AttributeSerialization(#[from] serde_json::Error),
    #[error("failed to create cedar entity: {0}")]
    Entity(Box<cedar_policy::entities_errors::EntitiesError>),
}

impl From<cedar_policy::entities_errors::EntitiesError> for CedarEntityError {
    fn from(err: cedar_policy::entities_errors::EntitiesError) -> Self {
        CedarEntityError::Entity(Box::new(err))
    }
}

pub trait CedarEntity: serde::Serialize {
    /// MUST be a normalized cedar entity type name.
    ///
    /// See [`cedar_policy::EntityTypeName`] and <https://github.com/cedar-policy/rfcs/blob/main/text/0009-disallow-whitespace-in-entityuid.md>.
    const ENTITY_TYPE: &'static str;

    fn entity_id(&self) -> EntityId;

    fn entity_uid(&self) -> EntityUid {
        let name = EntityTypeName::from_str(Self::ENTITY_TYPE).expect("invalid entity type name");
        EntityUid::from_type_name_and_id(name, self.entity_id())
    }

    fn attributes(&self) -> Result<serde_json::value::Map<String, serde_json::Value>, CedarEntityError> {
        if let serde_json::Value::Object(object) = serde_json::to_value(self)? {
            Ok(object)
        } else {
            Ok(serde_json::value::Map::new())
        }
    }

    fn parents(&self) -> HashSet<EntityUid> {
        HashSet::new()
    }

    fn to_entity(&self) -> Result<Entity, CedarEntityError> {
        let mut value = serde_json::value::Map::new();
        value.insert("uid".to_string(), uid_to_json(self.entity_uid()));
        value.insert("attrs".to_string(), serde_json::Value::Object(self.attributes()?));
        value.insert(
            "parents".to_string(),
            serde_json::Value::Array(self.parents().into_iter().map(uid_to_json).collect()),
        );

        let entity = Entity::from_json_value(serde_json::Value::Object(value), None)?;
        Ok(entity)
    }
}

impl<T: CedarEntity> CedarEntity for &T {
    const ENTITY_TYPE: &'static str = T::ENTITY_TYPE;

    fn entity_id(&self) -> EntityId {
        T::entity_id(self)
    }

    fn entity_uid(&self) -> EntityUid {
        T::entity_uid(self)
    }
}

impl<T: PrefixedId + CedarEntity> CedarEntity for Id<T> {
    const ENTITY_TYPE: &'static str = T::ENTITY_TYPE;

    fn entity_id(&self) -> EntityId {
        EntityId::new(self.to_string_unprefixed())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Unauthenticated;

impl CedarEntity for Unauthenticated {
    const ENTITY_TYPE: &'static str = "Unauthorized";

    fn entity_id(&self) -> EntityId {
        EntityId::new("unauthorized")
    }
}

#[derive(Debug, Clone, Copy, derive_more::Display, serde::Serialize)]
pub enum Action {
    // User related
    /// Register with email and password.
    #[display("register_with_email")]
    RegisterWithEmail,
    #[display("complete_register_with_email")]
    CompleteRegisterWithEmail,
    #[display("get_login_with_email_options")]
    GetLoginWithEmailOptions,
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

impl CedarEntity for Action {
    const ENTITY_TYPE: &'static str = "Action";

    fn entity_id(&self) -> EntityId {
        EntityId::new(self.to_string())
    }
}

/// A general resource that is used whenever there is no specific resource for a request. (e.g. user login)
#[derive(serde::Serialize)]
pub struct CoreApplication;

impl CedarEntity for CoreApplication {
    const ENTITY_TYPE: &'static str = "Application";

    fn entity_id(&self) -> EntityId {
        EntityId::new("core")
    }
}

pub fn is_authorized<G: CoreConfig>(
    global: &Arc<G>,
    user_session: Option<&UserSession>,
    principal: impl CedarEntity,
    action: impl CedarEntity,
    resource: impl CedarEntity,
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
        principal.entity_uid(),
        action.entity_uid(),
        resource.entity_uid(),
        context,
        None,
    )
    .into_tonic_internal_err("failed to validate cedar request")?;

    let entities = vec![
        principal
            .to_entity()
            .into_tonic_internal_err("failed to create cedar entity")?,
        action.to_entity().into_tonic_internal_err("failed to create cedar entity")?,
        resource
            .to_entity()
            .into_tonic_internal_err("failed to create cedar entity")?,
    ];

    let entities = Entities::empty()
        .add_entities(entities, None)
        .into_tonic_internal_err("failed to create cedar entities")?;

    match global.authorizer().is_authorized(&r, static_policies(), &entities).decision() {
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
