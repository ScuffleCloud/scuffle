use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use cedar_policy::{Decision, Entities, PolicySet, Schema};
use core_traits::ResultExt;
use scuffle_cedar_policy::{CedarAction, CedarEntity, Entity};
use tonic_types::{ErrorDetails, StatusExt};

fn static_policies() -> &'static PolicySet {
    const STATIC_POLICIES_STR: &str = include_str!("../static_policies.cedar");
    static STATIC_POLICIES: OnceLock<PolicySet> = OnceLock::new();

    STATIC_POLICIES.get_or_init(|| PolicySet::from_str(STATIC_POLICIES_STR).expect("failed to parse static policies"))
}

fn static_policies_schema() -> &'static Schema {
    const STATIC_POLICIES_SCHEMA_STR: &str = include_str!("../static_policies.cedarschema");
    static STATIC_POLICIES_SCHEMA: OnceLock<Schema> = OnceLock::new();

    STATIC_POLICIES_SCHEMA
        .get_or_init(|| Schema::from_str(STATIC_POLICIES_SCHEMA_STR).expect("failed to parse static policies schema"))
}

pub(crate) async fn is_authorized<P: CedarEntity, R: CedarEntity, A: CedarAction<P, R>>(
    global: &Arc<impl core_traits::Global>,
    action: A,
    principal: impl Into<Entity<P>>,
    resource: impl Into<Entity<R>>,
    ctx: &A::Context,
) -> Result<(), tonic::Status> {
    let schema = static_policies_schema();

    let ctx_json = serde_json::to_value(ctx).into_tonic_internal_err("failed to create cedar context")?;

    let context = cedar_policy::Context::from_json_value(ctx_json, Some((schema, A::action_entity_uid())))
        .into_tonic_internal_err("failed to create cedar context")?;

    let principal = principal.into();
    let resource = resource.into();

    let r = cedar_policy::Request::new(
        principal.entity_uid().into(),
        A::action_entity_uid().clone(),
        resource.entity_uid().into(),
        context,
        Some(schema),
    )
    .into_tonic_internal_err("failed to validate cedar request")?;

    let entities = vec![
        cedar_policy::principal.to_entity(global.as_ref(), Some(schema)).await?,
        resource.to_entity(global.as_ref(), Some(schema)).await?,
    ];

    let entities = Entities::empty()
        .add_entities(entities, Some(schema))
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
