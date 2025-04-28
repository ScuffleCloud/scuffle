use std::collections::BTreeMap;
use std::sync::Arc;

use indexmap::IndexMap;
use prost_reflect::Kind;
use tinc_pb::http_endpoint_options;

use super::cel::{CelExpression, CelInput};
use super::utils::{field_ident_from_str, get_common_import_path, type_ident_from_str};

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoType {
    Value(ProtoValueType),
    Modified(ProtoModifiedValueType),
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
    WellKnown(ProtoWellKnownType),
    Message(ProtoPath),
    Enum(ProtoPath),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoWellKnownType {
    Timestamp,
    Duration,
    Struct,
    Value,
    Empty,
    List,
    Any,
}

impl ProtoValueType {
    pub fn from_pb(ty: &Kind) -> Self {
        match ty {
            Kind::Double => ProtoValueType::Double,
            Kind::Float => ProtoValueType::Float,
            Kind::Int32 => ProtoValueType::Int32,
            Kind::Int64 => ProtoValueType::Int64,
            Kind::Uint32 => ProtoValueType::UInt32,
            Kind::Uint64 => ProtoValueType::UInt64,
            Kind::Sint32 => ProtoValueType::Int32,
            Kind::Sint64 => ProtoValueType::Int64,
            Kind::Fixed32 => ProtoValueType::Float,
            Kind::Fixed64 => ProtoValueType::Double,
            Kind::Sfixed32 => ProtoValueType::Float,
            Kind::Sfixed64 => ProtoValueType::Double,
            Kind::Bool => ProtoValueType::Bool,
            Kind::String => ProtoValueType::String,
            Kind::Bytes => ProtoValueType::Bytes,
            Kind::Message(message) => ProtoValueType::from_proto_path(message.full_name()),
            Kind::Enum(enum_) => ProtoValueType::Enum(ProtoPath::new(enum_.full_name())),
        }
    }

    pub fn from_proto_path(path: &str) -> Self {
        match path {
            "google.protobuf.Timestamp" => ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp),
            "google.protobuf.Duration" => ProtoValueType::WellKnown(ProtoWellKnownType::Duration),
            "google.protobuf.Struct" => ProtoValueType::WellKnown(ProtoWellKnownType::Struct),
            "google.protobuf.Value" => ProtoValueType::WellKnown(ProtoWellKnownType::Value),
            "google.protobuf.Empty" => ProtoValueType::WellKnown(ProtoWellKnownType::Empty),
            "google.protobuf.ListValue" => ProtoValueType::WellKnown(ProtoWellKnownType::List),
            "google.protobuf.Any" => ProtoValueType::WellKnown(ProtoWellKnownType::Any),
            "google.protobuf.BoolValue" => ProtoValueType::Bool,
            "google.protobuf.Int32Value" => ProtoValueType::Int32,
            "google.protobuf.Int64Value" => ProtoValueType::Int64,
            "google.protobuf.UInt32Value" => ProtoValueType::UInt32,
            "google.protobuf.UInt64Value" => ProtoValueType::UInt64,
            "google.protobuf.FloatValue" => ProtoValueType::Float,
            "google.protobuf.DoubleValue" => ProtoValueType::Double,
            "google.protobuf.StringValue" => ProtoValueType::String,
            "google.protobuf.BytesValue" => ProtoValueType::Bytes,
            _ => ProtoValueType::Message(ProtoPath::new(path)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoEnumType {
    pub package: ProtoPath,
    pub full_name: ProtoPath,
    pub options: ProtoEnumOptions,
    pub variants: IndexMap<String, ProtoEnumVariant>,
}

impl ProtoEnumType {
    pub fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoEnumOptions {
    pub repr_enum: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoEnumVariant {
    pub full_name: ProtoPath,
    pub options: ProtoEnumVariantOptions,
    pub value: i32,
}

impl ProtoEnumVariant {
    pub fn rust_ident(&self) -> syn::Ident {
        type_ident_from_str(self.full_name.split('.').next_back().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoEnumVariantOptions {
    pub json_name: String,
    pub visibility: ProtoVisibility,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoModifiedValueType {
    Repeated(ProtoValueType),
    Map(ProtoValueType, ProtoValueType),
    Optional(ProtoValueType),
    OneOf(ProtoOneOfType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoMessageType {
    pub package: ProtoPath,
    pub full_name: ProtoPath,
    pub options: ProtoMessageOptions,
    pub fields: IndexMap<String, ProtoMessageField>,
}

impl ProtoMessageType {
    pub fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProtoMessageOptions {
    pub cel_expressions: Vec<CelExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoMessageField {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub ty: ProtoType,
    pub options: ProtoFieldOptions,
}

impl ProtoMessageField {
    pub fn rust_ident(&self) -> syn::Ident {
        field_ident_from_str(self.full_name.split('.').next_back().unwrap())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProtoFieldJsonOmittable {
    True,
    False,
    TrueButStillSerialize,
}

impl ProtoFieldJsonOmittable {
    pub fn from_pb(value: tinc_pb::JsonOmittable, nullable: bool) -> Self {
        match value {
            tinc_pb::JsonOmittable::Unspecified => {
                if nullable {
                    Self::TrueButStillSerialize
                } else {
                    Self::False
                }
            }
            tinc_pb::JsonOmittable::True => Self::True,
            tinc_pb::JsonOmittable::False => Self::False,
            tinc_pb::JsonOmittable::TrueButStillSerialize => Self::TrueButStillSerialize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtoVisibility {
    Default,
    Skip,
    InputOnly,
    OutputOnly,
}

impl ProtoVisibility {
    pub fn from_pb(visibility: tinc_pb::Visibility) -> Self {
        match visibility {
            tinc_pb::Visibility::Skip => ProtoVisibility::Skip,
            tinc_pb::Visibility::InputOnly => ProtoVisibility::InputOnly,
            tinc_pb::Visibility::OutputOnly => ProtoVisibility::OutputOnly,
            tinc_pb::Visibility::Unspecified => ProtoVisibility::Default,
        }
    }

    pub fn has_output(&self) -> bool {
        matches!(self, ProtoVisibility::OutputOnly | ProtoVisibility::Default)
    }

    pub fn has_input(&self) -> bool {
        matches!(self, ProtoVisibility::InputOnly | ProtoVisibility::Default)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoFieldOptions {
    pub json_name: String,
    pub json_omittable: ProtoFieldJsonOmittable,
    pub nullable: bool,
    pub flatten: bool,
    pub visibility: ProtoVisibility,
    pub cel_exprs: BTreeMap<CelInput, Vec<CelExpression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoOneOfType {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub options: ProtoOneOfOptions,
    pub fields: IndexMap<String, ProtoOneOfField>,
}

impl ProtoOneOfType {
    pub fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoOneOfOptions {
    pub tagged: Option<tinc_pb::schema_oneof_options::Tagged>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoOneOfField {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub ty: ProtoValueType,
    pub options: ProtoFieldOptions,
}

impl ProtoOneOfField {
    pub fn rust_ident(&self) -> syn::Ident {
        type_ident_from_str(self.full_name.split('.').next_back().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct ProtoPath(Arc<str>);

impl ProtoPath {
    pub fn trim_last_segment(&self) -> &str {
        // remove the last .<segment> from the path
        let (item, _) = self.0.rsplit_once('.').unwrap_or_default();
        item
    }
}

impl std::ops::Deref for ProtoPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ProtoPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ProtoPath {
    pub fn new(absolute: impl std::fmt::Display) -> Self {
        Self(absolute.to_string().into())
    }

    pub fn new_field(absolute: impl std::fmt::Display, field: impl std::fmt::Display) -> Self {
        Self(format!("{absolute}.{field}").into())
    }
}

impl PartialEq<&str> for ProtoPath {
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}

impl PartialEq<str> for ProtoPath {
    fn eq(&self, other: &str) -> bool {
        &*self.0 == other
    }
}

impl std::borrow::Borrow<str> for ProtoPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProtoPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoService {
    pub full_name: ProtoPath,
    pub package: ProtoPath,
    pub options: ProtoServiceOptions,
    pub methods: IndexMap<String, ProtoServiceMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoServiceOptions {
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoServiceMethod {
    pub full_name: ProtoPath,
    pub service: ProtoPath,
    pub input: ProtoValueType,
    pub output: ProtoValueType,
    pub endpoints: Vec<ProtoServiceMethodEndpoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoServiceMethodEndpoint {
    pub method: http_endpoint_options::Method,
    pub input: Option<http_endpoint_options::Input>,
}

#[derive(Debug, Clone)]
pub struct ProtoTypeRegistry {
    messages: BTreeMap<ProtoPath, ProtoMessageType>,
    enums: BTreeMap<ProtoPath, ProtoEnumType>,
    services: BTreeMap<ProtoPath, ProtoService>,
}

impl Default for ProtoTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtoTypeRegistry {
    pub fn new() -> Self {
        Self {
            messages: BTreeMap::new(),
            enums: BTreeMap::new(),
            services: BTreeMap::new(),
        }
    }

    pub fn register_message(&mut self, message: ProtoMessageType) {
        self.messages.insert(message.full_name.clone(), message);
    }

    pub fn register_enum(&mut self, enum_: ProtoEnumType) {
        self.enums.insert(enum_.full_name.clone(), enum_);
    }

    pub fn register_service(&mut self, service: ProtoService) {
        self.services.insert(service.full_name.clone(), service);
    }

    pub fn get_message(&self, full_name: &str) -> Option<&ProtoMessageType> {
        self.messages.get(full_name)
    }

    pub fn get_message_mut(&mut self, full_name: &str) -> Option<&mut ProtoMessageType> {
        self.messages.get_mut(full_name)
    }

    pub fn get_enum(&self, full_name: &str) -> Option<&ProtoEnumType> {
        self.enums.get(full_name)
    }

    pub fn get_enum_mut(&mut self, full_name: &str) -> Option<&mut ProtoEnumType> {
        self.enums.get_mut(full_name)
    }

    pub fn get_service(&self, full_name: &str) -> Option<&ProtoService> {
        self.services.get(full_name)
    }

    pub fn get_service_mut(&mut self, full_name: &str) -> Option<&mut ProtoService> {
        self.services.get_mut(full_name)
    }

    pub fn messages(&self) -> impl Iterator<Item = &ProtoMessageType> {
        self.messages.values()
    }

    pub fn enums(&self) -> impl Iterator<Item = &ProtoEnumType> {
        self.enums.values()
    }

    pub fn services(&self) -> impl Iterator<Item = &ProtoService> {
        self.services.values()
    }
}
