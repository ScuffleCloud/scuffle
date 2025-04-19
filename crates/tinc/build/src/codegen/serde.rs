use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use quote::quote;
use syn::parse_quote;
use tinc_pb::schema_oneof_options::Tagged;

use super::{field_ident_from_str, ident_from_str, strip_last_path_part, type_ident_from_str};
use crate::codegen::get_common_import_path;
use crate::extensions::{EnumOpts, FieldModifier, FieldOpts, FieldType, FieldVisibility, MessageOpts, OneofOpts};

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
    } else {
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
        prost.enum_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
    }

    prost.enum_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
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

fn rename_all(key: &str, style: Option<tinc_pb::RenameAll>, prost: &mut tonic_build::Config, is_enum: bool) -> bool {
    if let Some(style) = style.and_then(rename_all_to_serde_rename_all) {
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

fn rename_field(field: &str, style: tinc_pb::RenameAll) -> Option<String> {
    match style {
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

fn handle_oneof(
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
    message_key: &str,
    message: &MessageOpts,
    oneof: &OneofOpts,
    name: &str,
) -> anyhow::Result<()> {
    let oneof_key = format!("{message_key}.{name}",);

    if message.custom_impl {
        if let Some(rename) = &oneof.rename {
            serde_rename(&oneof_key, rename, prost);
        }
    }

    if oneof.custom_impl {
        return Ok(());
    }

    prost.enum_attribute(&oneof_key, "#[derive(::tinc::reexports::serde::Serialize)]");
    prost.enum_attribute(&oneof_key, "#[derive(::tinc::__private::de::Tracker)]");
    prost.field_attribute(&oneof_key, "#[tinc(oneof)]");
    rename_all(&oneof_key, oneof.rename_all, prost, true);

    let oneof_path = get_common_import_path(&message.package, &oneof_key);
    let oneof_ident = oneof_path.segments.last().unwrap().ident.clone();

    let variant_identifier_ident = ident_from_str("___identifier");
    let mut oneof_identifier_for_ident = variant_identifier_ident.clone();
    let mut variant_idents = Vec::new();
    let mut variant_idx_fn = Vec::new();
    let mut variant_name_fn = Vec::new();
    let mut variant_from_str_fn = Vec::new();
    let mut variant_fields = Vec::new();
    let mut variant_enum_ident = Vec::new();
    let mut deserializer_impl = Vec::new();
    let mut validation_impl = Vec::new();

    let tagged_impl = if let Some(Tagged { tag, content }) = &oneof.tagged {
        prost.enum_attribute(&oneof_key, format!("#[serde(tag = \"{tag}\", content = \"{content}\")]"));
        prost.enum_attribute(&oneof_key, "#[tinc(tagged)]");
        oneof_identifier_for_ident = ident_from_str("___tagged_identifier");
        quote! {
            #[derive(
                ::std::fmt::Debug,
                ::std::clone::Clone,
                ::core::marker::Copy,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::hash::Hash,
                ::core::cmp::Ord,
                ::core::cmp::PartialOrd,
            )]
            #[allow(non_camel_case_types)]
            pub enum #oneof_identifier_for_ident {
                ___tag,
                ___content,
            }

            impl ::tinc::__private::de::Identifier for #oneof_identifier_for_ident {
                const OPTIONS: &'static [&'static str] = &[
                    #tag,
                    #content,
                ];

                fn name(&self) -> &'static str {
                    match self {
                        #oneof_identifier_for_ident::___tag => #tag,
                        #oneof_identifier_for_ident::___content => #content,
                    }
                }
            }

            impl ::core::str::FromStr for #oneof_identifier_for_ident {
                type Err = ();

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    ::tinc::__tinc_field_from_str!(s,
                        #tag => #oneof_identifier_for_ident::___tag,
                        #content => #oneof_identifier_for_ident::___content,
                    )
                }
            }

            impl ::tinc::__private::de::TaggedOneOfIdentifier for #oneof_identifier_for_ident {
                const TAG: Self = #oneof_identifier_for_ident::___tag;
                const CONTENT: Self = #oneof_identifier_for_ident::___content;
            }
        }
    } else {
        quote! {}
    };

    oneof.fields.iter().enumerate().for_each(|(idx, (name, field))| {
        let ident = ident_from_str(format!("__field_{idx}"));
        let json_name = field
            .rename
            .clone()
            .or_else(|| rename_field(name, oneof.rename_all?))
            .unwrap_or_else(|| name.clone());

        variant_idents.push(ident.clone());
        variant_idx_fn.push(quote! {
            #variant_identifier_ident::#ident => #idx
        });
        variant_name_fn.push(quote! {
            #variant_identifier_ident::#ident => #json_name
        });
        variant_from_str_fn.push(quote! {
            #json_name => #variant_identifier_ident::#ident
        });
        variant_fields.push(quote! {
            #json_name
        });
        let enum_ident = type_ident_from_str(name);
        variant_enum_ident.push(enum_ident.clone());
        deserializer_impl.push(quote! {
            #variant_identifier_ident::#ident => {
                let tracker = match tracker {
                    ::core::option::Option::None => {
                        let ___Tracker::#enum_ident(tracker) = tracker.get_or_insert_with(|| ___Tracker::#enum_ident(Default::default())) else {
                            ::core::unreachable!()
                        };

                        tracker
                    },
                    ::core::option::Option::Some(___Tracker::#enum_ident(tracker)) => {
                        if !::tinc::__private::de::tracker_allow_duplicates(Some(tracker)) {
                            return ::tinc::__private::de::report_error(
                                ::tinc::__private::de::TrackedError::duplicate_field(),
                            );
                        }

                        tracker
                    },
                    ::core::option::Option::Some(tracker) => {
                        return ::core::result::Result::Err(
                            ::tinc::reexports::serde::de::Error::invalid_type(
                                ::tinc::reexports::serde::de::Unexpected::Other(
                                    ::tinc::__private::de::Identifier::name(&___tracker_to_identifier(tracker)),
                                ),
                                &::tinc::__private::de::Identifier::name(&#variant_identifier_ident::#ident),
                            ),
                        );
                    }
                };

                let value = match value.get_or_insert_with(|| Self::#enum_ident(Default::default())) {
                    Self::#enum_ident(value) => value,
                    value => {
                        return ::core::result::Result::Err(
                            ::tinc::reexports::serde::de::Error::invalid_type(
                                ::tinc::reexports::serde::de::Unexpected::Other(
                                    ::tinc::__private::de::Identifier::name(&___value_to_identifier(value)),
                                ),
                                &::tinc::__private::de::Identifier::name(&#variant_identifier_ident::#ident),
                            ),
                        );
                    }
                };

                ::tinc::__private::de::TrackerDeserializer::deserialize(
                    tracker,
                    value,
                    deserializer,
                )?;
            }
        });

        validation_impl.push(quote! {
            (Self::#enum_ident(value), ___Tracker::#enum_ident(tracker)) => {
                ::tinc::__private::de::TrackerValidation::validate(tracker, value)?;
            }
        });

        if let FieldType::Enum(path) = field.kind.field_type() {
            let path = get_common_import_path(message_key, &path);
            let path_str = quote! { #path };
            let field_key = format!("{oneof_key}.{name}");
            prost.field_attribute(&field_key, format!("#[serde(serialize_with = \"::tinc::__private::enum_::serialize::<{path_str}, _, _>\")]"));
            prost.field_attribute(&field_key, format!("#[tinc(enum = \"{path_str}\")]"));
        }
    });

    modules.entry(message.package.clone()).or_default().push(parse_quote! {
        const _: () = {
            #tagged_impl

            #[derive(
                ::std::fmt::Debug,
                ::std::clone::Clone,
                ::core::marker::Copy,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::hash::Hash,
                ::core::cmp::Ord,
                ::core::cmp::PartialOrd,
            )]
            #[allow(non_camel_case_types)]
            pub enum #variant_identifier_ident {
                #(#variant_idents),*
            }

            impl ::tinc::__private::de::Identifier for #variant_identifier_ident {
                const OPTIONS: &'static [&'static str] = &[#(#variant_fields),*];

                fn name(&self) -> &'static str {
                    match self {
                        #(#variant_name_fn),*
                    }
                }
            }

            impl ::core::str::FromStr for #variant_identifier_ident {
                type Err = ();

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    ::tinc::__tinc_field_from_str!(s, #(#variant_from_str_fn),*)
                }
            }

            impl ::tinc::__private::de::Expected for #oneof_path {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!(#oneof_ident))
                }
            }

            impl ::tinc::__private::de::IdentifierFor for #oneof_path {
                const NAME: &'static str = stringify!(#oneof_ident);
                type Identifier = #oneof_identifier_for_ident;
            }

            impl ::tinc::__private::de::TrackedOneOfVariant for #oneof_path {
                type Variant = #variant_identifier_ident;
            }

            type ___Tracker = <<#oneof_path as ::tinc::__private::de::TrackerFor>::Tracker as ::tinc::__private::de::TrackerWrapper>::Tracker;

            fn ___tracker_to_identifier(v: &___Tracker) -> #variant_identifier_ident {
                match v {
                    #(___Tracker::#variant_enum_ident(_) => #variant_identifier_ident::#variant_idents),*
                }
            }

            fn ___value_to_identifier(v: &#oneof_path) -> #variant_identifier_ident {
                match v {
                    #(#oneof_path::#variant_enum_ident(_) => #variant_identifier_ident::#variant_idents),*
                }
            }

            impl<'de> ::tinc::__private::de::TrackedOneOfDeserializer<'de> for #oneof_path {
                fn deserialize<D>(
                    value: &mut ::core::option::Option<#oneof_path>,
                    variant: #variant_identifier_ident,
                    tracker: &mut ::core::option::Option<___Tracker>,
                    deserializer: D,
                ) -> ::core::result::Result<(), D::Error>
                where
                    D: ::tinc::__private::de::DeserializeContent<'de>
                {
                    match variant {
                        #(#deserializer_impl),*
                    }

                    ::core::result::Result::Ok(())
                }

                fn validate<E>(&self, tracker: &mut <Self::Tracker as ::tinc::__private::de::TrackerWrapper>::Tracker) -> Result<(), E>
                where
                    E: serde::de::Error
                {
                    match (self, tracker) {
                        #(#validation_impl),*
                        (_, tracker) => {
                            return ::core::result::Result::Err(
                                ::tinc::reexports::serde::de::Error::custom(format!(
                                    "tracker and value do not match: {:?} != {:?}",
                                    ::tinc::__private::de::Identifier::name(&___tracker_to_identifier(tracker)),
                                    ::tinc::__private::de::Identifier::name(&___value_to_identifier(self)),
                                )),
                            );
                        }
                    }

                    ::core::result::Result::Ok(())
                }
            }
        };
    });

    Ok(())
}

