use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use quote::quote;
use syn::parse_quote;

use super::{field_ident_from_str, ident_from_str};
use crate::codegen::get_common_import_path;
use crate::extensions::{EnumOpts, FieldKind, FieldModifier, FieldType, FieldVisibility, MessageOpts};

fn message_attributes(key: &str, prost: &mut tonic_build::Config) {
    let attrs = [
        "#[derive(::tinc::reexports::serde::Serialize)]",
        "#[serde(crate = \"::tinc::reexports::serde\")]",
    ];

    for attr in &attrs {
        prost.message_attribute(key, attr);
    }
}

fn enum_attributes(key: &str, prost: &mut tonic_build::Config, repr_enum: bool) {
    if repr_enum {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde_repr::Serialize_repr)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde_repr::Deserialize_repr)]");
        // prost.enum_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema_repr)]");
    } else {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
        // prost.enum_attribute(key, "#[derive(::tinc::reexports::schemars::JsonSchema)]");
    }

    prost.enum_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
    // prost.enum_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
    // prost.enum_attribute(key, format!("#[schemars(rename = \"{key}\")]"));
}

fn field_visibility(key: &str, prost: &mut tonic_build::Config, visibility: Option<FieldVisibility>) {
    if let Some(visibility) = visibility {
        let attr = match visibility {
            FieldVisibility::Skip => "#[serde(skip)]",
            FieldVisibility::InputOnly => "#[serde(skip_serializing)]",
            FieldVisibility::OutputOnly => return,
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

fn rename_field(field: &str, style: Option<i32>) -> Option<String> {
    match style.and_then(|s| tinc_pb::RenameAll::try_from(s).ok())? {
        tinc_pb::RenameAll::LowerCase => Some(field.to_case(Case::Lower)),
        tinc_pb::RenameAll::UpperCase => Some(field.to_case(Case::Upper)),
        tinc_pb::RenameAll::PascalCase => Some(field.to_case(Case::Pascal)),
        tinc_pb::RenameAll::CamelCase => Some(field.to_case(Case::Camel)),
        tinc_pb::RenameAll::SnakeCase => Some(field.to_case(Case::Snake)),
        tinc_pb::RenameAll::KebabCase => Some(field.to_case(Case::Kebab)),
        tinc_pb::RenameAll::ScreamingSnakeCase => Some(field.to_case(Case::UpperSnake)),
        tinc_pb::RenameAll::ScreamingKebabCase => Some(field.to_case(Case::UpperKebab)),
        tinc_pb::RenameAll::Unspecified => None,
    }
}

fn serde_rename(key: &str, name: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, format!("#[serde(rename = \"{name}\")]"));
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
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    let message_custom_impl = message.opts.custom_impl.unwrap_or(false);

    // Process oneof fields.
    // for (oneof_name, oneof) in &message.oneofs {
    //     let oneof_key = format!("{message_key}.{oneof_name}");

    //     if !message_custom_impl {
    //         if let Some(rename) = &oneof.opts.rename {
    //             serde_rename(&oneof_key, rename, prost);
    //         }

    //         // if !oneof.opts.nullable() {
    //         //     // prost.enum_attribute(&oneof_key, "#[schemars(required)]");
    //         // } else if !oneof.opts.omitable() {
    //         //     // prost.enum_attribute(&oneof_key, "#[schemars(required)]");
    //         //     // prost.enum_attribute(&oneof_key, "#[schemars(transform = ::tinc::helpers::schemars_non_omitable)]");
    //         // }
    //     }

    //     if oneof.opts.custom_impl.unwrap_or(message_custom_impl) {
    //         continue;
    //     }

    //     enum_attributes(&oneof_key, prost, false);
    //     rename_all(&oneof_key, oneof.opts.rename_all, prost, true);

    //     if let Some(Tagged { tag, content }) = &oneof.opts.tagged {
    //         let attr = if let Some(content) = content {
    //             format!("#[serde(tag = \"{tag}\", content = \"{content}\")]")
    //         } else {
    //             format!("#[serde(tag = \"{tag}\")]")
    //         };

    //         prost.enum_attribute(&oneof_key, &attr);
    //     }
    // }

    if message_custom_impl {
        return Ok(());
    }

    message_attributes(message_key, prost);
    rename_all(message_key, message.opts.rename_all, prost, false);

    let field_enum_ident = ident_from_str("___field_enum");

    let mut field_enum_variants = Vec::new();
    let mut field_enum_idx_fn = Vec::new();
    let mut field_enum_name_fn = Vec::new();
    let mut field_enum_from_str_fn = Vec::new();
    let mut deserializer_fields = Vec::new();
    let mut deserializer_fn = Vec::new();
    let mut verify_deserialize_fn = Vec::new();

    for (idx, (field_name, field)) in message.fields.iter().enumerate() {
        let json_name = field
            .opts
            .rename
            .clone()
            .or_else(|| rename_field(field_name, message.opts.rename_all))
            .unwrap_or_else(|| field.json_name.clone());
        let ident = ident_from_str(format!("__field_{idx}"));
        field_enum_variants.push(ident.clone());
        field_enum_idx_fn.push(quote! {
            #field_enum_ident::#ident => #idx,
        });
        field_enum_name_fn.push(quote! {
            #field_enum_ident::#ident => #json_name,
        });
        field_enum_from_str_fn.push(quote! {
            #json_name => #field_enum_ident::#ident,
        });
        deserializer_fields.push(quote! {
            #json_name,
        });

        let field_name = field_ident_from_str(field_name);

        match field.kind.field_type() {
            FieldType::Enum(path) => {
                let path = get_common_import_path(message.package.as_str(), &path);
                let path_str = quote! { #path };
                prost.field_attribute(
                    format!("{message_key}.{field_name}"),
                    format!("#[serde(serialize_with = \"::tinc::__private::enum_::serialize::<{path_str}, _, _>\")]"),
                );
                prost.field_attribute(
                    format!("{message_key}.{field_name}"),
                    format!("#[tinc(enum = \"{path_str}\")]"),
                );
            }
            FieldType::WellKnown(_) => {
                prost.field_attribute(
                    format!("{message_key}.{field_name}"),
                    "#[serde(serialize_with = \"::tinc::__private::well_known::serialize\")]",
                );
            }
            _ => {}
        };

        let mut tracker = quote! {
            &mut tracker.value.#field_name
        };

        let mut value = quote! {
            &mut self.#field_name
        };

        // When a field is not nullable but prost generates an option<T>, we need to
        // remove the option before deserializing otherwise null will be a valid input.
        if matches!(field.kind, FieldKind::Optional(_)) && !field.nullable {
            tracker = quote! {
                &mut tracker.value.#field_name.get_or_insert_default().0
            };

            value = quote! {
                self.#field_name.get_or_insert_default()
            };
        }

        deserializer_fn.push(quote! {
            #field_enum_ident::#ident => {
                let tracker = #tracker;

                if !::tinc::__private::de::tracker_allow_duplicates(tracker.as_ref()) {
                    return Err(::tinc::reexports::serde::de::Error::duplicate_field(
                        #field_enum_ident::#ident.name(),
                    ));
                }

                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::DeserializeHelper {
                        value: #value,
                        tracker: tracker.get_or_insert_default(),
                    },
                )?;
            }
        });

        if !field.omitable {
            verify_deserialize_fn.push(quote! {
                if tracker.value.#field_name.is_none() {
                    let _token = ::tinc::__private::de::PathToken::push_field(stringify!(#field_name));
                    ::tinc::__private::de::report_error(
                        ::tinc::__private::de::TrackedError::missing_field(),
                    )?;
                }
            });
        }

        if matches!(field.kind.field_type(), FieldType::Message(_)) {
            let validation = match field.kind.modifier() {
                Some(FieldModifier::Optional) => quote! {
                    if let Some(tracker) = tracker.value.#field_name.as_mut().and_then(|tracker| tracker.0.as_mut()) {
                        if let Some(value) = self.#field_name.as_ref() {
                            let _token = ::tinc::__private::de::PathToken::push_field(stringify!(#field_name));
                            ::tinc::__private::de::TrackedStructDeserializer::<'de>::verify_deserialize(
                                value,
                                tracker,
                            )?;
                        }
                    }
                },
                Some(FieldModifier::List) => quote! {
                    if let Some(trackers) = tracker.value.#field_name.as_mut().map(|tracker| tracker.vec.iter_mut()) {
                        let _token = ::tinc::__private::de::PathToken::push_field(stringify!(#field_name));
                        for (idx, (value, tracker)) in self.#field_name.iter().zip(trackers).enumerate() {
                            let _token = ::tinc::__private::de::PathToken::push_index(idx);
                            ::tinc::__private::de::TrackedStructDeserializer::<'de>::verify_deserialize(
                                value,
                                tracker,
                            )?;
                        }
                    }
                },
                Some(FieldModifier::Map) => quote! {
                    if let Some(trackers) = tracker.value.#field_name.as_mut().map(|tracker| tracker.map.iter_mut()) {
                        let _token = ::tinc::__private::de::PathToken::push_field(stringify!(#field_name));
                        for (key, tracker) in trackers {
                            if let Some(value) = self.#field_name.get(key) {
                                let _token = ::tinc::__private::de::PathToken::push_key(key);
                                ::tinc::__private::de::TrackedStructDeserializer::<'de>::verify_deserialize(
                                    value,
                                    tracker,
                                )?;
                            }
                        }
                    }
                },
                None => quote! {
                    if let Some(tracker) = tracker.value.#field_name.as_mut() {
                        let mut _token = ::tinc::__private::de::PathToken::push_field(stringify!(#field_name));
                        ::tinc::__private::de::TrackedStructDeserializer::<'de>::verify_deserialize(
                            &mut self.#field_name,
                            tracker,
                        )?;
                    }
                },
            };

            verify_deserialize_fn.push(validation);
        }
    }

    let message_path = get_common_import_path(message.package.as_str(), message_key);
    let message_ident = message_path.segments.last().unwrap().ident.clone();

    prost.message_attribute(message_key, "#[derive(::tinc::__private::de::TincMessageTracker)]");

    let field_enum_impl = parse_quote! {
        const _: () = {
            #[derive(std::fmt::Debug, std::clone::Clone, core::marker::Copy)]
            #[allow(non_camel_case_types)]
            pub enum #field_enum_ident {
                #(#field_enum_variants),*
            }

            impl ::tinc::__private::de::StructField for #field_enum_ident {
                fn name(&self) -> &'static str {
                    #field_enum_ident::name(self)
                }
            }

            impl #field_enum_ident {
                pub const fn idx(&self) -> usize {
                    match self {
                        #(#field_enum_idx_fn)*
                    }
                }

                pub const fn name(&self) -> &'static str {
                    match self {
                        #(#field_enum_name_fn)*
                    }
                }
            }

            impl ::core::str::FromStr for #field_enum_ident {
                type Err = ();

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    ::tinc::__tinc_field_from_str!(s, #(#field_enum_from_str_fn)*)
                }
            }

            impl<'de> ::tinc::__private::de::TrackedStructDeserializer<'de> for #message_path {
                const NAME: &'static str = stringify!(#message_ident);
                const FIELDS: &'static [&'static str] = &[#(#deserializer_fields)*];

                type Field = #field_enum_ident;

                #[allow(unused_mut, dead_code)]
                fn deserialize<D>(
                    &mut self,
                    field: Self::Field,
                    mut tracker: &mut Self::Tracker,
                    deserializer: D,
                ) -> Result<(), D::Error>
                where
                    D: ::tinc::__private::de::DeserializeFieldValue<'de>,
                {
                    match field {
                        #(#deserializer_fn)*
                    }

                    ::core::result::Result::Ok(())
                }

                #[allow(unused_mut, dead_code)]
                fn verify_deserialize<E>(
                    &self,
                    mut tracker: &mut Self::Tracker,
                ) -> Result<(), E>
                where
                    E: ::tinc::reexports::serde::de::Error,
                {
                    #(#verify_deserialize_fn)*

                    ::core::result::Result::Ok(())
                }
            }

            impl ::tinc::__private::de::Expected for #message_path {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!(#message_ident))
                }
            }
        };
    };

    modules.entry(message.package.clone()).or_default().push(field_enum_impl);

    // // Process individual fields.
    // for (field_name, field) in &message.fields {
    //     if field
    //         .one_of
    //         .as_ref()
    //         .is_some_and(|oneof| message.oneofs.get(oneof).unwrap().opts.custom_impl.unwrap_or(false))
    //     {
    //         continue;
    //     }

    //     let name = field
    //         .opts
    //         .rename
    //         .as_ref()
    //         .or_else(|| message.opts.rename_all.is_none().then_some(&field.json_name));

    //     let field_key = if let Some(oneof) = &field.one_of {
    //         format!("{message_key}.{oneof}.{field_name}")
    //     } else {
    //         format!("{message_key}.{field_name}")
    //     };

    //     if let Some(name) = name {
    //         serde_rename(&field_key, name, prost);
    //     }

    //     with_attr(&field_key, &field.kind, field.nullable, field.omitable, prost);

    //     if field.omitable {
    //         // field_omitable(&field_key, prost);
    //     }

    //     field_visibility(&field_key, prost, field.visibility);
    // }

    Ok(())
}

pub(super) fn handle_enum(
    enum_key: &str,
    enum_: &EnumOpts,
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
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

    let enum_path = get_common_import_path(enum_.package.as_str(), enum_key);
    let enum_ident = enum_path.segments.last().unwrap().ident.clone();

    let enum_impl = parse_quote! {
        impl ::tinc::__private::de::Expected for #enum_path {
            fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an enum of `")?;
                write!(formatter, stringify!(#enum_ident))?;
                write!(formatter, "`")
            }
        }
    };

    modules.entry(enum_.package.clone()).or_default().push(enum_impl);

    Ok(())
}
