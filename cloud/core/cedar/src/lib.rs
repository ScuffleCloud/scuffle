use std::borrow::Borrow;
use std::collections::HashSet;
use std::str::FromStr;

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

    fn entity_uid(&self) -> cedar_policy::EntityUid {
        self.id().borrow().entity_uid()
    }
}

pub trait CedarIdentifiable {
    /// MUST be a normalized cedar entity type name.
    ///
    /// See [`cedar_policy::EntityTypeName`] and <https://github.com/cedar-policy/rfcs/blob/main/text/0009-disallow-whitespace-in-entityuid.md>.
    const ENTITY_TYPE: &'static str;

    fn entity_id(&self) -> cedar_policy::EntityId;

    fn entity_uid(&self) -> cedar_policy::EntityUid {
        let name = cedar_policy::EntityTypeName::from_str(Self::ENTITY_TYPE).expect("invalid entity type name");
        cedar_policy::EntityUid::from_type_name_and_id(name, self.entity_id())
    }
}

pub trait CedarEntity: CedarIdentifiable + serde::Serialize {
    fn parents(
        &self,
        global: &impl core_traits::Global,
    ) -> impl Future<Output = Result<HashSet<cedar_policy::EntityUid>, tonic::Status>> + Send {
        let _ = global;
        std::future::ready(Ok(HashSet::new()))
    }
}

mod macros;
mod models;
