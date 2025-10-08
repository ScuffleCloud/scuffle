macro_rules! cedar_entity {
    ($ty:ident, $id:ty) => {
        $crate::macros::cedar_entity_id!($ty, $id);

        impl $crate::CedarEntity for $ty {}
    };
}

pub(crate) use cedar_entity;

macro_rules! cedar_entity_id {
    ($ty:ident, $id:ty) => {
        impl $crate::CedarIdentifiable for $id {
            const ENTITY_TYPE: $crate::EntityTypeName = $crate::entity_type_name!(stringify!($ty));

            fn entity_id(&self) -> cedar_policy::EntityId {
                cedar_policy::EntityId::new(self.ulid().to_string())
            }
        }

        impl $crate::CedarEntityId for $ty {
            type Id<'a> = $id;

            #[allow(refining_impl_trait)]
            fn id(&self) -> &Self::Id<'_> {
                &self.id
            }
        }
    };
}

pub(crate) use cedar_entity_id;
