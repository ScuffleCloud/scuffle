use serde::ser::SerializeMap;

/// A type used by [crate::CedarEntity::TagType] to indicate that no tag is provided.
// The reason this is an enum is because an enum with no variants is unconstructable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTag {}

impl serde::Serialize for NoTag {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Since this is an empty enum it can never be constructed.
        match *self {}
    }
}

/// A type used by [crate::CedarAction::Context] to indicate that no context is provided.
// This is a struct because we need to be able to construct this.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EmptyContext;

impl serde::Serialize for EmptyContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

/// A type used by [crate::CedarEntity::Attrs] to indicate that the entity has no attributes.
// This is a struct because we need to be able to construct this.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NoAttributes;

impl serde::Serialize for NoAttributes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}
