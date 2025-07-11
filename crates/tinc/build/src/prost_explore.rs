use std::collections::BTreeMap;

use anyhow::Context;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use prost_reflect::prost_types::source_code_info::Location;
use prost_reflect::{
    DescriptorPool, EnumDescriptor, ExtensionDescriptor, FileDescriptor, Kind, MessageDescriptor, ServiceDescriptor,
};
use quote::format_ident;
use tinc_cel::{CelEnum, CelValueConv};

use crate::codegen::cel::{CelExpression, CelExpressions};
use crate::codegen::prost_sanatize::{strip_enum_prefix, to_upper_camel};
use crate::types::{
    Comments, ProtoEnumOptions, ProtoEnumType, ProtoEnumVariant, ProtoEnumVariantOptions, ProtoFieldOptions,
    ProtoFieldSerdeOmittable, ProtoMessageField, ProtoMessageOptions, ProtoMessageType, ProtoModifiedValueType,
    ProtoOneOfField, ProtoOneOfOptions, ProtoOneOfType, ProtoPath, ProtoService, ProtoServiceMethod,
    ProtoServiceMethodEndpoint, ProtoServiceMethodIo, ProtoServiceOptions, ProtoType, ProtoTypeRegistry, ProtoValueType,
    ProtoVisibility, Tagged,
};

