use std::collections::BTreeMap;
use std::sync::Arc;

use indexmap::IndexMap;
#[cfg(feature = "prost")]
use syn::parse_quote;
use tinc_pb::http_endpoint_options;

use crate::Mode;
use crate::codegen::cel::CelExpressions;
use crate::codegen::utils::{field_ident_from_str, get_common_import_path, type_ident_from_str};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProtoType {
    Value(ProtoValueType),
    Modified(ProtoModifiedValueType),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProtoValueType {
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
pub(crate) enum ProtoWellKnownType {
    Timestamp,
    Duration,
    Struct,
    Value,
    Empty,
    ListValue,
    Any,
}

impl ProtoValueType {
    #[cfg(feature = "prost")]
    pub(crate) fn from_pb(ty: &prost_reflect::Kind) -> Self {
        match ty {
            prost_reflect::Kind::Double => ProtoValueType::Double,
            prost_reflect::Kind::Float => ProtoValueType::Float,
            prost_reflect::Kind::Int32 => ProtoValueType::Int32,
            prost_reflect::Kind::Int64 => ProtoValueType::Int64,
            prost_reflect::Kind::Uint32 => ProtoValueType::UInt32,
            prost_reflect::Kind::Uint64 => ProtoValueType::UInt64,
            prost_reflect::Kind::Sint32 => ProtoValueType::Int32,
            prost_reflect::Kind::Sint64 => ProtoValueType::Int64,
            prost_reflect::Kind::Fixed32 => ProtoValueType::Float,
            prost_reflect::Kind::Fixed64 => ProtoValueType::Double,
            prost_reflect::Kind::Sfixed32 => ProtoValueType::Float,
            prost_reflect::Kind::Sfixed64 => ProtoValueType::Double,
            prost_reflect::Kind::Bool => ProtoValueType::Bool,
            prost_reflect::Kind::String => ProtoValueType::String,
            prost_reflect::Kind::Bytes => ProtoValueType::Bytes,
            prost_reflect::Kind::Message(message) => ProtoValueType::from_proto_path(message.full_name()),
            prost_reflect::Kind::Enum(enum_) => ProtoValueType::Enum(ProtoPath::new(enum_.full_name())),
        }
    }

    pub(crate) fn from_proto_path(path: &str) -> Self {
        match path {
            "google.protobuf.Timestamp" => ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp),
            "google.protobuf.Duration" => ProtoValueType::WellKnown(ProtoWellKnownType::Duration),
            "google.protobuf.Struct" => ProtoValueType::WellKnown(ProtoWellKnownType::Struct),
            "google.protobuf.Value" => ProtoValueType::WellKnown(ProtoWellKnownType::Value),
            "google.protobuf.Empty" => ProtoValueType::WellKnown(ProtoWellKnownType::Empty),
            "google.protobuf.ListValue" => ProtoValueType::WellKnown(ProtoWellKnownType::ListValue),
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

    pub(crate) fn rust_path(&self, package: &str, mode: Mode) -> syn::Path {
        match (self, mode) {
            (ProtoValueType::Enum(name), _) => get_common_import_path(package, name),
            (ProtoValueType::Message(name), _) => get_common_import_path(package, name),
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::Timestamp)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::Duration), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::Duration)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::Struct), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::Struct)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::Value), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::Value)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::Empty), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::Empty)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::ListValue), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::List)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::WellKnown(ProtoWellKnownType::Any), Mode::Prost) => {
                parse_quote!(::tinc::well_known::prost::Any)
            }
            #[cfg(feature = "prost")]
            (ProtoValueType::Bool, Mode::Prost) => parse_quote!(::tinc::well_known::prost::BoolValue),
            #[cfg(feature = "prost")]
            (ProtoValueType::Int32, Mode::Prost) => parse_quote!(::tinc::well_known::prost::Int32Value),
            #[cfg(feature = "prost")]
            (ProtoValueType::Int64, Mode::Prost) => parse_quote!(::tinc::well_known::prost::Int64Value),
            #[cfg(feature = "prost")]
            (ProtoValueType::UInt32, Mode::Prost) => parse_quote!(::tinc::well_known::prost::UInt32Value),
            #[cfg(feature = "prost")]
            (ProtoValueType::UInt64, Mode::Prost) => parse_quote!(::tinc::well_known::prost::UInt64Value),
            #[cfg(feature = "prost")]
            (ProtoValueType::Float, Mode::Prost) => parse_quote!(::tinc::well_known::prost::FloatValue),
            #[cfg(feature = "prost")]
            (ProtoValueType::Double, Mode::Prost) => parse_quote!(::tinc::well_known::prost::DoubleValue),
            #[cfg(feature = "prost")]
            (ProtoValueType::String, Mode::Prost) => parse_quote!(::tinc::well_known::prost::String),
            #[cfg(feature = "prost")]
            (ProtoValueType::Bytes, Mode::Prost) => parse_quote!(::tinc::well_known::prost::Bytes),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoEnumType {
    pub package: ProtoPath,
    pub full_name: ProtoPath,
    pub options: ProtoEnumOptions,
    pub variants: IndexMap<String, ProtoEnumVariant>,
}

