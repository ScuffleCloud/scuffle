use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use quote::quote;
use syn::parse_quote;
use tinc_pb::schema_oneof_options::Tagged;

use super::ident_from_str;
use crate::codegen::get_common_import;
use crate::extensions::{EnumOpts, FieldKind, FieldVisibility, MessageOpts, PrimitiveKind};

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

fn with_attr(key: &str, mut field_kind: &FieldKind, nullable: bool, omitable: bool, prost: &mut tonic_build::Config) {
    // fn schemars_with(field_kind: &FieldKind, current_namespace: &str) -> Option<String> {
    //     match field_kind {
    //         FieldKind::WellKnown(well_known) => Some(well_known.path().to_owned()),
    //         FieldKind::Optional(inner) => Some(format!(
    //             "::core::option::Option<{}>",
    //             schemars_with(inner, current_namespace)?
    //         )),
    //         FieldKind::List(inner) => Some(format!("::std::vec::Vec<{}>", schemars_with(inner, current_namespace)?)),
    //         FieldKind::Map(key, inner) => Some(format!(
    //             "::std::collections::HashMap<{}, {}>",
    //             match key {
    //                 PrimitiveKind::Bytes => unimplemented!("map keys cannot be bytes"),
    //                 PrimitiveKind::F32 => unimplemented!("map keys cannot be f32"),
    //                 PrimitiveKind::F64 => unimplemented!("map keys cannot be f64"),
    //                 _ => key.path(),
    //             },
    //             schemars_with(inner, current_namespace)?
    //         )),
    //         FieldKind::Enum(name) => Some(get_common_import(current_namespace, name)),
    //         FieldKind::Primitive(_) => None,
    //         FieldKind::Message(_) => None,
    //     }
    // }

    let is_optional = matches!(field_kind, FieldKind::Optional(_));

    match field_kind.inner() {
        // Special handling for well-known types.
        FieldKind::WellKnown(_) => {
            prost.field_attribute(key, "#[serde(serialize_with = \"::tinc::helpers::well_known::serialize\")]");
            // let deserialize_fn = if is_optional && !nullable {
            //     "::tinc::helpers::well_known::deserialize_non_optional"
            // } else {
            //     "::tinc::helpers::well_known::deserialize"
            // };
            // prost.field_attribute(key, format!("#[serde(deserialize_with = \"{deserialize_fn}\")]"));
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
        // _ if is_optional && !nullable => {
        //     prost.field_attribute(
        //         key,
        //         "#[serde(deserialize_with = \"::tinc::helpers::deserialize_non_null_option\")]",
        //     );
        // }
        // _ if is_optional && !omitable => {
        //     prost.field_attribute(
        //         key,
        //         "#[serde(deserialize_with = \"::tinc::helpers::deserialize_non_omitable\")]",
        //     );
        // }
        _ => {}
    }

    if is_optional && (!nullable || !omitable) {
        field_kind = field_kind.strip_option();
        // prost.field_attribute(key, "#[schemars(required)]");
    }

    // if let Some(with) = schemars_with(field_kind, key) {
    //     prost.field_attribute(key, format!("#[schemars(with = \"{with}\")]"));
    // }

    // if nullable && !omitable {
    //     prost.field_attribute(key, "#[schemars(transform = ::tinc::helpers::schemars_non_omitable)]");
    // }
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
            #json_name => Some(#field_enum_ident::#ident),
        });
        deserializer_fields.push(quote! {
            #json_name,
        });

        let duplicate_check = quote! {
            if !tracker.inner.set_field_present(&field) {
                return Err(::tinc::reexports::serde::de::Error::duplicate_field(::tinc::de::StructField::name(&field)));
            }
        };

        let field_name = ident_from_str(field_name);

        let style = match (&field.kind, field.kind.inner(), field.nullable) {
            // Optional primitive
            (FieldKind::Optional(_), FieldKind::Primitive(_), false) => {
                quote! {
                    #duplicate_check
                    self.#field_name = ::core::option::Option::Some(deserializer.deserialize()?);
                }
            }
            (FieldKind::Optional(_), FieldKind::Enum(path), false) => {
                let path = syn::parse_str::<syn::Path>(&get_common_import(message.package.as_str(), path)).unwrap();

                quote! {
                    #duplicate_check
                    self.#field_name = ::core::option::Option::Some(::core::convert::Into::into(deserializer.deserialize::<#path>()?));
                }
            }
            // List of primitive or enum.
            (FieldKind::List(_), FieldKind::Primitive(_), _) => quote! {
                #duplicate_check
                deserializer.deserialize_seed(tinc::de::tracker::repeated::RepeatedDeserializer::new(
                    &mut self.#field_name,
                    ::tinc::de::tracker::Tracker {
                        inner: tracker.inner.push_child_repeated(&field)?,
                        shared: tracker.shared,
                    },
                ))?;
            },
            (FieldKind::List(_), FieldKind::Enum(path), _) => {
                let path = syn::parse_str::<syn::Path>(&get_common_import(message.package.as_str(), path)).unwrap();

                quote! {
                    #duplicate_check
                    deserializer.deserialize_seed(tinc::de::tracker::repeated::RepeatedDeserializer::new_enum(
                        &mut self.#field_name,
                        ::tinc::de::tracker::Tracker {
                            inner: tracker.inner.push_child_repeated(&field)?,
                            shared: tracker.shared,
                        },
                        ::core::marker::PhantomData::<#path>,
                    ))?;
                }
            }
            // Map of primitive or enum.
            (FieldKind::Map(_, _), FieldKind::Primitive(_), _) => quote! {
                deserializer.deserialize_seed(tinc::de::tracker::map::MapDeserializer::new(
                    &mut self.#field_name,
                    ::tinc::de::tracker::Tracker {
                        inner: tracker.inner.push_child_map(&field)?,
                        shared: tracker.shared,
                    },
                ))?;
            },
            (FieldKind::Map(_, _), FieldKind::Enum(path), _) => {
                let path = syn::parse_str::<syn::Path>(&get_common_import(message.package.as_str(), path)).unwrap();

                quote! {
                    deserializer.deserialize_seed(tinc::de::tracker::map::MapDeserializer::new_enum(
                        &mut self.#field_name,
                        ::tinc::de::tracker::Tracker {
                            inner: tracker.inner.push_child_map(&field)?,
                            shared: tracker.shared,
                        },
                        ::core::marker::PhantomData::<#path>,
                    ))?;
                }
            }
            (FieldKind::Optional(_) | FieldKind::Primitive(_), FieldKind::Primitive(_), _) => {
                quote! {
                    #duplicate_check
                    self.#field_name = deserializer.deserialize()?;
                }
            }
            (FieldKind::Optional(_), FieldKind::Enum(path), _) => {
                let path = syn::parse_str::<syn::Path>(&get_common_import(message.package.as_str(), path)).unwrap();

                quote! {
                    #duplicate_check
                    self.#field_name = deserializer.deserialize::<::core::option::Option<#path>>()?.map(::core::convert::Into::into);
                }
            }
            (FieldKind::Enum(_), FieldKind::Enum(path), _) => {
                let path = syn::parse_str::<syn::Path>(&get_common_import(message.package.as_str(), path)).unwrap();

                quote! {
                    #duplicate_check
                    self.#field_name = deserializer.deserialize::<#path>()?.into();
                }
            }
            // Optional message.
            (FieldKind::Optional(_), FieldKind::Message(_), true) => quote! {
                deserializer.deserialize_seed(tinc::de::tracker::struct_::OptionalStructDeserializer::new(
                    &mut self.#field_name,
                    ::tinc::de::StructField::name(&field),
                    ::tinc::de::tracker::Tracker {
                        inner: tracker.inner.push_child_struct(&field)?,
                        shared: tracker.shared,
                    },
                ))?;
            },
            (FieldKind::Optional(_), FieldKind::Message(_), false) => quote! {
                #duplicate_check
                deserializer.deserialize_seed(tinc::de::tracker::struct_::StructDeserializer::new(
                    self.#field_name.get_or_insert_default(),
                    ::tinc::de::tracker::Tracker {
                        inner: tracker.inner.push_child_struct(&field)?,
                        shared: tracker.shared,
                    },
                ))?;
            },
            // List of message.
            (FieldKind::List(_), FieldKind::Message(_), _) => quote! {
                #duplicate_check
                deserializer.deserialize_seed(tinc::de::tracker::repeated_struct::RepeatedStructDeserializer::new(
                    &mut self.#field_name,
                    ::tinc::de::tracker::Tracker {
                        inner: tracker.inner.push_child_repeated_struct(&field)?,
                        shared: tracker.shared,
                    },
                ))?;
            },
            // Map of message.
            (FieldKind::Map(_, _), FieldKind::Message(_), _) => quote! {
                deserializer.deserialize_seed(tinc::de::tracker::map_struct::MapStructDeserializer::new(
                    &mut self.#field_name,
                    ::tinc::de::tracker::Tracker {
                        inner: tracker.inner.push_child_map_struct(&field)?,
                        shared: tracker.shared,
                    },
                ))?;
            },
            // Well-known type.
            (_, FieldKind::WellKnown(_), _) => quote! {},
            _ => unimplemented!("unsupported field kind: {:?}", field.kind),
        };

        deserializer_fn.push(quote! {
            #field_enum_ident::#ident => {
                #style
            }
        });
    }

    let message_path = syn::parse_str::<syn::Path>(&get_common_import(message.package.as_str(), message_key)).unwrap();
    let message_ident = message_path.segments.last().unwrap().ident.clone();

    let field_enum_impl = parse_quote! {
        const _: () = {
            #[derive(std::fmt::Debug, std::clone::Clone, core::marker::Copy)]
            #[allow(non_camel_case_types)]
            pub enum #field_enum_ident {
                #(#field_enum_variants),*
            }

            impl ::tinc::de::StructField for #field_enum_ident {
                fn idx(&self) -> usize {
                    match self {
                        #(#field_enum_idx_fn)*
                    }
                }

                fn name(&self) -> &'static str {
                    match self {
                        #(#field_enum_name_fn)*
                    }
                }

                fn from_str(s: &str) -> Option<Self> {
                    match s {
                        #(#field_enum_from_str_fn)*
                        _ => None,
                    }
                }
            }

            impl<'de> ::tinc::de::TrackedStructDeserializer<'de> for #message_path {
                const NAME: &'static str = stringify!(#message_ident);
                const FIELDS: &'static [&'static str] = &[#(#deserializer_fields)*];

                type Field = #field_enum_ident;

                #[inline]
                fn deserialize<D>(
                    &mut self,
                    field: Self::Field,
                    tracker: &mut ::tinc::de::tracker::Tracker<'_, ::tinc::de::tracker::struct_::TrackerStruct>,
                    deserializer: D,
                ) -> Result<(), D::Error>
                where
                    D: ::tinc::de::DeserializeFieldValue<'de>,
                {
                    match field {
                        #(#deserializer_fn)*
                    }

                    Ok(())
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
