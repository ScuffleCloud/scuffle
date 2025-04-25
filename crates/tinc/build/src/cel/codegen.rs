use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoType {
    Value(ProtoValueType),
    Modified(ProtoModifiedValueType),
}

impl ProtoType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            ProtoType::Value(ty) => ty.can_be_cel(),
            ProtoType::Modified(ty) => ty.can_be_cel(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoValueType {
    String,
    Bytes,
    Int32,
    Int64,
    UInt32,
    UInt64,
    Float,
    Double,
    Bool,
    Timestamp,
    Duration,
    Message(ProtoMessageType),
    Enum(ProtoEnumType),
}

impl ProtoValueType {
    pub fn can_be_cel(&self) -> bool {
        match self {
            ProtoValueType::String | ProtoValueType::Bytes => true,
            ProtoValueType::Int32 | ProtoValueType::Int64 | ProtoValueType::UInt32 | ProtoValueType::UInt64 => true,
            ProtoValueType::Float | ProtoValueType::Double => true,
            ProtoValueType::Bool => true,
            ProtoValueType::Timestamp | ProtoValueType::Duration => true,
            ProtoValueType::Message(_) => false,
            ProtoValueType::Enum(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoEnumType {
    // pub path: syn::Path,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoModifiedValueType {
    Repeated(ProtoValueType),
    Map(ProtoValueType, ProtoValueType),
    Optional(ProtoValueType),
    OneOf(ProtoOneOfType),
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

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoMessageType {
    pub name: String,
    pub fields: BTreeMap<String, ProtoMessageField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoMessageField {
    pub ty: ProtoType,
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoOneOfType {
    pub fields: BTreeMap<String, ProtoValueType>,
}
