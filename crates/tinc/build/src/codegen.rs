use std::collections::BTreeMap;

use convert_case::{Boundary, Casing};
use tinc_pb::schema_oneof_options::Tagged;

use crate::extensions::{EnumOpts, Extensions, FieldKind, FieldVisibility, MessageOpts, PrimitiveKind, ServiceOpts};

fn message_attributes(key: &str, prost: &mut tonic_build::Config) {
    prost.message_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
    prost.message_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
    prost.message_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema)]");
    prost.message_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
    prost.message_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
    prost.message_attribute(key, "#[schemars(deny_unknown_fields)]");
    prost.message_attribute(key, format!("#[schemars(rename = \"{key}\")]"));
}

fn enum_attributes(key: &str, prost: &mut tonic_build::Config, repr_enum: bool) {
    if repr_enum {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde_repr::Serialize_repr)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde_repr::Deserialize_repr)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema_repr)]");
    } else {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema)]");
    }
    prost.enum_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
    prost.enum_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
    prost.enum_attribute(key, format!("#[schemars(rename = \"{key}\")]"));
}

fn field_omitable(key: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, "#[serde(default)]");
}

fn field_visibility(key: &str, prost: &mut tonic_build::Config, visibility: Option<FieldVisibility>) {
    match visibility {
        Some(FieldVisibility::Skip) => prost.field_attribute(key, "#[serde(skip)]"),
        Some(FieldVisibility::InputOnly) => prost.field_attribute(key, "#[serde(skip_serializing)]"),
        Some(FieldVisibility::OutputOnly) => prost.field_attribute(key, "#[serde(skip_deserializing)]"),
        _ => return,
    };
}

fn rename_all(key: &str, style: Option<i32>, prost: &mut tonic_build::Config, is_enum: bool) -> bool {
    let Some(style) = style
        .and_then(|style| tinc_pb::RenameAll::try_from(style).ok())
        .and_then(rename_all_to_serde_rename_all)
    else {
        return false;
    };

    let attr = format!("#[serde(rename_all = \"{style}\")]");
    if is_enum {
        prost.enum_attribute(key, &attr);
    } else {
        prost.message_attribute(key, &attr);
    }

    true
}

fn serde_rename(key: &str, name: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, format!("#[serde(rename = \"{name}\")]"));
}

fn get_common_import(start: &str, end: &str) -> String {
    let start_parts: Vec<&str> = start.split('.').collect();
    let end_parts: Vec<&str> = end.split('.').collect();

    // Find common prefix length
    let common_len = start_parts.iter().zip(&end_parts).take_while(|(a, b)| a == b).count();

    // Number of `super::` needed
    let num_supers = start_parts.len() - common_len - 2;
    let super_prefix = "super::".repeat(num_supers);

    // Remaining path from the common ancestor
    let relative_path = end_parts[common_len..].join("::");

    // Construct the final result
    format!("{}{}", super_prefix, relative_path)
}

