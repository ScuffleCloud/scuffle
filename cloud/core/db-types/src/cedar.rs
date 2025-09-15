use std::str::FromStr;

pub trait CedarEntity: serde::Serialize {
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
