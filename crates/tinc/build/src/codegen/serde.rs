use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use quote::quote;
use syn::parse_quote;

use crate::codegen::get_common_import_path;
use crate::extensions::{EnumOpts, FieldKind, FieldModifier, FieldType, FieldVisibility, MessageOpts, WellKnownType};

use super::{field_ident_from_str, ident_from_str};

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

        let set_field_present = quote! {
            tracker.inner.set_field_present(&field)
        };

        let duplicate_check = quote! {
            if !#set_field_present {
                return Err(::tinc::reexports::serde::de::Error::duplicate_field(::tinc::__private::de::StructField::name(&field)));
            }
        };

        let field_name = field_ident_from_str(field_name);

        let enum_or_well_known_helper = |modifier: Option<FieldModifier>, nullable: bool, path: &syn::Path| {
            match (modifier, nullable) {
                (Some(FieldModifier::Optional), true) => quote! {
                    #duplicate_check
                    self.#field_name = ::tinc::__private::de::DeserializeFieldValue::deserialize::<#path>(
                        deserializer
                    )?.map(::core::convert::Into::into);
                },
                (Some(FieldModifier::Optional), false) => quote! {
                    #duplicate_check
                    self.#field_name = ::core::option::Option::Some(::core::convert::Into::into(
                        ::tinc::__private::de::DeserializeFieldValue::deserialize::<#path>(deserializer)?
                    ));
                },
                (Some(FieldModifier::Map), _) => quote! {
                    #set_field_present;
                    ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                        deserializer,
                        ::tinc::__private::de::tracker::map::MapDeserializer::new_with_helper(
                            &mut self.#field_name,
                            ::tinc::__private::de::tracker::Tracker {
                                inner: tracker.inner.push_child_map(&field)?,
                                shared: tracker.shared,
                            },
                            ::core::marker::PhantomData::<#path>,
                        ),
                    )?;
                },
                (Some(FieldModifier::List), _) => quote! {
                    #duplicate_check

                    ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                        deserializer,
                        ::tinc::__private::de::tracker::repeated::RepeatedDeserializer::new_with_helper(
                            &mut self.#field_name,
                            ::tinc::__private::de::tracker::Tracker {
                                inner: tracker.inner.push_child_repeated(&field)?,
                                shared: tracker.shared,
                            },
                            ::core::marker::PhantomData::<#path>,
                        ),
                    )?;
                },
                (None, _) => quote! {
                    #duplicate_check
                    self.#field_name = ::core::convert::Into::into(
                        ::tinc::__private::de::DeserializeFieldValue::deserialize::<#path>(deserializer)?
                    );
                },
            }
        };

        let deserialize_impl = match (field.kind.field_type(), field.kind.modifier(), field.nullable) {
            (FieldType::Enum(path), modifier, nullable) => {
                let path = get_common_import_path(message.package.as_str(), &path);
                let path_str = quote! { #path };
                prost.field_attribute(format!("{message_key}.{field_name}"), format!("#[serde(serialize_with = \"::tinc::__private::enum_::serialize::<{path_str}, _, _>\")]"));
                enum_or_well_known_helper(modifier, nullable, &path)
            }
            (FieldType::WellKnown(well_known), modifier, nullable) => {
                let path = match well_known {
                    WellKnownType::Duration => parse_quote! { ::tinc::__private::well_known::Duration },
                    WellKnownType::Empty => parse_quote! { ::tinc::__private::well_known::Empty },
                    WellKnownType::Timestamp => parse_quote! { ::tinc::__private::well_known::Timestamp },
                    WellKnownType::Any => parse_quote! { ::tinc::__private::well_known::Any },
                    WellKnownType::List => parse_quote! { ::tinc::__private::well_known::List },
                    WellKnownType::Struct => parse_quote! { ::tinc::__private::well_known::Struct },
                    WellKnownType::Value => parse_quote! { ::tinc::__private::well_known::Value },
                    _ => unreachable!("well-known type not supported: {:?}", well_known),
                };

                prost.field_attribute(format!("{message_key}.{field_name}"), "#[serde(serialize_with = \"::tinc::__private::well_known::serialize\")]");
                enum_or_well_known_helper(modifier, nullable, &path)
            }
            (FieldType::Message(_), Some(FieldModifier::Optional), true) => quote! {
                #set_field_present;
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::struct_::OptionalStructDeserializer::new(
                        &mut self.#field_name,
                        ::tinc::__private::de::StructField::name(&field),
                        ::tinc::__private::de::tracker::Tracker {
                            inner: tracker.inner.push_child_struct(&field)?,
                            shared: tracker.shared,
                        },
                    ),
                )?;
            },
            (FieldType::Message(_), Some(FieldModifier::Optional), false) => quote! {
                #set_field_present;
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::struct_::StructDeserializer::new(
                        self.#field_name.get_or_insert_default(),
                        ::tinc::__private::de::StructField::name(&field),
                        ::tinc::__private::de::tracker::Tracker {
                            inner: tracker.inner.push_child_struct(&field)?,
                            shared: tracker.shared,
                        },
                    ),
                )?;
            },
            (FieldType::Message(_), Some(FieldModifier::Map), _) => quote! {
                #set_field_present;
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::map_struct::MapStructDeserializer::new(
                        &mut self.#field_name,
                        ::tinc::__private::de::tracker::Tracker {
                        inner: tracker.inner.push_child_map_struct(&field)?,
                        shared: tracker.shared,
                        },
                    ),
                )?;
            },
            (FieldType::Message(_), Some(FieldModifier::List), _) => quote! {
                #duplicate_check
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::repeated_struct::RepeatedStructDeserializer::new(
                        &mut self.#field_name,
                        ::tinc::__private::de::tracker::Tracker {
                            inner: tracker.inner.push_child_repeated_struct(&field)?,
                            shared: tracker.shared,
                        },
                    ),
                )?;
            },
            (FieldType::Message(_), None, _) => quote! {
                #set_field_present;
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::struct_::StructDeserializer::new(
                        &mut self.#field_name,
                        ::tinc::__private::de::StructField::name(&field),
                        ::tinc::__private::de::tracker::Tracker {
                        inner: tracker.inner.push_child_struct(&field)?,
                        shared: tracker.shared,
                        },
                    ),
                )?;
            },
            (FieldType::Primitive(_), Some(FieldModifier::Optional), true) | (FieldType::Primitive(_), None, _) => quote! {
                #duplicate_check
                self.#field_name = ::tinc::__private::de::DeserializeFieldValue::deserialize(
                    deserializer
                )?;
            },
            (FieldType::Primitive(_), Some(FieldModifier::Optional), false) => quote! {
                #duplicate_check
                self.#field_name = ::core::option::Option::Some(
                    ::tinc::__private::de::DeserializeFieldValue::deserialize(deserializer)?
                );
            },
            (FieldType::Primitive(_), Some(FieldModifier::Map), _) => quote! {
                #set_field_present;
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::map::MapDeserializer::new(
                        &mut self.#field_name,
                        ::tinc::__private::de::tracker::Tracker {
                            inner: tracker.inner.push_child_map(&field)?,
                            shared: tracker.shared,
                        },
                    ),
                )?;
            },
            (FieldType::Primitive(_), Some(FieldModifier::List), _) => quote! {
                #duplicate_check
                ::tinc::__private::de::DeserializeFieldValue::deserialize_seed(
                    deserializer,
                    ::tinc::__private::de::tracker::repeated::RepeatedDeserializer::new(
                        &mut self.#field_name,
                        ::tinc::__private::de::tracker::Tracker {
                            inner: tracker.inner.push_child_repeated(&field)?,
                            shared: tracker.shared,
                        },
                    ),
                )?;
            },
        };

        deserializer_fn.push(quote! {
            #field_enum_ident::#ident => {
                #deserialize_impl
            }
        });

        if !field.omitable {
            verify_deserialize_fn.push(quote! {
                if !tracker.inner.get_field_presence(&#field_enum_ident::#ident) {
                    tracker.report_error(
                        ::core::option::Option::Some(
                            ::tinc::__private::de::tracker::ErrorLocation::StructField {
                                name: ::tinc::__private::de::StructField::name(&#field_enum_ident::#ident),
                            },
                        ),
                        ::tinc::reexports::serde::de::Error::missing_field(
                            ::tinc::__private::de::StructField::name(&#field_enum_ident::#ident),
                        ),
                    )?;
                }
            });
        }

        let common = quote! {
            ::tinc::__private::de::TrackedStructDeserializer::verify_deserialize(
                field,
                ::tinc::__private::de::tracker::Tracker {
                    inner,
                    shared: tracker.shared,
                },
            )?;
        };

        match (&field.kind, field.kind.inner()) {
            (FieldKind::Map(_, _), FieldKind::Message(_)) => verify_deserialize_fn.push(quote! {
                if let ::core::option::Option::Some(::tinc::__private::de::tracker::TrackerAny::MapStruct(map)) = tracker.inner.get_child(&#field_enum_ident::#ident) {
                    for (key, inner) in map.children.iter_mut() {
                        if let Some(field) = key.get_value(&self.#field_name) {
                            #common
                        }
                    }
                }
            }),
            (FieldKind::List(_), FieldKind::Message(_)) => verify_deserialize_fn.push(quote! {
                if let ::core::option::Option::Some(::tinc::__private::de::tracker::TrackerAny::RepeatedStruct(repeated)) = tracker.inner.get_child(&#field_enum_ident::#ident) {
                    for (inner, field) in ::core::iter::Iterator::zip(repeated.children.iter_mut(), self.#field_name.iter()) {
                        #common
                    }
                }
            }),
            (FieldKind::Optional(_), FieldKind::Message(_)) => verify_deserialize_fn.push(quote! {
                if let ::core::option::Option::Some(::tinc::__private::de::tracker::TrackerAny::Struct(inner)) = tracker.inner.get_child(&#field_enum_ident::#ident) {
                    if let ::core::option::Option::Some(field) = &self.#field_name {
                        #common
                    }
                }
            }),
            (FieldKind::Message(_), _) => verify_deserialize_fn.push(quote! {
                if let ::core::option::Option::Some(::tinc::__private::de::tracker::TrackerAny::Struct(inner)) = tracker.inner.get_child(&#field_enum_ident::#ident) {
                    let field = &self.#field_name;
                    #common
                }
            }),
            _ => {},
        }
    }

    let message_path = get_common_import_path(message.package.as_str(), message_key);
    let message_ident = message_path.segments.last().unwrap().ident.clone();

    let field_enum_impl = parse_quote! {
        const _: () = {
            #[derive(std::fmt::Debug, std::clone::Clone, core::marker::Copy)]
            #[allow(non_camel_case_types)]
            pub enum #field_enum_ident {
                #(#field_enum_variants),*
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

            impl ::tinc::__private::de::StructField for #field_enum_ident {
                fn idx(&self) -> usize {
                    #field_enum_ident::idx(self)
                }

                fn name(&self) -> &'static str {
                    #field_enum_ident::name(self)
                }

                fn from_str(s: &str) -> Option<Self> {
                    ::tinc::__tinc_field_from_str!(
                        s,
                        #(#field_enum_from_str_fn)*
                    )
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
                    mut tracker: ::tinc::__private::de::tracker::Tracker<'_, ::tinc::__private::de::tracker::struct_::TrackerStruct>,
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
                    mut tracker: ::tinc::__private::de::tracker::Tracker<'_, ::tinc::__private::de::tracker::struct_::TrackerStruct>,
                ) -> Result<(), E>
                where
                    E: ::tinc::reexports::serde::de::Error,
                {
                    #(#verify_deserialize_fn)*

                    ::core::result::Result::Ok(())
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
