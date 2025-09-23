use std::collections::BTreeMap;

use crate::{CedarChild, CedarEntity, CedarId, EntityTypeName, EntityUid};

#[derive(serde_derive::Serialize)]
struct StaticEntityUid {
    #[serde(rename = "type")]
    ty: EntityTypeName,
    id: smol_str::SmolStr,
}

/// A cedar entity
#[derive(serde_derive::Serialize, bon::Builder)]
#[builder(start_fn(
    name = builder_internal,
    vis = "",
))]
pub struct Entity<E>
where
    E: CedarEntity,
{
    #[builder(start_fn)]
    uid: StaticEntityUid,

    #[builder(start_fn)]
    attrs: E::Attrs,

    #[builder(field)]
    parents: Vec<StaticEntityUid>,
    #[builder(field)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    tags: BTreeMap<smol_str::SmolStr, E::TagType>,
}

impl<E: CedarEntity, S: entity_builder::IsComplete> From<EntityBuilder<E, S>> for Entity<E> {
    fn from(value: EntityBuilder<E, S>) -> Self {
        value.build()
    }
}

impl<E: CedarEntity> Entity<E> {
    /// Create a builder for an entity by providing the entity id as well as the attrs for the entity.
    pub fn builder(id: E::Id, attrs: E::Attrs) -> EntityBuilder<E> {
        Self::builder_internal(
            StaticEntityUid {
                ty: E::TYPE_NAME,
                id: id.into_smol_string(),
            },
            attrs,
        )
    }
}

impl<E: CedarEntity, S: entity_builder::State> EntityBuilder<E, S> {
    /// Add a parent to this entity by providing its EntityId
    pub fn parent<P>(mut self, id: impl Into<P::Id>) -> Self
    where
        E: CedarChild<P>,
        P: CedarEntity,
    {
        self.parents.push(StaticEntityUid {
            ty: P::TYPE_NAME,
            id: id.into().into_smol_string(),
        });
        self
    }

    /// Add a tag to this entity by providing a key and the tag itself.
    pub fn tag(mut self, key: impl Into<smol_str::SmolStr>, value: impl Into<E::TagType>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}

impl<E: CedarEntity> Entity<E> {
    /// Get the [EntityRef] that represents this entity.
    pub fn entity_uid(&self) -> EntityUid<E> {
        EntityUid::new_from_str(self.uid.id.clone())
    }
}
