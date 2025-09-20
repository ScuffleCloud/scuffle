use std::marker::PhantomData;

use serde::ser::SerializeMap;

use crate::{CedarEntity, CedarId};

/// An entity ref is used by entities to refer to another entity.
pub struct EntityUid<E> {
    id: smol_str::SmolStr,
    _marker: PhantomData<E>,
}

impl<E> serde::Serialize for EntityUid<E>
where
    E: CedarEntity,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        map.serialize_entry("type", E::TYPE_NAME.as_str())?;
        map.serialize_entry("id", self.id.as_str())?;

        map.end()
    }
}

impl<E: CedarEntity> EntityUid<E> {
    /// Create an entity uid by providing a id.
    pub fn new(id: impl Into<E::Id>) -> Self {
        Self::new_from_str(id.into().into_smol_string())
    }

    pub(crate) fn new_from_str(id: impl Into<smol_str::SmolStr>) -> Self {
        EntityUid {
            id: id.into(),
            _marker: PhantomData,
        }
    }
}

impl<E: CedarEntity> From<EntityUid<E>> for cedar_policy::EntityUid {
    fn from(value: EntityUid<E>) -> Self {
        Self::from_type_name_and_id(E::entity_type_name().clone(), cedar_policy::EntityId::new(value.id))
    }
}