fn with_attr(key: &str, mut field_kind: &FieldKind, nullable: bool, omitable: bool, prost: &mut tonic_build::Config) {
    fn schemars_with(field_kind: &FieldKind, current_namespace: &str) -> Option<String> {
        match field_kind {
            FieldKind::WellKnown(well_known) => Some(format!("::tinc::helpers::well_known::{}", well_known.name())),
            FieldKind::Optional(inner) => Some(format!(
                "::core::option::Option<{}>",
                schemars_with(inner, current_namespace)?
            )),
            FieldKind::List(inner) => Some(format!("::std::vec::Vec<{}>", schemars_with(inner, current_namespace)?)),
            FieldKind::Map(key, inner) => Some(format!(
                "::std::collections::HashMap<{}, {}>",
                match key {
                    PrimitiveKind::String => "::std::string::String",
                    PrimitiveKind::Bool => "::core::primitive::bool",
                    PrimitiveKind::I32 => "::core::primitive::i32",
                    PrimitiveKind::I64 => "::core::primitive::i64",
                    PrimitiveKind::U32 => "::core::primitive::u32",
                    PrimitiveKind::U64 => "::core::primitive::u64",
                    PrimitiveKind::Bytes => unimplemented!("map keys cannot be bytes"),
                    PrimitiveKind::F32 => unimplemented!("map keys cannot be f32"),
                    PrimitiveKind::F64 => unimplemented!("map keys cannot be f64"),
                },
                schemars_with(inner, current_namespace)?,
            )),
            FieldKind::Enum(name) => Some(get_common_import(current_namespace, name)),
            FieldKind::Primitive(_) => None,
            FieldKind::Message(_) => None,
        }
    }

    let is_optional = matches!(field_kind, FieldKind::Optional(_));

    match field_kind.inner() {
        // Well known types are `Message` types which get generated as `Option<T>`
        // Therefore we need to strip this option when its !nullable
        Some(FieldKind::WellKnown(_)) => {
            prost.field_attribute(key, "#[serde(serialize_with = \"::tinc::helpers::well_known::serialize\")]");
            prost.field_attribute(
                key,
                format!(
                    "#[serde(deserialize_with = \"{}\")]",
                    if is_optional && !nullable {
                        "::tinc::helpers::well_known::deserialize_non_optional"
                    } else {
                        "::tinc::helpers::well_known::deserialize"
                    }
                ),
            );
        }
        Some(FieldKind::Enum(name)) => {
            prost.field_attribute(
                key,
                format!(
                    "#[serde(with = \"::tinc::helpers::Enum::<{}>\")]",
                    get_common_import(key, name)
                ),
            );
        }
        _ if is_optional && !nullable => {
            prost.field_attribute(
                key,
                "#[serde(deserialize_with = \"::tinc::helpers::deserialize_non_null_option\")]",
            );
        }
        _ if is_optional && !omitable => {
            prost.field_attribute(
                key,
                "#[serde(deserialize_with = \"::tinc::helpers::deserialize_non_omitable\")]",
            );
        }
        _ => {}
    }

    if is_optional && (!nullable || !omitable) {
        field_kind = field_kind.strip_option();
        prost.field_attribute(key, "#[schemars(required)]");
    }

    if let Some(with) = schemars_with(field_kind, key) {
        prost.field_attribute(key, &format!("#[schemars(with = \"{with}\")]"));
    }

    if nullable && !omitable {
        prost.field_attribute(key, "#[schemars(transform = ::tinc::helpers::schemars_non_omitable)]");
    }
}

fn rename_all_to_serde_rename_all(style: tinc_pb::RenameAll) -> Option<&'static str> {
    match style {
        tinc_pb::RenameAll::LowerCase => Some("lowercase"),
        tinc_pb::RenameAll::UpperCase => Some("uppercase"),
        tinc_pb::RenameAll::PascalCase => Some("PascalCase"),
        tinc_pb::RenameAll::CamelCase => Some("camelCase"),
        tinc_pb::RenameAll::SnakeCase => Some("snake_case"),
        tinc_pb::RenameAll::KebabCase => Some("kebab-case"),
        tinc_pb::RenameAll::ScreamingSnakeCase => Some("SCREAMING_SNAKE_CASE"),
        tinc_pb::RenameAll::ScreamingKebabCase => Some("SCREAMING-KEBAB-CASE"),
        tinc_pb::RenameAll::Unspecified => None,
    }
}

fn object_type_path(key: &str, package: &str) -> syn::Path {
    println!("key: {key} package: {package}");

    let mut key = key
        .strip_prefix(package)
        .expect("key not in package")
        .split('.')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();

    let key_length = key.len();
    if key_length > 1 {
        for k in key[..key_length - 1].iter_mut() {
            *k = k
                .with_boundaries(&Boundary::digit_letter())
                .to_case(convert_case::Case::Snake);
        }

        key[key_length - 1] = key[key_length - 1].to_case(convert_case::Case::Pascal);
    }

    syn::parse_str::<syn::Path>(&key.join("::")).unwrap()
}

