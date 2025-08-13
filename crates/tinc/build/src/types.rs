use std::collections::BTreeMap;
use std::fmt::Write;
use std::sync::Arc;

use indexmap::IndexMap;
use tinc_pb_prost::http_endpoint_options;

use crate::codegen::cel::{CelExpression, CelExpressions};
use crate::codegen::utils::{field_ident_from_str, get_common_import_path, type_ident_from_str};
use crate::{ExternPaths, Mode};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProtoType {
    Value(ProtoValueType),
    Modified(ProtoModifiedValueType),
}

impl ProtoType {
    pub(crate) fn value_type(&self) -> Option<&ProtoValueType> {
        match self {
            Self::Value(value) => Some(value),
            Self::Modified(modified) => modified.value_type(),
        }
    }

    pub(crate) fn nested(&self) -> bool {
        matches!(
            self,
            Self::Modified(ProtoModifiedValueType::Map(_, _) | ProtoModifiedValueType::Repeated(_))
        )
    }
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

    pub(crate) fn proto_path(&self) -> &str {
        match self {
            ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp) => "google.protobuf.Timestamp",
            ProtoValueType::WellKnown(ProtoWellKnownType::Duration) => "google.protobuf.Duration",
            ProtoValueType::WellKnown(ProtoWellKnownType::Struct) => "google.protobuf.Struct",
            ProtoValueType::WellKnown(ProtoWellKnownType::Value) => "google.protobuf.Value",
            ProtoValueType::WellKnown(ProtoWellKnownType::Empty) => "google.protobuf.Empty",
            ProtoValueType::WellKnown(ProtoWellKnownType::ListValue) => "google.protobuf.ListValue",
            ProtoValueType::WellKnown(ProtoWellKnownType::Any) => "google.protobuf.Any",
            ProtoValueType::Bool => "google.protobuf.BoolValue",
            ProtoValueType::Int32 => "google.protobuf.Int32Value",
            ProtoValueType::Int64 => "google.protobuf.Int64Value",
            ProtoValueType::UInt32 => "google.protobuf.UInt32Value",
            ProtoValueType::UInt64 => "google.protobuf.UInt64Value",
            ProtoValueType::Float => "google.protobuf.FloatValue",
            ProtoValueType::Double => "google.protobuf.DoubleValue",
            ProtoValueType::String => "google.protobuf.StringValue",
            ProtoValueType::Bytes => "google.protobuf.BytesValue",
            ProtoValueType::Enum(path) | ProtoValueType::Message(path) => path.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoEnumType {
    pub package: ProtoPath,
    pub full_name: ProtoPath,
    pub comments: Comments,
    pub options: ProtoEnumOptions,
    pub variants: IndexMap<String, ProtoEnumVariant>,
}

impl ProtoEnumType {
    fn rust_path(&self, package: &str) -> syn::Path {
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
    pub comments: Comments,
    pub options: ProtoEnumVariantOptions,
    pub rust_ident: syn::Ident,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoEnumVariantOptions {
    pub serde_name: String,
    pub visibility: ProtoVisibility,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProtoModifiedValueType {
    Repeated(ProtoValueType),
    Map(ProtoValueType, ProtoValueType),
    Optional(ProtoValueType),
    OneOf(ProtoOneOfType),
}

impl ProtoModifiedValueType {
    pub(crate) fn value_type(&self) -> Option<&ProtoValueType> {
        match self {
            Self::Repeated(v) => Some(v),
            Self::Map(_, v) => Some(v),
            Self::Optional(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoMessageType {
    pub package: ProtoPath,
    pub full_name: ProtoPath,
    pub comments: Comments,
    pub options: ProtoMessageOptions,
    pub fields: IndexMap<String, ProtoMessageField>,
}

impl ProtoMessageType {
    fn rust_path(&self, package: &str) -> syn::Path {
        get_common_import_path(package, &self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ProtoMessageOptions {
    pub cel: Vec<CelExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoMessageField {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub ty: ProtoType,
    pub comments: Comments,
    pub options: ProtoFieldOptions,
}

impl ProtoMessageField {
    pub(crate) fn rust_ident(&self) -> syn::Ident {
        field_ident_from_str(self.full_name.split('.').next_back().unwrap())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ProtoFieldSerdeOmittable {
    True,
    False,
    TrueButStillSerialize,
}

impl ProtoFieldSerdeOmittable {
    pub(crate) fn is_true(&self) -> bool {
        matches!(self, Self::True | Self::TrueButStillSerialize)
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
    pub(crate) fn from_pb(visibility: tinc_pb_prost::Visibility) -> Self {
        match visibility {
            tinc_pb_prost::Visibility::Skip => ProtoVisibility::Skip,
            tinc_pb_prost::Visibility::InputOnly => ProtoVisibility::InputOnly,
            tinc_pb_prost::Visibility::OutputOnly => ProtoVisibility::OutputOnly,
            tinc_pb_prost::Visibility::Unspecified => ProtoVisibility::Default,
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
    pub serde_name: String,
    pub serde_omittable: ProtoFieldSerdeOmittable,
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
    pub tagged: Option<Tagged>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tagged {
    pub tag: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoOneOfField {
    pub full_name: ProtoPath,
    pub message: ProtoPath,
    pub comments: Comments,
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
    pub comments: Comments,
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
    pub comments: Comments,
    pub input: ProtoServiceMethodIo,
    pub output: ProtoServiceMethodIo,
    pub endpoints: Vec<ProtoServiceMethodEndpoint>,
    pub cel: Vec<CelExpression>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Comments {
    pub leading: Option<Arc<str>>,
    pub detached: Arc<[Arc<str>]>,
    pub trailing: Option<Arc<str>>,
}

impl Comments {
    pub(crate) fn is_empty(&self) -> bool {
        self.leading.is_none() && self.detached.is_empty() && self.trailing.is_none()
    }
}

impl std::fmt::Display for Comments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut newline = false;
        if let Some(leading) = self.leading.as_ref() {
            leading.trim().fmt(f)?;
            newline = true;
        }

        for detached in self.detached.iter() {
            if newline {
                f.write_char('\n')?;
            }
            newline = true;
            detached.trim().fmt(f)?;
        }

        if let Some(detached) = self.trailing.as_ref() {
            if newline {
                f.write_char('\n')?;
            }
            detached.trim().fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProtoServiceMethodEndpoint {
    pub method: http_endpoint_options::Method,
    pub request: Option<http_endpoint_options::Request>,
    pub response: Option<http_endpoint_options::Response>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProtoTypeRegistry {
    messages: BTreeMap<ProtoPath, ProtoMessageType>,
    enums: BTreeMap<ProtoPath, ProtoEnumType>,
    services: BTreeMap<ProtoPath, ProtoService>,
    extern_paths: ExternPaths,
    _mode: Mode,
}

impl ProtoTypeRegistry {
    pub(crate) fn new(mode: Mode, extern_paths: ExternPaths) -> Self {
        Self {
            messages: BTreeMap::new(),
            enums: BTreeMap::new(),
            services: BTreeMap::new(),
            extern_paths,
            _mode: mode,
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

    pub(crate) fn resolve_rust_path(&self, package: &str, path: &str) -> Option<syn::Path> {
        self.extern_paths
            .resolve(path)
            .or_else(|| Some(self.enums.get(path)?.rust_path(package)))
            .or_else(|| Some(self.messages.get(path)?.rust_path(package)))
    }

    pub(crate) fn has_extern(&self, path: &str) -> bool {
        self.extern_paths.contains(path)
    }
}
