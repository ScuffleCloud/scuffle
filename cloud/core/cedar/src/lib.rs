use std::borrow::Borrow;

use core_traits::ResultExt;
use serde::ser::SerializeMap;

pub trait CedarEntityId: CedarEntity {
    type Id<'a>: CedarIdentifiable
    where
        Self: 'a;

    fn id(&self) -> impl std::borrow::Borrow<Self::Id<'_>>;
}

impl<T: CedarEntityId> CedarIdentifiable for T {
    const ENTITY_TYPE: &'static str = T::Id::ENTITY_TYPE;

    fn entity_id(&self) -> cedar_policy::EntityId {
        self.id().borrow().entity_id()
    }

    fn entity_uid(&self) -> JsonEntityUid {
        self.id().borrow().entity_uid()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonEntityUid {
    pub type_name: &'static str,
    pub id: cedar_policy::EntityId,
}

impl From<JsonEntityUid> for cedar_policy::EntityUid {
    fn from(value: JsonEntityUid) -> Self {
        cedar_policy::EntityUid::from_type_name_and_id(value.type_name.parse().unwrap(), value.id)
    }
}

impl serde::Serialize for JsonEntityUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        map.serialize_entry("type", self.type_name)?;
        map.serialize_entry("id", self.id.unescaped())?;
        map.end()
    }
}

pub trait CedarIdentifiable {
    /// MUST be a normalized cedar entity type name.
    ///
    /// See [`cedar_policy::EntityTypeName`] and <https://github.com/cedar-policy/rfcs/blob/main/text/0009-disallow-whitespace-in-entityuid.md>.
    const ENTITY_TYPE: &'static str;

    fn entity_id(&self) -> cedar_policy::EntityId;

    fn entity_uid(&self) -> JsonEntityUid {
        JsonEntityUid {
            type_name: Self::ENTITY_TYPE,
            id: self.entity_id(),
        }
    }
}

pub trait CedarEntity: CedarIdentifiable + serde::Serialize + Send + Sync {
    fn parents(
        &self,
        global: &impl core_traits::Global,
    ) -> impl Future<Output = Result<impl IntoIterator<Item = JsonEntityUid>, tonic::Status>> + Send {
        let _ = global;
        std::future::ready(Ok([]))
    }

    fn additional_attributes(
        &self,
        global: &impl core_traits::Global,
    ) -> impl Future<Output = Result<impl serde::Serialize, tonic::Status>> + Send {
        let _ = global;
        std::future::ready(Ok(()))
    }

    /// Returns the attributes of the entity as a map.
    /// Also includes additional attributes from [`additional_attributes`](Self::additional_attributes).
    fn attributes(
        &self,
        global: &impl core_traits::Global,
    ) -> impl std::future::Future<Output = Result<impl serde::Serialize, tonic::Status>> + Send {
        #[derive(serde_derive::Serialize)]
        struct Merged<A, B> {
            #[serde(flatten)]
            a: A,
            #[serde(flatten)]
            b: B,
        }

        async move {
            Ok(Merged {
                a: self,
                b: self.additional_attributes(global).await?,
            })
        }
    }

    fn to_entity(
        &self,
        global: &impl core_traits::Global,
    ) -> impl std::future::Future<Output = Result<cedar_policy::Entity, tonic::Status>> + Send {
        async move {
            let value = serde_json::json!({
                "uid": self.entity_uid(),
                "attrs": self.attributes(global).await?,
                "parents": self.parents(global).await?.into_iter().collect::<Vec<_>>(),
            });

            cedar_policy::Entity::from_json_value(value, None).into_tonic_internal_err("failed to create cedar entity")
        }
    }
}

mod macros;
mod models;
