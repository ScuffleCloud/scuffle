macro_rules! cedar_entity {
    ($ty:ident) => {
        $crate::macros::cedar_entity_id!($ty);

        impl $crate::CedarEntity for $ty {}
    };
}

pub(crate) use cedar_entity;

macro_rules! cedar_entity_id {
    ($ty:ident) => {
        impl $crate::CedarIdentifiable for ::core_db_types::id::Id<$ty> {
            const ENTITY_TYPE: $crate::EntityTypeName = $crate::entity_type_name!(stringify!($ty));

            fn entity_id(&self) -> cedar_policy::EntityId {
                cedar_policy::EntityId::new(self.unprefixed().to_string())
            }
        }

        impl $crate::CedarEntityId for $ty {
            type Id<'a> = ::core_db_types::id::Id<$ty>;

            #[allow(refining_impl_trait)]
            fn id(&self) -> &Self::Id<'_> {
                &self.id
            }
        }
    };
}

pub(crate) use cedar_entity_id;
