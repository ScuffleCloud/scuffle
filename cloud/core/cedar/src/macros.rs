macro_rules! impl_cedar_identity {
    ($ty:ident) => {
        impl $crate::CedarEntity for $ty {
            const ENTITY_TYPE: &'static str = stringify!($ty);

            fn entity_id(&self) -> cedar_policy::EntityId {
                cedar_policy::EntityId::new(self.id.to_string_unprefixed())
            }
        }
    };
}

pub(crate) use impl_cedar_identity;
