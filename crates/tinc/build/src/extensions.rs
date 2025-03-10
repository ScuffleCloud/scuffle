use std::collections::BTreeMap;

use anyhow::Context;
use prost_reflect::{DescriptorPool, EnumDescriptor, ExtensionDescriptor, MessageDescriptor, ServiceDescriptor};

pub struct Extension<T> {
    name: &'static str,
    descriptor: Option<ExtensionDescriptor>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Extension<T> {
    pub fn new(name: &'static str, pool: &DescriptorPool) -> Self {
        Self {
            name,
            descriptor: pool.get_extension_by_name(name),
            _marker: std::marker::PhantomData,
        }
    }

    fn decode(&self, incoming: &T::Incoming) -> anyhow::Result<Option<T>>
    where
        T: ProstExtension,
    {
        let mut messages = self.decode_all(incoming)?;
        if messages.is_empty() {
            Ok(None)
        } else {
            Ok(Some(messages.swap_remove(0)))
        }
    }

    fn decode_all(&self, incoming: &T::Incoming) -> anyhow::Result<Vec<T>>
    where
        T: ProstExtension,
    {
        let Some(extension) = &self.descriptor else {
            return Ok(Vec::new());
        };

        let descriptor = T::get_options(incoming);
        let Some(descriptor) = descriptor else {
            return Ok(Vec::new());
        };

        let message = descriptor.get_extension(extension);
        match message.as_ref() {
            prost_reflect::Value::Message(message) => {
                if message.fields().next().is_some() {
                    let message = message
                        .transcode_to::<T>()
                        .with_context(|| format!("{} is not a valid {}", self.name, std::any::type_name::<T>()))?;

                    Ok(vec![message])
                } else {
                    Ok(Vec::new())
                }
            }
            prost_reflect::Value::List(list) => list
                .iter()
                .map(|value| {
                    let message = value.as_message().context("expected a message")?;
                    Ok(message.transcode_to::<T>()?)
                })
                .collect(),
            _ => anyhow::bail!("expected a message or list of messages"),
        }
    }
}

trait ProstExtension: prost::Message + Default {
    type Incoming;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage>;
}

impl ProstExtension for tinc_pb::SchemaMessageOptions {
    type Incoming = prost_reflect::MessageDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::SchemaFieldOptions {
    type Incoming = prost_reflect::FieldDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::SchemaEnumOptions {
    type Incoming = prost_reflect::EnumDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::SchemaVariantOptions {
    type Incoming = prost_reflect::EnumValueDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::HttpEndpointOptions {
    type Incoming = prost_reflect::MethodDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::HttpRouterOptions {
    type Incoming = prost_reflect::ServiceDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::SchemaOneofOptions {
    type Incoming = prost_reflect::OneofDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

#[derive(Debug, Clone)]
pub enum FieldKind {
    Primitive(PrimitiveKind),
    Message(String),
    Enum(String),
    List(Box<FieldKind>),
    Map(PrimitiveKind, Box<FieldKind>),
    Optional(Box<FieldKind>),
    WellKnown(WellKnownType),
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveKind {
    Bool,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    String,
    Bytes,
}

impl PrimitiveKind {
    pub fn from_field(field: &prost_reflect::FieldDescriptor) -> Option<Self> {
        match field.kind() {
            prost_reflect::Kind::Double => Some(PrimitiveKind::F64),
            prost_reflect::Kind::Float => Some(PrimitiveKind::F32),
            prost_reflect::Kind::Int32 | prost_reflect::Kind::Sint32 | prost_reflect::Kind::Sfixed32 => {
                Some(PrimitiveKind::I32)
            }
            prost_reflect::Kind::Int64 | prost_reflect::Kind::Sint64 | prost_reflect::Kind::Sfixed64 => {
                Some(PrimitiveKind::I64)
            }
            prost_reflect::Kind::Uint64 | prost_reflect::Kind::Fixed64 => Some(PrimitiveKind::U64),
            prost_reflect::Kind::Uint32 | prost_reflect::Kind::Fixed32 => Some(PrimitiveKind::U32),
            prost_reflect::Kind::Bool => Some(PrimitiveKind::Bool),
            prost_reflect::Kind::String => Some(PrimitiveKind::String),
            prost_reflect::Kind::Bytes => Some(PrimitiveKind::Bytes),
            _ => None,
        }
    }
}

// #[derive(Debug, Clone)]
// pub enum WellKnown {
//     Raw(WellKnownType),
//     Option(Box<WellKnown>),
//     List(Box<WellKnown>),
//     Map(PrimitiveKind, Box<WellKnown>),
// }

fn get_common_import(start: &str, end: &str) -> String {
    let start_parts: Vec<&str> = start.split('.').collect();
    let end_parts: Vec<&str> = end.split('.').collect();

    // Find common prefix length
    let common_len = start_parts.iter().zip(&end_parts).take_while(|(a, b)| a == b).count();

    // Number of `super::` needed
    let num_supers = start_parts.len() - common_len - 1;
    let super_prefix = "super::".repeat(num_supers);

    // Remaining path from the common ancestor
    let relative_path = end_parts[common_len..].join("::");

    // Construct the final result
    format!("{}{}", super_prefix, relative_path)
}

impl FieldKind {
    pub fn serde_with(&self, current_namespace: &str) -> Option<String> {
        match self {
            FieldKind::WellKnown(_) => Some("::tinc::serde_helpers::well_known".to_owned()),
            FieldKind::Optional(inner) => Some(inner.serde_with(current_namespace)?),
            FieldKind::List(inner) => Some(inner.serde_with(current_namespace)?),
            FieldKind::Map(_, inner) => Some(inner.serde_with(current_namespace)?),
            FieldKind::Enum(name) => Some(format!(
                "::tinc::serde_helpers::Enum::<{}>",
                get_common_import(current_namespace, name)
            )),
            FieldKind::Primitive(_) => None,
            FieldKind::Message(_) => None,
        }
    }

    // pub fn schemars_with(&self, current_namespace: &str) -> Option<String> {
    //     match self {
    //         FieldKind::WellKnown(well_known) => Some(format!("::tinc::serde_helpers::well_known::{}", well_known.name())),
    //         FieldKind::Optional(inner) => Some(format!(
    //             "::tinc::serde_helpers::SchemaOptional<{}>",
    //             inner.schemars_with(current_namespace)?
    //         )),
    //         FieldKind::List(inner) => Some(format!(
    //             "::tinc::serde_helpers::SchemaList<{}>",
    //             inner.schemars_with(current_namespace)?
    //         )),
    //         FieldKind::Map(key, inner) => Some(format!(
    //             "::tinc::serde_helpers::SchemaMap<::tinc::serde_helpers::primitive_types::{}, {}>",
    //             match key {
    //                 PrimitiveKind::String => "String",
    //                 PrimitiveKind::Bytes => "Bytes",
    //                 PrimitiveKind::Bool => "Bool",
    //                 PrimitiveKind::I32 => "I32",
    //                 PrimitiveKind::I64 => "I64",
    //                 PrimitiveKind::U32 => "U32",
    //                 PrimitiveKind::U64 => "U64",
    //                 PrimitiveKind::F32 => "F32",
    //                 PrimitiveKind::F64 => "F64",
    //             },
    //             inner.schemars_with(current_namespace)?,
    //         )),
    //         FieldKind::Enum(name) => Some(get_common_import(current_namespace, name)),
    //         FieldKind::Primitive(_) => None,
    //         FieldKind::Message(_) => None,
    //     }
    // }

    pub fn enum_name(&self) -> Option<&str> {
        match self {
            FieldKind::Enum(name) => Some(name),
            FieldKind::List(kind) => kind.enum_name(),
            FieldKind::Map(_, value) => value.enum_name(),
            FieldKind::Optional(kind) => kind.enum_name(),
            _ => None,
        }
    }

    pub fn message_name(&self) -> Option<&str> {
        match self {
            FieldKind::Message(name) => Some(name),
            FieldKind::List(kind) => kind.message_name(),
            FieldKind::Map(_, value) => value.message_name(),
            FieldKind::Optional(kind) => kind.message_name(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WellKnownType {
    // RFC 3339
    Timestamp,
    // Duration (3.0000s)
    Duration,
    // Struct (map<string, any>)
    Struct,
    // Value (any)
    Value,
    // Empty (no fields)
    Empty,
    // List (repeated any)
    List,
}

impl WellKnownType {
    pub fn proto_name(&self) -> &str {
        match self {
            WellKnownType::Timestamp => "google.protobuf.Timestamp",
            WellKnownType::Duration => "google.protobuf.Duration",
            WellKnownType::Struct => "google.protobuf.Struct",
            WellKnownType::Value => "google.protobuf.Value",
            WellKnownType::Empty => "google.protobuf.Empty",
            WellKnownType::List => "google.protobuf.ListValue",
        }
    }

    pub fn from_proto_name(name: &str) -> Option<Self> {
        match name {
            "google.protobuf.Timestamp" => Some(WellKnownType::Timestamp),
            "google.protobuf.Duration" => Some(WellKnownType::Duration),
            "google.protobuf.Struct" => Some(WellKnownType::Struct),
            "google.protobuf.Value" => Some(WellKnownType::Value),
            "google.protobuf.Empty" => Some(WellKnownType::Empty),
            "google.protobuf.ListValue" => Some(WellKnownType::List),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            WellKnownType::Timestamp => "Timestamp",
            WellKnownType::Duration => "Duration",
            WellKnownType::Struct => "Struct",
            WellKnownType::Value => "Value",
            WellKnownType::Empty => "Empty",
            WellKnownType::List => "List",
        }
    }
}

impl FieldKind {
    pub fn from_field(field: &prost_reflect::FieldDescriptor, required: bool) -> anyhow::Result<Self> {
        let (kind, optional) = match field.kind() {
            prost_reflect::Kind::Message(message) if field.is_map() => {
                let key =
                    PrimitiveKind::from_field(&message.map_entry_key_field()).context("map key is not a valid primitive")?;
                let value = Self::from_field(&message.map_entry_value_field(), true).context("map value")?;
                (FieldKind::Map(key, Box::new(value)), field.supports_presence())
            }
            prost_reflect::Kind::Message(message) => match WellKnownType::from_proto_name(message.full_name()) {
                Some(well_known) => (FieldKind::WellKnown(well_known), true),
                None if message.full_name().starts_with("google.protobuf.") => {
                    anyhow::bail!("well-known type not supported: {}", message.full_name());
                }
                _ => (FieldKind::Message(message.full_name().to_owned()), true),
            },
            prost_reflect::Kind::Enum(enum_) => (FieldKind::Enum(enum_.full_name().to_owned()), field.supports_presence()),
            _ => {
                let kind = PrimitiveKind::from_field(field).context("unknown field kind")?;
                (FieldKind::Primitive(kind), field.supports_presence())
            }
        };

        if field.is_list() {
            Ok(FieldKind::List(Box::new(kind)))
        } else if optional && !required {
            Ok(FieldKind::Optional(Box::new(kind)))
        } else {
            Ok(kind)
        }
    }
}

pub struct Extensions {
    // Message extensions
    schema_message: Extension<tinc_pb::SchemaMessageOptions>,
    schema_field: Extension<tinc_pb::SchemaFieldOptions>,
    schema_oneof: Extension<tinc_pb::SchemaOneofOptions>,

    // Enum extensions
    schema_enum: Extension<tinc_pb::SchemaEnumOptions>,
    schema_variant: Extension<tinc_pb::SchemaVariantOptions>,

    // Service extensions
    http_endpoint: Extension<tinc_pb::HttpEndpointOptions>,
    http_router: Extension<tinc_pb::HttpRouterOptions>,

    messages: BTreeMap<String, MessageOpts>,
    enums: BTreeMap<String, EnumOpts>,
    services: BTreeMap<String, ServiceOpts>,
}

#[derive(Default, Debug)]
pub struct MessageOpts {
    pub opts: tinc_pb::SchemaMessageOptions,
    pub fields: BTreeMap<String, FieldOpts>,
    pub oneofs: BTreeMap<String, OneofOpts>,
}

#[derive(Debug)]
pub struct FieldOpts {
    pub kind: FieldKind,
    pub json_name: String,
    pub one_of: Option<String>,
    pub opts: tinc_pb::SchemaFieldOptions,
}

#[derive(Default, Debug)]
pub struct EnumOpts {
    pub opts: tinc_pb::SchemaEnumOptions,
    pub variants: BTreeMap<String, VariantOpts>,
}

#[derive(Default, Debug)]
pub struct VariantOpts {
    pub opts: tinc_pb::SchemaVariantOptions,
}

#[derive(Default, Debug)]
pub struct ServiceOpts {
    pub opts: tinc_pb::HttpRouterOptions,
    pub methods: BTreeMap<String, MethodOpts>,
}

#[derive(Default, Debug)]
pub struct MethodOpts {
    pub opts: Vec<tinc_pb::HttpEndpointOptions>,
    pub input: String,
    pub output: String,
}

#[derive(Default, Debug)]
pub struct OneofOpts {
    pub opts: tinc_pb::SchemaOneofOptions,
}

impl Extensions {
    pub fn new(pool: &DescriptorPool) -> Self {
        Self {
            schema_message: Extension::new("tinc.schema_message", pool),
            schema_field: Extension::new("tinc.schema_field", pool),
            schema_enum: Extension::new("tinc.schema_enum", pool),
            schema_variant: Extension::new("tinc.schema_variant", pool),
            http_endpoint: Extension::new("tinc.http_endpoint", pool),
            http_router: Extension::new("tinc.http_router", pool),
            schema_oneof: Extension::new("tinc.schema_oneof", pool),
            messages: BTreeMap::new(),
            enums: BTreeMap::new(),
            services: BTreeMap::new(),
        }
    }

    pub fn messages(&self) -> &BTreeMap<String, MessageOpts> {
        &self.messages
    }

    pub fn enums(&self) -> &BTreeMap<String, EnumOpts> {
        &self.enums
    }

    pub fn services(&self) -> &BTreeMap<String, ServiceOpts> {
        &self.services
    }

    pub fn process(&mut self, pool: &DescriptorPool) -> anyhow::Result<()> {
        for service in pool.services() {
            self.process_service(pool, &service, false)
                .with_context(|| service.full_name().to_owned())?;
        }

        for message in pool.all_messages() {
            self.process_message(pool, &message, false)
                .with_context(|| message.full_name().to_owned())?;
        }

        for enum_ in pool.all_enums() {
            self.process_enum(pool, &enum_, false)
                .with_context(|| enum_.full_name().to_owned())?;
        }

        Ok(())
    }

    fn process_service(
        &mut self,
        pool: &DescriptorPool,
        service: &ServiceDescriptor,
        mut insert: bool,
    ) -> anyhow::Result<()> {
        if self.services.contains_key(service.full_name()) {
            return Ok(());
        }

        let opts = self.http_router.decode(service)?;
        insert = insert || opts.is_some();

        let mut service_opts = ServiceOpts {
            opts: opts.unwrap_or_default(),
            methods: BTreeMap::new(),
        };

        for method in service.methods() {
            let opts = self
                .http_endpoint
                .decode_all(&method)
                .with_context(|| method.name().to_owned())?;

            insert = insert || !opts.is_empty();

            let input = method.input();
            let output = method.output();

            service_opts.methods.insert(
                method.name().to_owned(),
                MethodOpts {
                    opts,
                    input: input.full_name().to_owned(),
                    output: output.full_name().to_owned(),
                },
            );

            for message in [input, output] {
                self.process_message(pool, &message, true)
                    .with_context(|| method.name().to_owned())
                    .with_context(|| message.full_name().to_owned())?;
            }
        }

        if insert {
            self.services.insert(service.full_name().to_owned(), service_opts);
        }

        Ok(())
    }

    fn process_message(&mut self, pool: &DescriptorPool, message: &MessageDescriptor, insert: bool) -> anyhow::Result<()> {
        if self.messages.contains_key(message.full_name()) {
            return Ok(());
        }

        let opts = self.schema_message.decode(message)?;

        let fields = message
            .fields()
            .map(|field| {
                let opts = self
                    .schema_field
                    .decode(&field)
                    .with_context(|| field.full_name().to_owned())?;
                Ok((field, opts))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let oneofs = fields
            .iter()
            .filter_map(|(field, _)| field.containing_oneof())
            .map(|oneof| {
                let opts = self.schema_oneof.decode(&oneof)?;
                Ok((oneof, opts))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        if !insert
            && opts.is_none()
            && fields.iter().all(|(_, opts)| opts.is_none())
            && oneofs.iter().all(|(_, opts)| opts.is_none())
        {
            return Ok(());
        }

        self.messages.insert(
            message.full_name().to_owned(),
            MessageOpts {
                opts: opts.unwrap_or_default(),
                fields: BTreeMap::new(),
                oneofs: BTreeMap::new(),
            },
        );

        for (field, opts) in fields {
            let opts = opts.unwrap_or_default();

            let kind = FieldKind::from_field(&field, opts.required || field.containing_oneof().is_some())
                .with_context(|| field.full_name().to_owned())?;
            self.messages.get_mut(message.full_name()).unwrap().fields.insert(
                field.name().to_owned(),
                FieldOpts {
                    kind: kind.clone(),
                    one_of: field.containing_oneof().map(|f| f.name().to_owned()),
                    json_name: field.json_name().to_owned(),
                    opts,
                },
            );

            if let Some(name) = kind.message_name() {
                self.process_message(pool, &pool.get_message_by_name(name).unwrap(), true)
                    .with_context(|| field.full_name().to_owned())
                    .with_context(|| name.to_owned())?;
            } else if let Some(name) = kind.enum_name() {
                self.process_enum(pool, &pool.get_enum_by_name(name).unwrap(), true)
                    .with_context(|| field.full_name().to_owned())
                    .with_context(|| name.to_owned())?;
            }
        }

        for (oneof, opts) in oneofs {
            self.messages.get_mut(message.full_name()).unwrap().oneofs.insert(
                oneof.name().to_owned(),
                OneofOpts {
                    opts: opts.unwrap_or_default(),
                },
            );
        }

        Ok(())
    }

    fn process_enum(&mut self, pool: &DescriptorPool, enum_: &EnumDescriptor, insert: bool) -> anyhow::Result<()> {
        if self.enums.contains_key(enum_.full_name()) {
            return Ok(());
        }

        let opts = self.schema_enum.decode(enum_)?;

        let values = enum_
            .values()
            .map(|value| {
                let opts = self
                    .schema_variant
                    .decode(&value)
                    .with_context(|| value.full_name().to_owned())?;
                Ok((value, opts))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        if !insert && opts.is_none() && values.iter().all(|(_, opts)| opts.is_none()) {
            return Ok(());
        }

        self.enums.insert(
            enum_.full_name().to_owned(),
            EnumOpts {
                opts: opts.unwrap_or_default(),
                variants: BTreeMap::new(),
            },
        );

        let enum_opts = self.enums.get_mut(enum_.full_name()).unwrap();

        for (variant, opts) in values {
            enum_opts.variants.insert(
                variant.name().to_owned(),
                VariantOpts {
                    opts: opts.unwrap_or_default(),
                },
            );
        }

        Ok(())
    }
}