struct FieldBuilder<'a> {
    deserializer_fields: &'a mut Vec<proc_macro2::TokenStream>,
    field_enum_variants: &'a mut Vec<proc_macro2::TokenStream>,
    field_enum_name_fn: &'a mut Vec<proc_macro2::TokenStream>,
    field_enum_from_str_fn: &'a mut Vec<proc_macro2::TokenStream>,
    field_enum_from_str_flattened_fn: &'a mut Vec<proc_macro2::TokenStream>,
    deserializer_fn: &'a mut Vec<proc_macro2::TokenStream>,
    verify_deserialize_fn: &'a mut Vec<proc_macro2::TokenStream>,
}

fn handle_message_field(
    prost: &mut tonic_build::Config,
    message: &MessageOpts,
    message_key: &str,
    field: &FieldOpts,
    name: &str,
    builder: FieldBuilder<'_>,
    field_enum_ident: &syn::Ident,
) -> anyhow::Result<()> {
    let json_name = field
        .rename
        .clone()
        .or_else(|| rename_field(name, message.rename_all?))
        .unwrap_or_else(|| name.to_owned());

    let ident = ident_from_str(format!("__field_{name}"));
    if field.flatten {
        let key = match field.kind.field_type() {
            FieldType::Message(key) => key,
            FieldType::OneOf(key) => key,
            _ => anyhow::bail!("flattened fields must be messages or oneofs"),
        };

        anyhow::ensure!(
            matches!(field.kind.modifier(), Some(FieldModifier::Optional) | None),
            "flattened fields cannot be lists or maps"
        );

        let message_path = get_common_import_path(message.package.as_str(), &key);

        let flattened_identifier = quote! {
            <#message_path as ::tinc::__private::de::IdentifierFor>::Identifier
        };

        builder.deserializer_fields.push(quote! {
            <#flattened_identifier as ::tinc::__private::de::Identifier>::OPTIONS
        });
        builder.field_enum_variants.push(quote! {
            #ident(#flattened_identifier)
        });
        builder.field_enum_name_fn.push(quote! {
            #field_enum_ident::#ident(flatten) => ::tinc::__private::de::Identifier::name(flatten)
        });
        builder.field_enum_from_str_flattened_fn.push(quote! {
            #ident
        });
    } else {
        builder.deserializer_fields.push(quote! {
            &[#json_name]
        });
        builder.field_enum_variants.push(quote! {
            #ident
        });
        builder.field_enum_name_fn.push(quote! {
            #field_enum_ident::#ident => #json_name
        });
        builder.field_enum_from_str_fn.push(quote! {
            #json_name => #field_enum_ident::#ident
        });
    }

    let field_name = field_ident_from_str(name);

    match field.kind.field_type() {
        FieldType::Enum(path) => {
            let path = get_common_import_path(strip_last_path_part(message_key), &path);
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
        &mut tracker.#field_name
    };

    let mut value = quote! {
        &mut self.#field_name
    };

    // When a field is not nullable but prost generates an option<T>, we need to
    // remove the option before deserializing otherwise null will be a valid input.
    if matches!(field.kind.modifier(), Some(FieldModifier::Optional))
        && (!field.nullable || field.flatten)
        && !matches!(field.kind.field_type(), FieldType::OneOf(_))
    {
        tracker = quote! {
            (#tracker).get_or_insert_default()
        };

        value = quote! {
            (#value).get_or_insert_default()
        };
    }

    if field.flatten {
        builder.deserializer_fn.push(quote! {
            #field_enum_ident::#ident(field) => {
                let _token = ::tinc::__private::de::PathAllowerToken::push(#name)?;
                ::tinc::__private::de::TrackerDeserializeIdentifier::<'de>::deserialize(
                    (#tracker).get_or_insert_default(),
                    #value,
                    field,
                    deserializer,
                )?;
            }
        });
    } else {
        builder.deserializer_fn.push(quote! {
            #field_enum_ident::#ident => {
                let _token = ::tinc::__private::de::PathAllowerToken::push(#name)?;
                let tracker = #tracker;

                if !::tinc::__private::de::tracker_allow_duplicates(tracker.as_ref()) {
                    return ::tinc::__private::de::report_error(
                        ::tinc::__private::de::TrackedError::duplicate_field(),
                    );
                }

                ::tinc::__private::de::TrackerDeserializer::deserialize(
                    tracker.get_or_insert_default(),
                    #value,
                    deserializer,
                )?;
            }
        });
    }

    if !field.omitable && !field.flatten {
        builder.verify_deserialize_fn.push(quote! {
            if tracker.#field_name.is_none() {
                let _token = ::tinc::__private::de::PathToken::push_field(
                    ::tinc::__private::de::Identifier::name(&#field_enum_ident::#ident),
                );
                ::tinc::__private::de::report_error(
                    ::tinc::__private::de::TrackedError::missing_field(),
                )?;
            }
        });
    }

    let push_field_token = if field.flatten {
        quote! {}
    } else {
        quote! {
            let _token = ::tinc::__private::de::PathToken::push_field(
                ::tinc::__private::de::Identifier::name(&#field_enum_ident::#ident),
            );
        }
    };

    if matches!(field.kind.field_type(), FieldType::Message(_)) {
        let validation = match (field.kind.modifier(), field.flatten) {
            (Some(FieldModifier::List), _) => quote! {
                if let Some(trackers) = tracker.#field_name.as_mut() {
                    #push_field_token
                    for (idx, (value, tracker)) in self.#field_name.iter().zip(trackers.iter_mut()).enumerate() {
                        let _token = ::tinc::__private::de::PathToken::push_index(idx);
                        ::tinc::__private::de::TrackerValidation::validate(tracker, value)?;
                    }
                }
            },
            (Some(FieldModifier::Map), _) => quote! {
                if let Some(trackers) = tracker.#field_name.as_mut() {
                    #push_field_token
                    for (key, tracker) in trackers.iter_mut() {
                        if let Some(value) = self.#field_name.get(key) {
                            let _token = ::tinc::__private::de::PathToken::push_key(key);
                            ::tinc::__private::de::TrackerValidation::validate(tracker, value)?;
                        }
                    }
                }
            },
            (Some(FieldModifier::Optional), _) => quote! {
                match (self.#field_name.as_ref(), tracker.#field_name.as_mut().and_then(|tracker| tracker.as_mut())) {
                    (Some(value), Some(tracker)) => {
                        #push_field_token
                        ::tinc::__private::de::TrackerValidation::validate(tracker, value)?;
                    },
                    (None, Some(_)) => {
                        return Err(::serde::de::Error::custom("tracker is Some while value is None"));
                    },
                    (Some(_), None) => {
                        return Err(::serde::de::Error::custom("tracker is None while value is set"));
                    },
                    (None, None) => {}
                }
            },
            (None, false) => quote! {
                if let Some(tracker) = tracker.#field_name.as_mut() {
                    #push_field_token
                    ::tinc::__private::de::TrackerValidation::validate(
                        tracker,
                        &self.#field_name,
                    )?;
                }
            },
            (None, true) => quote! {{
                if let Some(tracker) = tracker.#field_name.as_mut() {
                    #push_field_token
                    ::tinc::__private::de::TrackerValidation::validate(
                        &self.#field_name,
                        tracker,
                    )?;
                }
            }},
        };

        builder.verify_deserialize_fn.push(validation);
    }

    Ok(())
}