impl ProtoEnumType {
    pub(crate) fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoEnumOptions {
    pub repr_enum: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoEnumVariant {
    pub full_name: ProtoPath,
    pub options: ProtoEnumVariantOptions,
    pub rust_ident: syn::Ident,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoEnumVariantOptions {
    pub json_name: String,
    pub visibility: ProtoVisibility,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProtoModifiedValueType {
    Repeated(ProtoValueType),
    Map(ProtoValueType, ProtoValueType),
    Optional(ProtoValueType),
    OneOf(ProtoOneOfType),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoMessageType {
    pub package: ProtoPath,
    pub full_name: ProtoPath,
    pub options: ProtoMessageOptions,
    pub fields: IndexMap<String, ProtoMessageField>,
}

impl ProtoMessageType {
    pub(crate) fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ProtoMessageOptions {
    pub cel: Vec<tinc_pb::CelExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoMessageField {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub ty: ProtoType,
    pub options: ProtoFieldOptions,
}

impl ProtoMessageField {
    pub(crate) fn rust_ident(&self) -> syn::Ident {
        field_ident_from_str(self.full_name.split('.').next_back().unwrap())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ProtoFieldJsonOmittable {
    True,
    False,
    TrueButStillSerialize,
}

impl ProtoFieldJsonOmittable {
    pub(crate) fn from_pb(value: tinc_pb::JsonOmittable, nullable: bool) -> Self {
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
pub(crate) enum ProtoVisibility {
    Default,
    Skip,
    InputOnly,
    OutputOnly,
}

impl ProtoVisibility {
    pub(crate) fn from_pb(visibility: tinc_pb::Visibility) -> Self {
        match visibility {
            tinc_pb::Visibility::Skip => ProtoVisibility::Skip,
            tinc_pb::Visibility::InputOnly => ProtoVisibility::InputOnly,
            tinc_pb::Visibility::OutputOnly => ProtoVisibility::OutputOnly,
            tinc_pb::Visibility::Unspecified => ProtoVisibility::Default,
        }
    }

    pub(crate) fn has_output(&self) -> bool {
        matches!(self, ProtoVisibility::OutputOnly | ProtoVisibility::Default)
    }

    pub(crate) fn has_input(&self) -> bool {
        matches!(self, ProtoVisibility::InputOnly | ProtoVisibility::Default)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoFieldOptions {
    pub json_name: String,
    pub json_omittable: ProtoFieldJsonOmittable,
    pub nullable: bool,
    pub flatten: bool,
    pub visibility: ProtoVisibility,
    pub cel_exprs: CelExpressions,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoOneOfType {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub options: ProtoOneOfOptions,
    pub fields: IndexMap<String, ProtoOneOfField>,
}

impl ProtoOneOfType {
    pub(crate) fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoOneOfOptions {
    pub tagged: Option<tinc_pb::oneof_options::Tagged>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoOneOfField {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub ty: ProtoValueType,
    pub options: ProtoFieldOptions,
}

impl ProtoOneOfField {
    pub(crate) fn rust_ident(&self) -> syn::Ident {
        type_ident_from_str(self.full_name.split('.').next_back().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct ProtoPath(pub Arc<str>);

impl ProtoPath {
    pub(crate) fn trim_last_segment(&self) -> &str {
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
    pub(crate) fn new(absolute: impl std::fmt::Display) -> Self {
        Self(absolute.to_string().into())
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
pub(crate) struct ProtoService {
    pub full_name: ProtoPath,
    pub package: ProtoPath,
    pub options: ProtoServiceOptions,
    pub methods: IndexMap<String, ProtoServiceMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoServiceOptions {
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProtoServiceMethodIo {
    Single(ProtoValueType),
    Stream(ProtoValueType),
}

impl ProtoServiceMethodIo {
    pub(crate) fn is_stream(&self) -> bool {
        matches!(self, ProtoServiceMethodIo::Stream(_))
    }

    pub(crate) fn value_type(&self) -> &ProtoValueType {
        match self {
            ProtoServiceMethodIo::Single(ty) => ty,
            ProtoServiceMethodIo::Stream(ty) => ty,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoServiceMethod {
    pub full_name: ProtoPath,
    pub service: ProtoPath,
    pub input: ProtoServiceMethodIo,
    pub output: ProtoServiceMethodIo,
    pub endpoints: Vec<ProtoServiceMethodEndpoint>,
    pub cel: Vec<tinc_pb::CelExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoServiceMethodEndpoint {
    pub method: http_endpoint_options::Method,
    pub input: Option<http_endpoint_options::Input>,
    pub response: Option<http_endpoint_options::Response>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProtoTypeRegistry {
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
    pub(crate) fn new() -> Self {
        Self {
            messages: BTreeMap::new(),
            enums: BTreeMap::new(),
            services: BTreeMap::new(),
        }
    }

    pub(crate) fn register_message(&mut self, message: ProtoMessageType) {
        self.messages.insert(message.full_name.clone(), message);
    }

    pub(crate) fn register_enum(&mut self, enum_: ProtoEnumType) {
        self.enums.insert(enum_.full_name.clone(), enum_);
    }

    pub(crate) fn register_service(&mut self, service: ProtoService) {
        self.services.insert(service.full_name.clone(), service);
    }

    pub(crate) fn get_message(&self, full_name: &str) -> Option<&ProtoMessageType> {
        self.messages.get(full_name)
    }

    pub(crate) fn get_message_mut(&mut self, full_name: &str) -> Option<&mut ProtoMessageType> {
        self.messages.get_mut(full_name)
    }

    pub(crate) fn get_enum(&self, full_name: &str) -> Option<&ProtoEnumType> {
        self.enums.get(full_name)
    }

    pub(crate) fn get_service(&self, full_name: &str) -> Option<&ProtoService> {
        self.services.get(full_name)
    }

    pub(crate) fn messages(&self) -> impl Iterator<Item = &ProtoMessageType> {
        self.messages.values()
    }

    pub(crate) fn enums(&self) -> impl Iterator<Item = &ProtoEnumType> {
        self.enums.values()
    }

    pub(crate) fn services(&self) -> impl Iterator<Item = &ProtoService> {
        self.services.values()
    }
}