pub(crate) struct Extension<T> {
    name: &'static str,
    descriptor: Option<ExtensionDescriptor>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Extension<T> {
    fn new(name: &'static str, pool: &DescriptorPool) -> Self {
        Self {
            name,
            descriptor: pool.get_extension_by_name(name),
            _marker: std::marker::PhantomData,
        }
    }

    fn descriptor(&self) -> Option<&ExtensionDescriptor> {
        self.descriptor.as_ref()
    }

    fn decode(&self, incoming: &T::Incoming) -> anyhow::Result<Option<T>>
    where
        T: ProstExtension,
    {
        let mut messages = self.decode_all(incoming)?;
        Ok(if messages.is_empty() {
            None
        } else {
            Some(messages.swap_remove(0))
        })
    }

    fn decode_all(&self, incoming: &T::Incoming) -> anyhow::Result<Vec<T>>
    where
        T: ProstExtension,
    {
        let extension = match &self.descriptor {
            Some(ext) => ext,
            None => return Ok(Vec::new()),
        };

        let descriptor = match T::get_options(incoming) {
            Some(desc) => desc,
            None => return Ok(Vec::new()),
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
                    message.transcode_to::<T>().context("transcoding failed")
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

impl ProstExtension for tinc_pb_prost::MessageOptions {
    type Incoming = prost_reflect::MessageDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::FieldOptions {
    type Incoming = prost_reflect::FieldDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::PredefinedConstraints {
    type Incoming = prost_reflect::FieldDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::EnumOptions {
    type Incoming = prost_reflect::EnumDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::EnumVariantOptions {
    type Incoming = prost_reflect::EnumValueDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::MethodOptions {
    type Incoming = prost_reflect::MethodDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::ServiceOptions {
    type Incoming = prost_reflect::ServiceDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb_prost::OneofOptions {
    type Incoming = prost_reflect::OneofDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

fn rename_field(field: &str, style: tinc_pb_prost::RenameAll) -> Option<String> {
    match style {
        tinc_pb_prost::RenameAll::LowerCase => Some(field.to_lowercase()),
        tinc_pb_prost::RenameAll::UpperCase => Some(field.to_uppercase()),
        tinc_pb_prost::RenameAll::PascalCase => Some(field.to_case(Case::Pascal)),
        tinc_pb_prost::RenameAll::CamelCase => Some(field.to_case(Case::Camel)),
        tinc_pb_prost::RenameAll::SnakeCase => Some(field.to_case(Case::Snake)),
        tinc_pb_prost::RenameAll::KebabCase => Some(field.to_case(Case::Kebab)),
        tinc_pb_prost::RenameAll::ScreamingSnakeCase => Some(field.to_case(Case::UpperSnake)),
        tinc_pb_prost::RenameAll::ScreamingKebabCase => Some(field.to_case(Case::UpperKebab)),
        tinc_pb_prost::RenameAll::Unspecified => None,
    }
}

pub(crate) struct Extensions<'a> {
    pool: &'a DescriptorPool,
    // Message extensions.
    ext_message: Extension<tinc_pb_prost::MessageOptions>,
    ext_field: Extension<tinc_pb_prost::FieldOptions>,
    ext_oneof: Extension<tinc_pb_prost::OneofOptions>,
    ext_predefined: Extension<tinc_pb_prost::PredefinedConstraints>,

    // Enum extensions.
    ext_enum: Extension<tinc_pb_prost::EnumOptions>,
    ext_variant: Extension<tinc_pb_prost::EnumVariantOptions>,

    // Service extensions.
    ext_method: Extension<tinc_pb_prost::MethodOptions>,
    ext_service: Extension<tinc_pb_prost::ServiceOptions>,
}

impl<'a> Extensions<'a> {
    pub(crate) fn new(pool: &'a DescriptorPool) -> Self {
        Self {
            pool,
            ext_message: Extension::new("tinc.message", pool),
            ext_field: Extension::new("tinc.field", pool),
            ext_predefined: Extension::new("tinc.predefined", pool),
            ext_enum: Extension::new("tinc.enum", pool),
            ext_variant: Extension::new("tinc.variant", pool),
            ext_method: Extension::new("tinc.method", pool),
            ext_service: Extension::new("tinc.service", pool),
            ext_oneof: Extension::new("tinc.oneof", pool),
        }
    }

    pub(crate) fn process(&self, registry: &mut ProtoTypeRegistry) -> anyhow::Result<()> {
        self.pool
            .files()
            .map(|file| FileWalker::new(file, self))
            .try_for_each(|file| {
                anyhow::ensure!(
                    !file.file.package_name().is_empty(),
                    "you must provide a proto package for file: {}",
                    file.file.name()
                );

                file.process(registry)
            })
    }
}

struct FileWalker<'a> {
    file: FileDescriptor,
    extensions: &'a Extensions<'a>,
    locations: Vec<Location>,
}

impl<'a> FileWalker<'a> {
    fn new(file: FileDescriptor, extensions: &'a Extensions) -> Self {
        Self {
            extensions,
            locations: file
                .file_descriptor_proto()
                .source_code_info
                .clone()
                .map(|mut si| {
                    si.location.retain(|l| {
                        let len = l.path.len();
                        len > 0 && len.is_multiple_of(2)
                    });

                    si.location.sort_by(|a, b| a.path.cmp(&b.path));

                    si.location
                })
                .unwrap_or_default(),
            file,
        }
    }

    fn location(&self, path: &[i32]) -> Option<&Location> {
        let idx = self
            .locations
            .binary_search_by_key(&path, |location| location.path.as_slice())
            .ok()?;
        Some(&self.locations[idx])
    }

    fn process(&self, registry: &mut ProtoTypeRegistry) -> anyhow::Result<()> {
        for message in self.file.messages() {
            // FileDescriptorProto.message_type = 4
            self.process_message(&message, registry)
                .with_context(|| format!("message {}", message.full_name()))?;
        }

        for enum_ in self.file.enums() {
            // FileDescriptorProto.enum_type = 5
            self.process_enum(&enum_, registry)
                .with_context(|| format!("enum {}", enum_.full_name()))?;
        }

        for service in self.file.services() {
            // FileDescriptorProto.service = 6
            self.process_service(&service, registry)
                .with_context(|| format!("service {}", service.full_name()))?;
        }

        Ok(())
    }

    fn process_service(&self, service: &ServiceDescriptor, registry: &mut ProtoTypeRegistry) -> anyhow::Result<()> {
        if registry.get_service(service.full_name()).is_some() {
            return Ok(());
        }

        let mut methods = IndexMap::new();

        let opts = self.extensions.ext_service.decode(service)?.unwrap_or_default();
        let service_full_name = ProtoPath::new(service.full_name());

        for method in service.methods() {
            let input = method.input();
            let output = method.output();

            let method_input = ProtoValueType::from_proto_path(input.full_name());
            let method_output = ProtoValueType::from_proto_path(output.full_name());

            let opts = self
                .extensions
                .ext_method
                .decode(&method)
                .with_context(|| format!("method {}", method.full_name()))?
                .unwrap_or_default();

            let mut endpoints = Vec::new();
            for endpoint in opts.endpoint {
                let Some(method) = endpoint.method else {
                    continue;
                };

                endpoints.push(ProtoServiceMethodEndpoint {
                    method,
                    request: endpoint.request,
                    response: endpoint.response,
                });
            }

            methods.insert(
                method.name().to_owned(),
                ProtoServiceMethod {
                    full_name: ProtoPath::new(method.full_name()),
                    service: service_full_name.clone(),
                    comments: self.location(method.path()).map(location_to_comments).unwrap_or_default(),
                    input: if method.is_client_streaming() {
                        ProtoServiceMethodIo::Stream(method_input)
                    } else {
                        ProtoServiceMethodIo::Single(method_input)
                    },
                    output: if method.is_server_streaming() {
                        ProtoServiceMethodIo::Stream(method_output)
                    } else {
                        ProtoServiceMethodIo::Single(method_output)
                    },
                    endpoints,
                    cel: opts
                        .cel
                        .into_iter()
                        .map(|expr| CelExpression {
                            expression: expr.expression,
                            jsonschemas: expr.jsonschemas,
                            message: expr.message,
                            this: None,
                        })
                        .collect(),
                },
            );
        }

        registry.register_service(ProtoService {
            full_name: ProtoPath::new(service.full_name()),
            comments: self.location(service.path()).map(location_to_comments).unwrap_or_default(),
            package: ProtoPath::new(service.package_name()),
            options: ProtoServiceOptions { prefix: opts.prefix },
            methods,
        });

        Ok(())
    }

    fn process_message(&self, message: &MessageDescriptor, registry: &mut ProtoTypeRegistry) -> anyhow::Result<()> {
        let opts = self.extensions.ext_message.decode(message)?;

        let fields = message
            .fields()
            .map(|field| {
                let opts = self
                    .extensions
                    .ext_field
                    .decode(&field)
                    .with_context(|| field.full_name().to_owned())?;
                Ok((field, opts.unwrap_or_default()))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let opts = opts.unwrap_or_default();
        let message_full_name = ProtoPath::new(message.full_name());
        let rename_all = opts.rename_all.and_then(|v| tinc_pb_prost::RenameAll::try_from(v).ok());

        let mut message_type = ProtoMessageType {
            full_name: message_full_name.clone(),
            comments: self.location(message.path()).map(location_to_comments).unwrap_or_default(),
            package: ProtoPath::new(message.package_name()),
            fields: IndexMap::new(),
            options: ProtoMessageOptions {
                cel: opts
                    .cel
                    .into_iter()
                    .map(|cel| CelExpression {
                        expression: cel.expression,
                        jsonschemas: cel.jsonschemas,
                        message: cel.message,
                        this: None,
                    })
                    .collect(),
            },
        };

        for (field, opts) in fields {
            // This means the field is nullable, and can be omitted from the payload.
            let proto3_optional = field.field_descriptor_proto().proto3_optional();
            let visibility = ProtoVisibility::from_pb(opts.visibility());

            let field_opts = ProtoFieldOptions {
                serde_omittable: ProtoFieldSerdeOmittable::from_prost_pb(opts.json_omittable(), proto3_optional),
                nullable: proto3_optional,
                visibility,
                flatten: opts.flatten(),
                serde_name: opts
                    .rename
                    .or_else(|| rename_field(field.name(), rename_all?))
                    .unwrap_or_else(|| field.name().to_owned()),
                cel_exprs: gather_cel_expressions(&self.extensions.ext_predefined, &field.options())
                    .context("gathering cel expressions")?,
            };

            let Some(Some(oneof)) = (!proto3_optional).then(|| field.containing_oneof()) else {
                message_type.fields.insert(
                    field.name().to_owned(),
                    ProtoMessageField {
                        full_name: ProtoPath::new(field.full_name()),
                        message: message_full_name.clone(),
                        comments: self.location(field.path()).map(location_to_comments).unwrap_or_default(),
                        ty: match field.kind() {
                            Kind::Message(message) if field.is_map() => ProtoType::Modified(ProtoModifiedValueType::Map(
                                ProtoValueType::from_pb(&message.map_entry_key_field().kind()),
                                ProtoValueType::from_pb(&message.map_entry_value_field().kind()),
                            )),
                            // Prost will generate messages as optional even if they are not optional in the proto.
                            kind if field.is_list() => {
                                ProtoType::Modified(ProtoModifiedValueType::Repeated(ProtoValueType::from_pb(&kind)))
                            }
                            kind if proto3_optional || matches!(kind, Kind::Message(_)) => {
                                ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::from_pb(&kind)))
                            }
                            kind => ProtoType::Value(ProtoValueType::from_pb(&kind)),
                        },
                        options: field_opts,
                    },
                );
                continue;
            };

            let opts = self.extensions.ext_oneof.decode(&oneof)?.unwrap_or_default();
            let mut entry = message_type.fields.entry(oneof.name().to_owned());
            let oneof = match entry {
                indexmap::map::Entry::Occupied(ref mut entry) => entry.get_mut(),
                indexmap::map::Entry::Vacant(entry) => {
                    let visibility = ProtoVisibility::from_pb(opts.visibility());
                    let json_omittable = ProtoFieldSerdeOmittable::from_prost_pb(opts.json_omittable(), false);

                    entry.insert(ProtoMessageField {
                        full_name: ProtoPath::new(oneof.full_name()),
                        message: message_full_name.clone(),
                        comments: self.location(oneof.path()).map(location_to_comments).unwrap_or_default(),
                        options: ProtoFieldOptions {
                            flatten: opts.flatten(),
                            nullable: json_omittable.is_true(),
                            serde_omittable: json_omittable,
                            serde_name: opts
                                .rename
                                .or_else(|| rename_field(oneof.name(), rename_all?))
                                .unwrap_or_else(|| oneof.name().to_owned()),
                            visibility,
                            cel_exprs: CelExpressions::default(),
                        },
                        ty: ProtoType::Modified(ProtoModifiedValueType::OneOf(ProtoOneOfType {
                            full_name: ProtoPath::new(oneof.full_name()),
                            message: message_full_name.clone(),
                            fields: IndexMap::new(),
                            options: ProtoOneOfOptions {
                                tagged: opts.tagged.clone().map(|tagged| Tagged {
                                    content: tagged.content,
                                    tag: tagged.tag,
                                }),
                            },
                        })),
                    })
                }
            };

            let ProtoType::Modified(ProtoModifiedValueType::OneOf(ProtoOneOfType {
                ref full_name,
                ref mut fields,
                ..
            })) = oneof.ty
            else {
                panic!("field type is not a oneof but is being added to a oneof");
            };

            let field_ty = ProtoValueType::from_pb(&field.kind());

            fields.insert(
                field.name().to_owned(),
                ProtoOneOfField {
                    // This is because the field name should contain the oneof name, by
                    // default the `field.full_name()` just has the field name on the message
                    // instead of through the oneof.
                    full_name: ProtoPath::new(format!("{full_name}.{}", field.name())),
                    message: message_full_name.clone(),
                    comments: self.location(field.path()).map(location_to_comments).unwrap_or_default(),
                    ty: field_ty.clone(),
                    options: field_opts,
                },
            );
        }

        registry.register_message(message_type);

        for child in message.child_messages() {
            if child.is_map_entry() {
                continue;
            }

            self.process_message(&child, registry)?;
        }

        for child in message.child_enums() {
            self.process_enum(&child, registry)?;
        }

        Ok(())
    }

    fn process_enum(&self, enum_: &EnumDescriptor, registry: &mut ProtoTypeRegistry) -> anyhow::Result<()> {
        let opts = self.extensions.ext_enum.decode(enum_)?;

        let values = enum_
            .values()
            .map(|value| {
                let opts = self
                    .extensions
                    .ext_variant
                    .decode(&value)
                    .with_context(|| value.full_name().to_owned())?;
                Ok((value, opts))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let opts = opts.unwrap_or_default();
        let rename_all = opts
            .rename_all
            .and_then(|v| tinc_pb_prost::RenameAll::try_from(v).ok())
            .unwrap_or(tinc_pb_prost::RenameAll::ScreamingSnakeCase);

        let mut enum_opts = ProtoEnumType {
            full_name: ProtoPath::new(enum_.full_name()),
            comments: self.location(enum_.path()).map(location_to_comments).unwrap_or_default(),
            package: ProtoPath::new(enum_.package_name()),
            variants: IndexMap::new(),
            options: ProtoEnumOptions {
                repr_enum: opts.repr_enum(),
            },
        };

        for (variant, opts) in values {
            let opts = opts.unwrap_or_default();

            let visibility = ProtoVisibility::from_pb(opts.visibility());

            let name = strip_enum_prefix(&to_upper_camel(enum_.name()), &to_upper_camel(variant.name()));

            enum_opts.variants.insert(
                variant.name().to_owned(),
                ProtoEnumVariant {
                    comments: self.location(variant.path()).map(location_to_comments).unwrap_or_default(),
                    // This is not the same as variant.full_name() because that strips the enum name.
                    full_name: ProtoPath::new(format!("{}.{}", enum_.full_name(), variant.name())),
                    value: variant.number(),
                    rust_ident: format_ident!("{name}"),
                    options: ProtoEnumVariantOptions {
                        visibility,
                        serde_name: opts.rename.or_else(|| rename_field(&name, rename_all)).unwrap_or(name),
                    },
                },
            );
        }

        registry.register_enum(enum_opts);

        Ok(())
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum CelInput {
    Root,
    MapKey,
    MapValue,
    RepeatedItem,
}

pub(crate) fn gather_cel_expressions(
    extension: &Extension<tinc_pb_prost::PredefinedConstraints>,
    field_options: &prost_reflect::DynamicMessage,
) -> anyhow::Result<CelExpressions> {
    let Some(extension) = extension.descriptor() else {
        return Ok(CelExpressions::default());
    };

    let mut results = BTreeMap::new();
    let mut input = CelInput::Root;

    if field_options.has_extension(extension) {
        let value = field_options.get_extension(extension);
        let predef = value
            .as_message()
            .context("expected message")?
            .transcode_to::<tinc_pb_prost::PredefinedConstraints>()
            .context("invalid predefined constraint")?;
        match predef.r#type() {
            tinc_pb_prost::predefined_constraints::Type::Unspecified => {}
            tinc_pb_prost::predefined_constraints::Type::CustomExpression => {}
            tinc_pb_prost::predefined_constraints::Type::WrapperMapKey => {
                input = CelInput::MapKey;
            }
            tinc_pb_prost::predefined_constraints::Type::WrapperMapValue => {
                input = CelInput::MapValue;
            }
            tinc_pb_prost::predefined_constraints::Type::WrapperRepeatedItem => {
                input = CelInput::RepeatedItem;
            }
        }
    }

    for (ext, value) in field_options.extensions() {
        if &ext == extension {
            continue;
        }

        if let Some(message) = value.as_message() {
            explore_fields(extension, input, message, &mut results)?;
        }
    }

    Ok(CelExpressions {
        field: results.remove(&CelInput::Root).unwrap_or_default(),
        map_key: results.remove(&CelInput::MapKey).unwrap_or_default(),
        map_value: results.remove(&CelInput::MapValue).unwrap_or_default(),
        repeated_item: results.remove(&CelInput::RepeatedItem).unwrap_or_default(),
    })
}

fn explore_fields(
    extension: &prost_reflect::ExtensionDescriptor,
    input: CelInput,
    value: &prost_reflect::DynamicMessage,
    results: &mut BTreeMap<CelInput, Vec<CelExpression>>,
) -> anyhow::Result<()> {
    for (field, value) in value.fields() {
        let options = field.options();
        let mut input = input;
        if options.has_extension(extension) {
            let message = options.get_extension(extension);
            let predef = message
                .as_message()
                .unwrap()
                .transcode_to::<tinc_pb_prost::PredefinedConstraints>()
                .unwrap();
            match predef.r#type() {
                tinc_pb_prost::predefined_constraints::Type::Unspecified => {}
                tinc_pb_prost::predefined_constraints::Type::CustomExpression => {
                    if let Some(list) = value.as_list() {
                        results.entry(input).or_default().extend(
                            list.iter()
                                .filter_map(|item| item.as_message())
                                .filter_map(|msg| msg.transcode_to::<tinc_pb_prost::CelExpression>().ok())
                                .map(|expr| CelExpression {
                                    expression: expr.expression,
                                    jsonschemas: expr.jsonschemas,
                                    message: expr.message,
                                    this: None,
                                }),
                        );
                    }
                    continue;
                }
                tinc_pb_prost::predefined_constraints::Type::WrapperMapKey => {
                    input = CelInput::MapKey;
                }
                tinc_pb_prost::predefined_constraints::Type::WrapperMapValue => {
                    input = CelInput::MapValue;
                }
                tinc_pb_prost::predefined_constraints::Type::WrapperRepeatedItem => {
                    input = CelInput::RepeatedItem;
                }
            }

            results
                .entry(input)
                .or_default()
                .extend(predef.cel.into_iter().map(|expr| CelExpression {
                    expression: expr.expression,
                    jsonschemas: expr.jsonschemas,
                    message: expr.message,
                    this: Some(prost_to_cel(value, &field.kind())),
                }));
        }

        let Some(message) = value.as_message() else {
            continue;
        };

        explore_fields(extension, input, message, results)?;
    }

    Ok(())
}

fn prost_to_cel(value: &prost_reflect::Value, kind: &Kind) -> tinc_cel::CelValue<'static> {
    match value {
        prost_reflect::Value::String(s) => tinc_cel::CelValue::String(s.clone().into()),
        prost_reflect::Value::Message(msg) => tinc_cel::CelValue::Map(
            msg.fields()
                .map(|(field, value)| {
                    (
                        tinc_cel::CelValue::String(field.name().to_owned().into()),
                        prost_to_cel(value, &field.kind()),
                    )
                })
                .collect(),
        ),
        prost_reflect::Value::EnumNumber(value) => tinc_cel::CelValue::Enum(CelEnum::new(
            kind.as_enum().expect("enum").full_name().to_owned().into(),
            *value,
        )),
        prost_reflect::Value::Bool(v) => v.conv(),
        prost_reflect::Value::I32(v) => v.conv(),
        prost_reflect::Value::I64(v) => v.conv(),
        prost_reflect::Value::U32(v) => v.conv(),
        prost_reflect::Value::U64(v) => v.conv(),
        prost_reflect::Value::Bytes(b) => tinc_cel::CelValue::Bytes(b.into()),
        prost_reflect::Value::F32(v) => v.conv(),
        prost_reflect::Value::F64(v) => v.conv(),
        prost_reflect::Value::List(list) => {
            tinc_cel::CelValue::List(list.iter().map(|item| prost_to_cel(item, kind)).collect())
        }
        prost_reflect::Value::Map(map) => tinc_cel::CelValue::Map(
            map.iter()
                .map(|(key, value)| {
                    let key = match key {
                        prost_reflect::MapKey::Bool(v) => v.conv(),
                        prost_reflect::MapKey::I32(v) => v.conv(),
                        prost_reflect::MapKey::I64(v) => v.conv(),
                        prost_reflect::MapKey::U32(v) => v.conv(),
                        prost_reflect::MapKey::U64(v) => v.conv(),
                        prost_reflect::MapKey::String(s) => tinc_cel::CelValue::String(s.clone().into()),
                    };

                    let v = prost_to_cel(value, &kind.as_message().expect("map").map_entry_value_field().kind());
                    (key, v)
                })
                .collect(),
        ),
    }
}

fn location_to_comments(location: &Location) -> Comments {
    Comments {
        leading: location.leading_comments.as_deref().map(Into::into),
        detached: location.leading_detached_comments.iter().map(|s| s.as_str().into()).collect(),
        trailing: location.trailing_comments.as_deref().map(Into::into),
    }
}

impl ProtoFieldSerdeOmittable {
    pub(crate) fn from_prost_pb(value: tinc_pb_prost::JsonOmittable, nullable: bool) -> Self {
        match value {
            tinc_pb_prost::JsonOmittable::Unspecified => {
                if nullable {
                    Self::TrueButStillSerialize
                } else {
                    Self::False
                }
            }
            tinc_pb_prost::JsonOmittable::True => Self::True,
            tinc_pb_prost::JsonOmittable::False => Self::False,
            tinc_pb_prost::JsonOmittable::TrueButStillSerialize => Self::TrueButStillSerialize,
        }
    }
}