pub(super) fn handle_message(
    message_key: &str,
    message: &MessageOpts,
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    // Process oneof fields.
    for (oneof_name, oneof) in &message.oneofs {
        handle_oneof(prost, modules, message_key, message, oneof, oneof_name)?;
    }

    if message.custom_impl {
        return Ok(());
    }

    message_attributes(message_key, prost);
    rename_all(message_key, message.rename_all, prost, false);

    let field_enum_ident = ident_from_str("___field_enum");

    let mut field_enum_variants = Vec::new();
    let mut field_enum_name_fn = Vec::new();
    let mut field_enum_from_str_fn = Vec::new();
    let mut field_enum_from_str_flattened_fn = Vec::new();
    let mut deserializer_fields = Vec::new();
    let mut deserializer_fn = Vec::new();
    let mut verify_deserialize_fn = Vec::new();

    for (field_name, field) in message.fields.iter() {
        handle_message_field(
            prost,
            message,
            message_key,
            field,
            field_name,
            FieldBuilder {
                deserializer_fields: &mut deserializer_fields,
                field_enum_variants: &mut field_enum_variants,
                field_enum_name_fn: &mut field_enum_name_fn,
                field_enum_from_str_fn: &mut field_enum_from_str_fn,
                field_enum_from_str_flattened_fn: &mut field_enum_from_str_flattened_fn,
                deserializer_fn: &mut deserializer_fn,
                verify_deserialize_fn: &mut verify_deserialize_fn,
            },
            &field_enum_ident,
        )?;
    }
    let message_path = get_common_import_path(message.package.as_str(), message_key);
    let message_ident = message_path.segments.last().unwrap().ident.clone();

    prost.message_attribute(message_key, "#[derive(::tinc::__private::de::Tracker)]");

    let field_enum_impl = parse_quote! {
        const _: () = {
            #[derive(
                ::std::fmt::Debug,
                ::std::clone::Clone,
                ::core::marker::Copy,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::hash::Hash,
                ::core::cmp::Ord,
                ::core::cmp::PartialOrd,
            )]
            #[allow(non_camel_case_types)]
            pub enum #field_enum_ident {
                #(#field_enum_variants),*
            }

            impl ::tinc::__private::de::Identifier for #field_enum_ident {
                const OPTIONS: &'static [&'static str] = ::tinc::__private_const_concat_str_array!(#(#deserializer_fields),*);

                fn name(&self) -> &'static str {
                    match self {
                        #(#field_enum_name_fn),*
                    }
                }
            }

            impl ::core::str::FromStr for #field_enum_ident {
                type Err = ();

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    ::tinc::__tinc_field_from_str!(s, #(#field_enum_from_str_fn),*, flattened: [#(#field_enum_from_str_flattened_fn),*])
                }
            }

            impl ::tinc::__private::de::IdentifierFor for #message_path {
                const NAME: &'static str = stringify!(#message_ident);
                type Identifier = #field_enum_ident;
            }

            impl<'de> ::tinc::__private::de::TrackedStructDeserializer<'de> for #message_path {
                #[allow(unused_mut, dead_code)]
                fn deserialize<D>(
                    &mut self,
                    field: Self::Identifier,
                    mut tracker: &mut <Self::Tracker as ::tinc::__private::de::TrackerWrapper>::Tracker,
                    deserializer: D,
                ) -> Result<(), D::Error>
                where
                    D: ::tinc::__private::de::DeserializeContent<'de>,
                {
                    match field {
                        #(#deserializer_fn),*
                    }

                    ::core::result::Result::Ok(())
                }

                #[allow(unused_mut, dead_code)]
                fn validate<E>(
                    &self,
                    mut tracker: &mut <Self::Tracker as ::tinc::__private::de::TrackerWrapper>::Tracker,
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

    Ok(())
}

pub(super) fn handle_enum(
    enum_key: &str,
    enum_: &EnumOpts,
    prost: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    if enum_.custom_impl {
        return Ok(());
    }

    enum_attributes(enum_key, prost, enum_.repr_enum);
    if !enum_.repr_enum {
        rename_all(enum_key, Some(enum_.rename_all), prost, true);
    }

    for (variant, variant_opts) in &enum_.variants {
        let variant_key = format!("{enum_key}.{variant}");

        if !enum_.repr_enum {
            if let Some(rename) = &variant_opts.rename {
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
