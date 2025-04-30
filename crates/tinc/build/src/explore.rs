use anyhow::Context;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use prost_reflect::{DescriptorPool, EnumDescriptor, ExtensionDescriptor, Kind, MessageDescriptor, ServiceDescriptor};

use super::types::ProtoServiceOptions;
use crate::codegen::cel::{CelExpression, gather_cel_expressions};
use crate::codegen::prost_sanatize::{strip_enum_prefix, to_upper_camel};
use crate::types::{
    ProtoEnumOptions, ProtoEnumType, ProtoEnumVariant, ProtoEnumVariantOptions, ProtoFieldJsonOmittable, ProtoFieldOptions,
    ProtoMessageField, ProtoMessageOptions, ProtoMessageType, ProtoModifiedValueType, ProtoOneOfField, ProtoOneOfOptions,
    ProtoOneOfType, ProtoPath, ProtoService, ProtoServiceMethod, ProtoServiceMethodEndpoint, ProtoServiceMethodIo,
    ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoVisibility, ProtoWellKnownType,
};

fn rename_field(field: &str, style: tinc_pb::RenameAll) -> Option<String> {
    match style {
        tinc_pb::RenameAll::LowerCase => Some(field.to_lowercase()),
        tinc_pb::RenameAll::UpperCase => Some(field.to_uppercase()),
        tinc_pb::RenameAll::PascalCase => Some(field.to_case(Case::Pascal)),
        tinc_pb::RenameAll::CamelCase => Some(field.to_case(Case::Camel)),
        tinc_pb::RenameAll::SnakeCase => Some(field.to_case(Case::Snake)),
        tinc_pb::RenameAll::KebabCase => Some(field.to_case(Case::Kebab)),
        tinc_pb::RenameAll::ScreamingSnakeCase => Some(field.to_case(Case::UpperSnake)),
        tinc_pb::RenameAll::ScreamingKebabCase => Some(field.to_case(Case::UpperKebab)),
        tinc_pb::RenameAll::Unspecified => None,
    }
}

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

    pub fn descriptor(&self) -> Option<&ExtensionDescriptor> {
        self.descriptor.as_ref()
    }

    pub fn name(&self) -> &'static str {
        self.name
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

