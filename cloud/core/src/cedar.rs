use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use cedar_policy::{
    Decision, Entities, Entity, EntityAttrEvaluationError, EntityId, EntityTypeName, EntityUid, PolicySet,
    RestrictedExpression,
};
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

pub trait CedarEntity {
    /// MUST be a normalized cedar entity type name.
    ///
    /// See [`cedar_policy::EntityTypeName`] and <https://github.com/cedar-policy/rfcs/blob/main/text/0009-disallow-whitespace-in-entityuid.md>.
    const ENTITY_TYPE: &'static str;

    fn entity_id(&self) -> EntityId;

    fn entity_uid(&self) -> EntityUid {
        let name = EntityTypeName::from_str(Self::ENTITY_TYPE).expect("invalid entity type name");
        EntityUid::from_type_name_and_id(name, self.entity_id())
    }

    fn attributes(&self) -> HashMap<String, RestrictedExpression> {
        HashMap::new()
    }

    fn parents(&self) -> HashSet<EntityUid> {
        HashSet::new()
    }

    fn to_entity(&self) -> Result<Entity, EntityAttrEvaluationError> {
        Entity::new(self.entity_uid(), self.attributes(), self.parents())
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

    fn attributes(&self) -> HashMap<String, RestrictedExpression> {
        [("id".to_string(), RestrictedExpression::new_string(self.to_string()))]
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Clone, Copy, derive_more::Display)]
pub enum Action {
    // User related
    /// Register with email and password.
    #[display("register_with_email_password")]
    RegisterWithEmailPassword,
    /// Register with Google OAuth2.
    #[display("register_with_google")]
    RegisterWithGoogle,
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
    #[display("update_user_password")]
    UpdateUserPassword,
    #[display("update_user_names")]
    UpdateUserNames,
    #[display("update_user_primary_email")]
    UpdateUserPrimaryEmail,
    #[display("list_user_emails")]
    ListUserEmails,
    #[display("create_user_email")]
    CreateUserEmail,
    #[display("delete_user_email")]
    DeleteUserEmail,
    #[display("list_user_webauthn_credentials")]
    ListUserWebauthnCredentials,

    // UserSession related
    #[display("approve_user_session_request")]
    ApproveUserSessionRequest,
    #[display("refresh_user_session")]
    RefreshUserSession,
    #[display("invalidate_user_session")]
    InvalidateUserSession,
}

impl CedarEntity for Action {
    const ENTITY_TYPE: &'static str = "Action";

    fn entity_id(&self) -> EntityId {
        EntityId::new(self.to_string())
    }
}

/// A general resource that is used whenever there is no specific resource for a request. (e.g. user login)
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
