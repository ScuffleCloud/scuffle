use std::collections::BTreeMap;

use tinc_pb::schema_oneof_options::Tagged;

use crate::codegen::get_common_import;
use crate::extensions::{EnumOpts, FieldKind, FieldVisibility, MessageOpts, PrimitiveKind};

fn message_attributes(key: &str, prost: &mut tonic_build::Config) {
    let attrs = [
        "#[derive(::tinc::reexports::serde::Serialize)]",
        "#[derive(::tinc::reexports::serde::Deserialize)]",
        "#[derive(::tinc::reexports::schemars::JsonSchema)]",
        "#[serde(crate = \"::tinc::reexports::serde\")]",
        "#[schemars(crate = \"::tinc::reexports::schemars\")]",
        "#[schemars(deny_unknown_fields)]",
        &format!("#[schemars(rename = \"{key}\")]"),
    ];

    for attr in &attrs {
        prost.message_attribute(key, attr);
    }
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
    if let Some(visibility) = visibility {
        let attr = match visibility {
            FieldVisibility::Skip => "#[serde(skip)]",
            FieldVisibility::InputOnly => "#[serde(skip_serializing)]",
            FieldVisibility::OutputOnly => "#[serde(skip_deserializing)]",
        };

        prost.field_attribute(key, attr);
    }
}

fn rename_all(key: &str, style: Option<i32>, prost: &mut tonic_build::Config, is_enum: bool) -> bool {
    if let Some(style) = style
        .and_then(|s| tinc_pb::RenameAll::try_from(s).ok())
        .and_then(rename_all_to_serde_rename_all)
    {
        let attr = format!("#[serde(rename_all = \"{style}\")]");
        if is_enum {
            prost.enum_attribute(key, &attr);
        } else {
            prost.message_attribute(key, &attr);
        }

        true
    } else {
        false
    }
}

fn serde_rename(key: &str, name: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, format!("#[serde(rename = \"{name}\")]"));
}

fn with_attr(key: &str, mut field_kind: &FieldKind, nullable: bool, omitable: bool, prost: &mut tonic_build::Config) {
    fn schemars_with(field_kind: &FieldKind, current_namespace: &str) -> Option<String> {
        match field_kind {
            FieldKind::WellKnown(well_known) => Some(well_known.path().to_owned()),
            FieldKind::Optional(inner) => Some(format!(
                "::core::option::Option<{}>",
                schemars_with(inner, current_namespace)?
            )),
            FieldKind::List(inner) => Some(format!("::std::vec::Vec<{}>", schemars_with(inner, current_namespace)?)),
            FieldKind::Map(key, inner) => Some(format!(
                "::std::collections::HashMap<{}, {}>",
                match key {
                    PrimitiveKind::Bytes => unimplemented!("map keys cannot be bytes"),
                    PrimitiveKind::F32 => unimplemented!("map keys cannot be f32"),
                    PrimitiveKind::F64 => unimplemented!("map keys cannot be f64"),
                    _ => key.path(),
                },
                schemars_with(inner, current_namespace)?
            )),
            FieldKind::Enum(name) => Some(get_common_import(current_namespace, name)),
            FieldKind::Primitive(_) => None,
            FieldKind::Message(_) => None,
        }
    }

    let is_optional = matches!(field_kind, FieldKind::Optional(_));

    match field_kind.inner() {
        // Special handling for well-known types.
        FieldKind::WellKnown(_) => {
            prost.field_attribute(key, "#[serde(serialize_with = \"::tinc::helpers::well_known::serialize\")]");
            let deserialize_fn = if is_optional && !nullable {
                "::tinc::helpers::well_known::deserialize_non_optional"
            } else {
                "::tinc::helpers::well_known::deserialize"
            };
            prost.field_attribute(key, format!("#[serde(deserialize_with = \"{deserialize_fn}\")]"));
        }
        FieldKind::Enum(name) => {
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
        prost.field_attribute(key, format!("#[schemars(with = \"{with}\")]"));
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

pub(super) fn handle_message(
    message_key: &str,
    message: &MessageOpts,
    prost: &mut tonic_build::Config,
    _: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    let message_custom_impl = message.opts.custom_impl.unwrap_or(false);

    // Process oneof fields.
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

        enum_attributes(&oneof_key, prost, false);
        rename_all(&oneof_key, oneof.opts.rename_all, prost, true);

        if let Some(Tagged { tag, content }) = &oneof.opts.tagged {
            let attr = if let Some(content) = content {
                format!("#[serde(tag = \"{tag}\", content = \"{content}\")]")
            } else {
                format!("#[serde(tag = \"{tag}\")]")
            };

            prost.enum_attribute(&oneof_key, &attr);
        }
    }

    if message_custom_impl {
        return Ok(());
    }

    message_attributes(message_key, prost);
    rename_all(message_key, message.opts.rename_all, prost, false);

    // Process individual fields.
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

pub(super) fn handle_enum(
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