impl ProstExtension for tinc_pb::MessageOptions {
    type Incoming = prost_reflect::MessageDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::FieldOptions {
    type Incoming = prost_reflect::FieldDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::PredefinedConstraint {
    type Incoming = prost_reflect::FieldDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::EnumOptions {
    type Incoming = prost_reflect::EnumDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::EnumVariantOptions {
    type Incoming = prost_reflect::EnumValueDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::MethodOptions {
    type Incoming = prost_reflect::MethodDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::ServiceOptions {
    type Incoming = prost_reflect::ServiceDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

impl ProstExtension for tinc_pb::OneofOptions {
    type Incoming = prost_reflect::OneofDescriptor;

    fn get_options(incoming: &Self::Incoming) -> Option<prost_reflect::DynamicMessage> {
        Some(incoming.options())
    }
}

pub struct Extensions {
    // Message extensions.
    ext_message: Extension<tinc_pb::MessageOptions>,
    ext_field: Extension<tinc_pb::FieldOptions>,
    ext_oneof: Extension<tinc_pb::OneofOptions>,
    ext_predefined: Extension<tinc_pb::PredefinedConstraint>,

    // Enum extensions.
    ext_enum: Extension<tinc_pb::EnumOptions>,
    ext_variant: Extension<tinc_pb::EnumVariantOptions>,

    // Service extensions.
    ext_method: Extension<tinc_pb::MethodOptions>,
    ext_service: Extension<tinc_pb::ServiceOptions>,
}

const ANY_NOT_SUPPORTED_ERROR: &str = "uses `google.protobuf.Any`, this is currently not supported.";

impl Extensions {
    pub fn new(pool: &DescriptorPool) -> Self {
        Self {
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

    pub fn process(&mut self, pool: &DescriptorPool, registry: &mut ProtoTypeRegistry) -> anyhow::Result<()> {
        for service in pool.services() {
            self.process_service(pool, &service, registry)
                .with_context(|| format!("service {}", service.full_name()))?;
        }

        for message in pool.all_messages() {
            self.process_message(pool, &message, false, registry)
                .with_context(|| format!("message {}", message.full_name()))?;
        }

        for enum_ in pool.all_enums() {
            self.process_enum(&enum_, false, registry)
                .with_context(|| format!("enum {}", enum_.full_name()))?;
        }

        Ok(())
    }

    fn process_service(
        &mut self,
        pool: &DescriptorPool,
        service: &ServiceDescriptor,
        registry: &mut ProtoTypeRegistry,
    ) -> anyhow::Result<()> {
        if registry.get_service(service.full_name()).is_some() {
            return Ok(());
        }

        let opts = self.ext_service.decode(service)?.unwrap_or_default();

        let mut methods = IndexMap::new();

        let service_full_name = ProtoPath::new(service.full_name());

        for method in service.methods() {
            let input = method.input();
            let output = method.output();

            let method_input = ProtoValueType::from_proto_path(input.full_name());
            let method_output = ProtoValueType::from_proto_path(output.full_name());

            anyhow::ensure!(
                !matches!(method_input, ProtoValueType::WellKnown(ProtoWellKnownType::Any)),
                "method {} {ANY_NOT_SUPPORTED_ERROR}",
                method.full_name()
            );
            anyhow::ensure!(
                !matches!(method_output, ProtoValueType::WellKnown(ProtoWellKnownType::Any)),
                "method {} {ANY_NOT_SUPPORTED_ERROR}",
                method.full_name()
            );

            if matches!(method_input, ProtoValueType::Message(_)) {
                self.process_message(pool, &input, true, registry)
                    .with_context(|| format!("message {}", input.full_name()))
                    .with_context(|| format!("method {}", method.full_name()))?;
            }

            if matches!(method_output, ProtoValueType::Message(_)) {
                self.process_message(pool, &output, true, registry)
                    .with_context(|| format!("message {}", output.full_name()))
                    .with_context(|| format!("method {}", method.full_name()))?;
            }

            let opts = self
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
                    input: endpoint.input,
                    response: endpoint.response,
                });
            }

            methods.insert(
                method.name().to_owned(),
                ProtoServiceMethod {
                    full_name: ProtoPath::new(method.full_name()),
                    service: service_full_name.clone(),
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
                        .iter()
                        .map(|expr| CelExpression::new(expr, None))
                        .collect::<anyhow::Result<_>>()?,
                },
            );
        }

        registry.register_service(ProtoService {
            full_name: ProtoPath::new(service.full_name()),
            package: ProtoPath::new(service.package_name()),
            options: ProtoServiceOptions { prefix: opts.prefix },
            methods,
        });

        Ok(())
    }

    fn process_message(
        &mut self,
        pool: &DescriptorPool,
        message: &MessageDescriptor,
        insert: bool,
        registry: &mut ProtoTypeRegistry,
    ) -> anyhow::Result<()> {
        if registry.get_message(message.full_name()).is_some() {
            return Ok(());
        }

        let opts = self.ext_message.decode(message)?;

        let fields = message
            .fields()
            .map(|field| {
                let opts = self.ext_field.decode(&field).with_context(|| field.full_name().to_owned())?;
                Ok((field, opts))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        if !insert && opts.is_none() && fields.iter().all(|(_, opts)| opts.is_none()) {
            return Ok(());
        }

        let message_full_name = ProtoPath::new(message.full_name());

        let rename_all = opts
            .as_ref()
            .and_then(|opts| opts.rename_all.and_then(|v| tinc_pb::RenameAll::try_from(v).ok()));

        registry.register_message(ProtoMessageType {
            full_name: message_full_name.clone(),
            package: ProtoPath::new(message.package_name()),
            fields: IndexMap::new(),
            options: ProtoMessageOptions {
                cel: opts
                    .as_ref()
                    .map(|opts| opts.cel.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .map(|expr| CelExpression::new(expr, None))
                    .collect::<anyhow::Result<_>>()?,
            },
        });

        for (field, opts) in fields {
            let message = registry.get_message_mut(message.full_name()).unwrap();

            let opts = opts.unwrap_or_default();

            // This means the field is nullable, and can be omitted from the payload.
            let proto3_optional = field.field_descriptor_proto().proto3_optional();
            let visibility = ProtoVisibility::from_pb(opts.visibility());

            let field_opts = ProtoFieldOptions {
                json_omittable: ProtoFieldJsonOmittable::from_pb(opts.json_omittable(), proto3_optional),
                nullable: proto3_optional,
                visibility,
                flatten: opts.flatten(),
                json_name: opts
                    .rename
                    .or_else(|| rename_field(field.name(), rename_all?))
                    .unwrap_or_else(|| field.name().to_owned()),
                cel_exprs: gather_cel_expressions(&self.ext_predefined, &field.options())
                    .context("gathering cel expressions")?,
            };

            let Some(Some(oneof)) = (!proto3_optional).then(|| field.containing_oneof()) else {
                message.fields.insert(
                    field.name().to_owned(),
                    ProtoMessageField {
                        full_name: ProtoPath::new(field.full_name()),
                        message: message_full_name.clone(),
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

                let ty = match &message.fields.get(field.name()).unwrap().ty {
                    ProtoType::Value(value) => value.clone(),
                    ProtoType::Modified(ProtoModifiedValueType::Map(_, value)) => value.clone(),
                    ProtoType::Modified(ProtoModifiedValueType::Repeated(value)) => value.clone(),
                    ProtoType::Modified(ProtoModifiedValueType::Optional(value)) => value.clone(),
                    ProtoType::Modified(ProtoModifiedValueType::OneOf(_)) => unreachable!(),
                };

                self.process_subtypes(ty, pool, registry)?;
                continue;
            };

            let opts = self.ext_oneof.decode(&oneof)?.unwrap_or_default();
            let mut entry = message.fields.entry(oneof.name().to_owned());
            let oneof = match entry {
                indexmap::map::Entry::Occupied(ref mut entry) => entry.get_mut(),
                indexmap::map::Entry::Vacant(entry) => {
                    let optional = opts.optional();
                    let visibility = ProtoVisibility::from_pb(opts.visibility());

                    entry.insert(ProtoMessageField {
                        full_name: ProtoPath::new(oneof.full_name()),
                        message: message_full_name.clone(),
                        options: ProtoFieldOptions {
                            flatten: opts.flatten(),
                            nullable: optional,
                            json_omittable: ProtoFieldJsonOmittable::from_pb(opts.json_omittable(), optional),
                            json_name: opts
                                .rename
                                .or_else(|| rename_field(oneof.name(), rename_all?))
                                .unwrap_or_else(|| oneof.name().to_owned()),
                            visibility,
                            cel_exprs: gather_cel_expressions(&self.ext_predefined, &field.options())
                                .context("gathering cel expressions")?,
                        },
                        ty: ProtoType::Modified(ProtoModifiedValueType::OneOf(ProtoOneOfType {
                            full_name: ProtoPath::new(oneof.full_name()),
                            message: message_full_name.clone(),
                            fields: IndexMap::new(),
                            options: ProtoOneOfOptions {
                                tagged: opts.tagged.clone(),
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
                    ty: field_ty.clone(),
                    options: field_opts,
                },
            );

            self.process_subtypes(field_ty, pool, registry)?;
        }

        Ok(())
    }

    fn process_subtypes(
        &mut self,
        ty: ProtoValueType,
        pool: &DescriptorPool,
        registry: &mut ProtoTypeRegistry,
    ) -> anyhow::Result<()> {
        match ty {
            ProtoValueType::Enum(path) => self.process_enum(&pool.get_enum_by_name(&path).unwrap(), true, registry),
            ProtoValueType::Message(path) => {
                self.process_message(pool, &pool.get_message_by_name(&path).unwrap(), true, registry)
            }
            _ => Ok(()),
        }
    }

    fn process_enum(
        &mut self,
        enum_: &EnumDescriptor,
        insert: bool,
        registry: &mut ProtoTypeRegistry,
    ) -> anyhow::Result<()> {
        if registry.get_enum(enum_.full_name()).is_some() {
            return Ok(());
        }

        let opts = self.ext_enum.decode(enum_)?;

        let values = enum_
            .values()
            .map(|value| {
                let opts = self
                    .ext_variant
                    .decode(&value)
                    .with_context(|| value.full_name().to_owned())?;
                Ok((value, opts))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        if !insert && opts.is_none() && values.iter().all(|(_, opts)| opts.is_none()) {
            return Ok(());
        }

        let opts = opts.unwrap_or_default();
        let rename_all = opts
            .rename_all
            .and_then(|v| tinc_pb::RenameAll::try_from(v).ok())
            .unwrap_or(tinc_pb::RenameAll::ScreamingSnakeCase);

        let mut enum_opts = ProtoEnumType {
            full_name: ProtoPath::new(enum_.full_name()),
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
                    // This is not the same as variant.full_name() because that strips the enum name.
                    full_name: ProtoPath::new(format!("{}.{}", enum_.full_name(), variant.name())),
                    value: variant.number(),
                    options: ProtoEnumVariantOptions {
                        visibility,
                        json_name: opts.rename.or_else(|| rename_field(&name, rename_all)).unwrap_or(name),
                    },
                },
            );
        }

        registry.register_enum(enum_opts);

        Ok(())
    }
}
