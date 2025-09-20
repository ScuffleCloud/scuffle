use crate::entity_type_name::EntityTypeName;

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
pub trait CedarEntity: serde::Serialize {
    /// Entities can have tags attached to them.
    /// Not all entities have tags and if your entity does not have tag you should use the [crate::NoTag] type.
    type TagType: serde::Serialize;
    /// The id type for this entity.
    type Id: CedarId;

    /// The full qualified cedar name of this entity type.
    const TYPE_NAME: EntityTypeName;

    /// The parsed type name for this entity.
    fn entity_type_name() -> &'static cedar_policy::EntityTypeName;
}

/// A trait defining a relationship between a cedar action its principal and resource.
pub trait CedarAction<Principal, Resource>: CedarActionEntity {
    /// The context used by this action.
    /// Not all actions have contexts and if your action does not have a context you should use the [crate::EmptyContext] type.
    type Context: serde::Serialize;
}

/// A trait which defines a action entity for a specific type.
pub trait CedarActionEntity {
    /// The entity uid for this action.
    fn action_entity_uid() -> &'static cedar_policy::EntityUid;
}

/// A trait defining a relationship between two cedar entities or two cedar actions.
/// This is used to construct entity parents or action groups.
pub trait CedarChild<Parent> {}
