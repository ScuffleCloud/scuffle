use crate::entity_type_name::EntityTypeName;
use crate::{CedarActionRequestError, Entity, EntityUid};

/// A type which can be used as a CedarId
pub trait CedarId {
    /// Convert self into a [smol_str::SmolStr]
    fn into_smol_string(self) -> smol_str::SmolStr;
}

impl CedarId for smol_str::SmolStr {
    fn into_smol_string(self) -> smol_str::SmolStr {
        self
    }
}

impl CedarId for String {
    fn into_smol_string(self) -> smol_str::SmolStr {
        self.into()
    }
}

/// A trait defining an entity.
pub trait CedarEntity {
    /// Entities can have tags attached to them.
    /// Not all entities have tags and if your entity does not have tag you should use the [crate::NoTag] type.
    type TagType: serde::Serialize;
    /// The id type for this entity.
    type Id: CedarId;
    /// The attributes for this entity. Normally this is [`Self`].
    type Attrs: serde::Serialize;

    /// The full qualified cedar name of this entity type.
    const TYPE_NAME: EntityTypeName;

    /// The parsed type name for this entity.
    fn entity_type_name() -> &'static cedar_policy::EntityTypeName;
}

/// A special trait for enum style entities.
pub trait CedarEnumEntity: CedarEntity<Id = Self, Attrs = crate::NoAttributes> {
    /// Convert an enum variant into an entity.
    fn into_entity(self) -> Entity<Self>
    where
        Self: Sized;
}

/// A trait defining a relationship between a cedar action its principal and resource.
pub trait CedarAction<Principal, Resource>: CedarActionEntity {
    /// The context used by this action.
    /// Not all actions have contexts and if your action does not have a context you should use the [crate::EmptyContext] type.
    type Context: serde::Serialize;

    /// Construct a [cedar_policy::Request] from this action.
    fn request(
        principal: EntityUid<Principal>,
        resource: EntityUid<Resource>,
        ctx: &Self::Context,
        schema: Option<&cedar_policy::Schema>,
    ) -> Result<cedar_policy::Request, CedarActionRequestError>
    where
        Principal: CedarEntity,
        Resource: CedarEntity,
    {
        let ctx_json = serde_json::to_value(ctx)?;
        let a_euid = Self::action_entity_uid();
        let context = cedar_policy::Context::from_json_value(ctx_json, schema.map(|s| (s, a_euid)))?;
        Ok(cedar_policy::Request::new(
            principal.into(),
            a_euid.clone(),
            resource.into(),
            context,
            schema,
        )?)
    }
}

/// A trait which defines a action entity for a specific type.
pub trait CedarActionEntity {
    /// The entity uid for this action.
    fn action_entity_uid() -> &'static cedar_policy::EntityUid;
}

/// A trait defining a relationship between two cedar entities or two cedar actions.
/// This is used to construct entity parents or action groups.
pub trait CedarChild<Parent> {}
