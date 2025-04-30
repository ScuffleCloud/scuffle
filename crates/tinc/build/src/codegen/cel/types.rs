use crate::types::{ProtoModifiedValueType, ProtoType, ProtoValueType, ProtoWellKnownType};

#[derive(Debug, Clone, PartialEq)]
pub enum CelType {
    CelValue,
    Proto(ProtoType),
}

impl CelType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            CelType::CelValue => true,
            CelType::Proto(ty) => ty.can_be_cel(),
        }
    }
}

impl ProtoType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            ProtoType::Value(ty) => ty.can_be_cel(),
            ProtoType::Modified(ty) => ty.can_be_cel(),
        }
    }
}

impl ProtoValueType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            ProtoValueType::String | ProtoValueType::Bytes => true,
            ProtoValueType::Int32 | ProtoValueType::Int64 | ProtoValueType::UInt32 | ProtoValueType::UInt64 => true,
            ProtoValueType::Float | ProtoValueType::Double => true,
            ProtoValueType::Bool => true,
            ProtoValueType::WellKnown(wk) => wk.can_be_cel(),
            ProtoValueType::Message { .. } => false,
            ProtoValueType::Enum { .. } => false,
        }
    }
}

impl ProtoModifiedValueType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            ProtoModifiedValueType::Repeated(ty) => ty.can_be_cel(),
            ProtoModifiedValueType::Map(key, value) => key.can_be_cel() && value.can_be_cel(),
            ProtoModifiedValueType::Optional(ty) => ty.can_be_cel(),
            ProtoModifiedValueType::OneOf(_) => false,
        }
    }
}

impl ProtoWellKnownType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            ProtoWellKnownType::Timestamp | ProtoWellKnownType::Duration => true,
            ProtoWellKnownType::Empty => true,
            ProtoWellKnownType::Struct => true,
            ProtoWellKnownType::Value => true,
            ProtoWellKnownType::List => true,
            ProtoWellKnownType::Any => false,
        }
    }
}
