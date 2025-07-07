use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use cedar_policy::{Decision, Entities, Entity, EntityId, EntityTypeName, EntityUid, PolicySet, RestrictedExpression};
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

pub fn is_authorized<G: CoreConfig>(
    global: &Arc<G>,
    user_session: Option<&UserSession>,
    action: impl CedarEntity,
    resource: impl CedarEntity,
) -> Result<(), tonic::Status> {
    let mut context = serde_json::Map::new();
    if let Some(session) = user_session {
        context.insert(
            "user_session".to_string(),
            serde_json::to_value(session).into_tonic_internal_err("failed to serialize user session")?,
        );
    }

    let context = cedar_policy::Context::from_json_value(serde_json::Value::Object(context), None)
        .into_tonic_internal_err("failed to create cedar context")?;

    let (principal_uid, principal_attrs) = user_session.map_or_else(
        || (UnauthenticatedPrincipal.entity_uid(), UnauthenticatedPrincipal.attributes()),
        |session| (session.user_id.entity_uid(), session.user_id.attributes()),
    );

    let r = cedar_policy::Request::new(
        principal_uid.clone(),
        action.entity_uid(),
        resource.entity_uid(),
        context,
        None,
    )
    .into_tonic_internal_err("failed to validate cedar request")?;

    let entities = vec![
        Entity::new(principal_uid, principal_attrs, HashSet::new())
            .into_tonic_internal_err("failed to create cedar entity")?,
        Entity::new(action.entity_uid(), action.attributes(), HashSet::new())
            .into_tonic_internal_err("failed to create cedar entity")?,
        Entity::new(resource.entity_uid(), resource.attributes(), HashSet::new())
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

pub struct UnauthenticatedPrincipal;

impl CedarEntity for UnauthenticatedPrincipal {
    const ENTITY_TYPE: &'static str = "Unauthenticated";

    fn entity_id(&self) -> EntityId {
        EntityId::new("") // empty id because all unauthenticated prinicipals are the same
    }
}

/// A CRUD action.
pub enum Action {
    Create,
    Update,
    Delete,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Create => write!(f, "create"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
        }
    }
}

impl CedarEntity for Action {
    const ENTITY_TYPE: &'static str = "Action";

    fn entity_id(&self) -> EntityId {
        EntityId::new(self.to_string())
    }
}
