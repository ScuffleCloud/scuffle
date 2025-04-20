use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use quote::quote;
use syn::parse_quote;
use tinc_pb::schema_oneof_options::Tagged;

use super::prost_sanatize::{strip_enum_prefix, to_upper_camel};
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

fn serde_rename(key: &str, name: &str, prost: &mut tonic_build::Config) {
    prost.field_attribute(key, format!("#[serde(rename = \"{name}\")]"));
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
    prost.enum_attribute(&oneof_key, "#[derive(::tinc::__private::Tracker)]");
    prost.field_attribute(&oneof_key, "#[tinc(oneof)]");

    let oneof_path = get_common_import_path(&message.package, &oneof_key);
    let oneof_ident = oneof_path.segments.last().unwrap().ident.clone();

    let variant_identifier_ident = ident_from_str("___identifier");
    let mut oneof_identifier_for_ident = variant_identifier_ident.clone();
    let mut variant_idents = Vec::new();
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

            impl ::tinc::__private::Identifier for #oneof_identifier_for_ident {
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

            impl ::tinc::__private::TaggedOneOfIdentifier for #oneof_identifier_for_ident {
                const TAG: Self = #oneof_identifier_for_ident::___tag;
                const CONTENT: Self = #oneof_identifier_for_ident::___content;
            }
        }
    } else {
        quote! {}
    };

    oneof.fields.iter().for_each(|(name, field)| {
        let ident = ident_from_str(format!("__field_{name}"));
        let json_name = field
            .rename
            .clone()
            .or_else(|| rename_field(name, oneof.rename_all?))
            .unwrap_or_else(|| name.clone());

        prost.field_attribute(format!("{oneof_key}.{name}"), format!("#[serde(rename = \"{json_name}\")]"));
        if !field.has_input() {
            prost.field_attribute(format!("{oneof_key}.{name}"), "#[serde(skip_serializing)]");
        }

        if field.has_output() {
            variant_idents.push(ident.clone());
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
                            if !::tinc::__private::tracker_allow_duplicates(Some(tracker)) {
                                return ::tinc::__private::report_error(
                                    ::tinc::__private::TrackedError::duplicate_field(),
                                );
                            }

                            tracker
                        },
                        ::core::option::Option::Some(tracker) => {
                            return ::core::result::Result::Err(
                                ::tinc::reexports::serde::de::Error::invalid_type(
                                    ::tinc::reexports::serde::de::Unexpected::Other(
                                        ::tinc::__private::Identifier::name(&___tracker_to_identifier(tracker)),
                                    ),
                                    &::tinc::__private::Identifier::name(&#variant_identifier_ident::#ident),
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
                                        ::tinc::__private::Identifier::name(&___value_to_identifier(value)),
                                    ),
                                    &::tinc::__private::Identifier::name(&#variant_identifier_ident::#ident),
                                ),
                            );
                        }
                    };

                    ::tinc::__private::TrackerDeserializer::deserialize(
                        tracker,
                        value,
                        deserializer,
                    )?;
                }
            });

            validation_impl.push(quote! {
                (Self::#enum_ident(value), ___Tracker::#enum_ident(tracker)) => {
                    ::tinc::__private::TrackerValidation::validate(tracker, value)?;
                }
            });
        }

        if let FieldType::Enum(path) = field.kind.field_type() {
            let path = get_common_import_path(message_key, &path);
            let path_str = quote! { #path };
            let field_key = format!("{oneof_key}.{name}");
            prost.field_attribute(&field_key, format!("#[serde(serialize_with = \"::tinc::__private::serialize_enum::<{path_str}, _, _>\")]"));
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

            impl ::tinc::__private::Identifier for #variant_identifier_ident {
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

            impl ::tinc::__private::Expected for #oneof_path {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!(#oneof_ident))
                }
            }

            impl ::tinc::__private::IdentifierFor for #oneof_path {
                const NAME: &'static str = stringify!(#oneof_ident);
                type Identifier = #oneof_identifier_for_ident;
            }

            impl ::tinc::__private::TrackedOneOfVariant for #oneof_path {
                type Variant = #variant_identifier_ident;
            }

            type ___Tracker = <<#oneof_path as ::tinc::__private::TrackerFor>::Tracker as ::tinc::__private::TrackerWrapper>::Tracker;

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

            impl<'de> ::tinc::__private::TrackedOneOfDeserializer<'de> for #oneof_path {
                fn deserialize<D>(
                    value: &mut ::core::option::Option<#oneof_path>,
                    variant: #variant_identifier_ident,
                    tracker: &mut ::core::option::Option<___Tracker>,
                    deserializer: D,
                ) -> ::core::result::Result<(), D::Error>
                where
                    D: ::tinc::__private::DeserializeContent<'de>
                {
                    match variant {
                        #(#deserializer_impl),*
                    }

                    ::core::result::Result::Ok(())
                }

                fn validate<E>(&self, tracker: &mut <Self::Tracker as ::tinc::__private::TrackerWrapper>::Tracker) -> Result<(), E>
                where
                    E: ::tinc::reexports::serde::de::Error
                {
                    match (self, tracker) {
                        #(#validation_impl),*
                        (_, tracker) => {
                            return ::core::result::Result::Err(
                                ::tinc::reexports::serde::de::Error::custom(format!(
                                    "tracker and value do not match: {:?} != {:?}",
                                    ::tinc::__private::Identifier::name(&___tracker_to_identifier(tracker)),
                                    ::tinc::__private::Identifier::name(&___value_to_identifier(self)),
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

    let field_key = format!("{message_key}.{name}");

    prost.field_attribute(&field_key, format!("#[serde(rename = \"{json_name}\")]"));

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

        prost.field_attribute(&field_key, "#[serde(flatten)]");

        if field.has_input() {
            let flattened_identifier = quote! {
                <#message_path as ::tinc::__private::IdentifierFor>::Identifier
            };

            builder.deserializer_fields.push(quote! {
                <#flattened_identifier as ::tinc::__private::Identifier>::OPTIONS
            });
            builder.field_enum_variants.push(quote! {
                #ident(#flattened_identifier)
            });
            builder.field_enum_name_fn.push(quote! {
                #field_enum_ident::#ident(flatten) => ::tinc::__private::Identifier::name(flatten)
            });
            builder.field_enum_from_str_flattened_fn.push(quote! {
                #ident
            });
        }
    } else if field.has_input() {
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

    if !field.has_output() {
        prost.field_attribute(&field_key, "#[serde(skip_serializing)]");
    }

    if !field.has_input() {
        return Ok(());
    }

    match field.kind.field_type() {
        FieldType::Enum(path) => {
            let path = get_common_import_path(strip_last_path_part(message_key), &path);
            let path_str = quote! { #path };
            prost.field_attribute(
                &field_key,
                format!("#[serde(serialize_with = \"::tinc::__private::serialize_enum::<{path_str}, _, _>\")]"),
            );
            prost.field_attribute(&field_key, format!("#[tinc(enum = \"{path_str}\")]"));
        }
        FieldType::WellKnown(_) => {
            prost.field_attribute(
                &field_key,
                "#[serde(serialize_with = \"::tinc::__private::serialize_well_known\")]",
            );
        }
        _ => {}
    };

    let field_name = field_ident_from_str(name);

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
                let _token = ::tinc::__private::PathAllowerToken::push(#name)?;
                ::tinc::__private::TrackerDeserializeIdentifier::<'de>::deserialize(
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
                let _token = ::tinc::__private::PathAllowerToken::push(#name)?;
                let tracker = #tracker;

                if !::tinc::__private::tracker_allow_duplicates(tracker.as_ref()) {
                    return ::tinc::__private::report_error(
                        ::tinc::__private::TrackedError::duplicate_field(),
                    );
                }

                ::tinc::__private::TrackerDeserializer::deserialize(
                    tracker.get_or_insert_default(),
                    #value,
                    deserializer,
                )?;
            }
        });
    }

    let push_field_token = if field.flatten {
        quote! {}
    } else {
        quote! {
            let _token = ::tinc::__private::PathToken::push_field(
                ::tinc::__private::Identifier::name(&#field_enum_ident::#ident),
            );
        }
    };

    let missing = if !field.omitable && !field.flatten {
        quote! {
            #push_field_token
            ::tinc::__private::report_error(
                ::tinc::__private::TrackedError::missing_field(),
            )?;
        }
    } else {
        quote! {}
    };

    builder.verify_deserialize_fn.push(quote! {
        if let Some(tracker) = tracker.#field_name.as_mut() {
            #push_field_token
            ::tinc::__private::TrackerValidation::validate(tracker, &self.#field_name)?;
        } else {
            #missing
        }
    });

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

    prost.message_attribute(message_key, "#[derive(::tinc::__private::Tracker)]");

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

            impl ::tinc::__private::Identifier for #field_enum_ident {
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

            impl ::tinc::__private::IdentifierFor for #message_path {
                const NAME: &'static str = stringify!(#message_ident);
                type Identifier = #field_enum_ident;
            }

            impl<'de> ::tinc::__private::TrackedStructDeserializer<'de> for #message_path {
                #[allow(unused_mut, dead_code)]
                fn deserialize<D>(
                    &mut self,
                    field: Self::Identifier,
                    mut tracker: &mut <Self::Tracker as ::tinc::__private::TrackerWrapper>::Tracker,
                    deserializer: D,
                ) -> Result<(), D::Error>
                where
                    D: ::tinc::__private::DeserializeContent<'de>,
                {
                    match field {
                        #(#deserializer_fn),*
                    }

                    ::core::result::Result::Ok(())
                }

                #[allow(unused_mut, dead_code)]
                fn validate<E>(
                    &self,
                    mut tracker: &mut <Self::Tracker as ::tinc::__private::TrackerWrapper>::Tracker,
                ) -> Result<(), E>
                where
                    E: ::tinc::reexports::serde::de::Error,
                {
                    #(#verify_deserialize_fn)*

                    ::core::result::Result::Ok(())
                }
            }

            impl ::tinc::__private::Expected for #message_path {
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

    let enum_path = get_common_import_path(enum_.package.as_str(), enum_key);
    let enum_ident = enum_path.segments.last().unwrap().ident.clone();
    let enum_ident_str = enum_ident.to_string();

    enum_attributes(enum_key, prost, enum_.repr_enum);
    for (variant, variant_opts) in &enum_.variants {
        let variant_key = format!("{enum_key}.{variant}");

        if !enum_.repr_enum {
            if let Some(rename) = &variant_opts.rename {
                serde_rename(&variant_key, rename, prost);
            } else if let Some(renamed) = rename_field(
                &strip_enum_prefix(&enum_ident_str, &to_upper_camel(variant)),
                enum_.rename_all,
            ) {
                serde_rename(&variant_key, &renamed, prost);
            }
        }

        match variant_opts.visibility {
            Some(FieldVisibility::InputOnly) => {
                prost.field_attribute(&variant_key, "#[serde(skip_serializing)]");
            }
            Some(FieldVisibility::OutputOnly) => {
                prost.field_attribute(&variant_key, "#[serde(skip_deserializing)]");
            }
            Some(FieldVisibility::Skip) => {
                prost.field_attribute(&variant_key, "#[serde(skip)]");
            }
            _ => {}
        }
    }

    let enum_impl = parse_quote! {
        impl ::tinc::__private::Expected for #enum_path {
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