fn handle_message(
    message_key: &str,
    message: &MessageOpts,
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    let message_custom_impl = message.opts.custom_impl.unwrap_or(false);
    for (oneof_name, oneof) in &message.oneofs {
        let oneof_key = format!("{message_key}.{oneof_name}");

        if !message_custom_impl {
            if let Some(rename) = &oneof.opts.rename {
                serde_rename(&oneof_key, rename, prost);
            }

            if !oneof.opts.nullable() {
                prost.enum_attribute(&oneof_key, "#[schemars(required)]");
            } else if !oneof.opts.omitable() {
                prost.enum_attribute(&oneof_key, "#[schemars(required)]");
                prost.enum_attribute(&oneof_key, "#[schemars(transform = ::tinc::helpers::schemars_non_omitable)]");
            }
        }

        if oneof.opts.custom_impl.unwrap_or(message_custom_impl) {
            continue;
        }

        // let type_path = object_type_path(&oneof_key, &message.package);

        // modules.entry(message.package.clone()).or_default().push(parse_quote! {
        //     const _: () = {
        //         impl ::tinc::Schema for #type_path {
        //         }
        //     };
        // });

        enum_attributes(&oneof_key, prost, false);
        rename_all(&oneof_key, oneof.opts.rename_all, prost, true);

        match &oneof.opts.tagged {
            Some(Tagged {
                tag,
                content: Some(content),
            }) => {
                prost.enum_attribute(&oneof_key, &format!("#[serde(tag = \"{tag}\", content = \"{content}\")]"));
            }
            Some(Tagged { tag, content: None }) => {
                prost.enum_attribute(&oneof_key, &format!("#[serde(tag = \"{tag}\")]"));
            }
            None => {}
        }
    }

    if message_custom_impl {
        return Ok(());
    }

    message_attributes(message_key, prost);
    rename_all(message_key, message.opts.rename_all, prost, false);

    for (field_name, field) in &message.fields {
        if field
            .one_of
            .as_ref()
            .is_some_and(|oneof| message.oneofs.get(oneof).unwrap().opts.custom_impl.unwrap_or(false))
        {
            continue;
        }

        let name = field
            .opts
            .rename
            .as_ref()
            .or_else(|| message.opts.rename_all.is_none().then_some(&field.json_name));

        let field_key = if let Some(oneof) = &field.one_of {
            format!("{message_key}.{oneof}.{field_name}")
        } else {
            format!("{message_key}.{field_name}")
        };

        if let Some(name) = name {
            serde_rename(&field_key, name, prost);
        }

        with_attr(&field_key, &field.kind, field.nullable, field.omitable, prost);

        if field.omitable {
            field_omitable(&field_key, prost);
        }
        field_visibility(&field_key, prost, field.visibility);
    }

    Ok(())
}

fn handle_enum(
    enum_key: &str,
    enum_: &EnumOpts,
    prost: &mut tonic_build::Config,
    _: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    if enum_.opts.custom_impl.unwrap_or(false) {
        return Ok(());
    }

    enum_attributes(enum_key, prost, enum_.opts.repr_enum.unwrap_or(false));
    if !enum_.opts.repr_enum() {
        // By default we use screaming_snake_case for enums
        let enum_rename_all = enum_.opts.rename_all.unwrap_or(tinc_pb::RenameAll::ScreamingSnakeCase as i32);
        rename_all(enum_key, Some(enum_rename_all), prost, true);
    }

    for (variant, variant_opts) in &enum_.variants {
        let variant_key = format!("{enum_key}.{variant}");

        if !enum_.opts.repr_enum() {
            if let Some(rename) = &variant_opts.opts.rename {
                serde_rename(&variant_key, rename, prost);
            }
        }

        field_visibility(&variant_key, prost, variant_opts.visibility);
    }

    Ok(())
}

fn handle_service(
    service_key: &str,
    service: &ServiceOpts,
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    Ok(())
}

pub fn generate_modules(
    extensions: &Extensions,
    prost: &mut tonic_build::Config,
) -> anyhow::Result<BTreeMap<String, Vec<syn::Item>>> {
    let mut modules = BTreeMap::new();

    extensions
        .messages()
        .try_for_each(|(key, message)| handle_message(key, message, prost, &mut modules))?;

    extensions
        .enums()
        .try_for_each(|(key, enum_)| handle_enum(key, enum_, prost, &mut modules))?;

    extensions
        .services()
        .try_for_each(|(key, service)| handle_service(key, service, prost, &mut modules))?;

    Ok(modules)
}
