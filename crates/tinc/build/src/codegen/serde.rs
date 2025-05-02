use anyhow::Context;
use quote::{ToTokens, format_ident, quote};
use syn::parse_quote;
use tinc_pb::oneof_options::Tagged;

use super::Package;
use super::cel::compiler::{CompiledExpr, Compiler};
use super::cel::types::CelType;
use super::cel::{CelExpression, functions};
use crate::types::{
    ProtoEnumType, ProtoFieldJsonOmittable, ProtoMessageField, ProtoMessageType, ProtoModifiedValueType, ProtoOneOfType,
    ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoVisibility,
};

fn handle_oneof(
    package: &mut Package,
    field_name: &str,
    oneof: &ProtoOneOfType,
    registry: &ProtoTypeRegistry,
) -> anyhow::Result<()> {
    let message_config = package.message_config(&oneof.message);
    message_config.field_attribute(field_name, parse_quote!(#[tinc(oneof)]));

    let oneof_config = message_config.oneof_config(field_name);

    oneof_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde::Serialize)]));
    oneof_config.attribute(parse_quote!(#[derive(::tinc::__private::Tracker)]));

    let variant_identifier_ident = quote::format_ident!("___identifier");
    let mut oneof_identifier_for_ident = variant_identifier_ident.clone();
    let mut variant_idents = Vec::new();
    let mut variant_name_fn = Vec::new();
    let mut variant_from_str_fn = Vec::new();
    let mut variant_fields = Vec::new();
    let mut variant_enum_ident = Vec::new();
    let mut deserializer_impl = Vec::new();
    let mut validation_impl = Vec::new();

    let tagged_impl = if let Some(Tagged { tag, content }) = &oneof.options.tagged {
        oneof_config.attribute(parse_quote!(#[serde(tag = #tag, content = #content)]));
        oneof_config.attribute(parse_quote!(#[tinc(tagged)]));
        oneof_identifier_for_ident = quote::format_ident!("___tagged_identifier");
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
        let ident = quote::format_ident!("__field_{name}");
        let json_name = field
            .options
            .json_name
            .clone();

        oneof_config.field_attribute(name, parse_quote!(#[serde(rename = #json_name)]));
        if !field.options.visibility.has_input() {
            oneof_config.field_attribute(name, parse_quote!(#[serde(skip_serializing)]));
        }

        if field.options.visibility.has_output() {
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
            let enum_ident = field.rust_ident();
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
                                return ::tinc::__private::report_tracked_error(
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

        if let ProtoValueType::Enum(path) = &field.ty {
            let enum_opts = registry.get_enum(path).expect("enum not found");
            let path_str = enum_opts.rust_path(&oneof.message).to_token_stream().to_string();
            let serialize_with = format!("::tinc::__private::serialize_enum::<{path_str}, _, _>");
            oneof_config.field_attribute(name, parse_quote!(#[serde(serialize_with = #serialize_with)]));
            oneof_config.field_attribute(name, parse_quote!(#[tinc(enum = #path_str)]));
        }
    });

    let message = registry.get_message(&oneof.message).expect("message not found");

    let oneof_path = oneof.rust_path(&message.package);
    let oneof_ident = oneof_path.segments.last().unwrap().ident.clone();

    package.push_item(parse_quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables)]
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

                fn validate(&self, tracker: &mut <Self::Tracker as ::tinc::__private::TrackerWrapper>::Tracker) -> Result<(), ::tinc::__private::ValidationError>
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
    cel_validation_fn: &'a mut Vec<proc_macro2::TokenStream>,
}

fn handle_message_field(
    package: &mut Package,
    field_name: &str,
    field: &ProtoMessageField,
    field_builder: FieldBuilder<'_>,
    field_enum_ident: &syn::Ident,
    registry: &ProtoTypeRegistry,
) -> anyhow::Result<()> {
    let json_name = &field.options.json_name;

    let message_config = package.message_config(&field.message);

    message_config.field_attribute(field_name, parse_quote!(#[serde(rename = #json_name)]));

    let message = registry.get_message(&field.message).expect("message not found");

    let ident = quote::format_ident!("__field_{field_name}");
    if field.options.flatten {
        let flattened_ty_path = match &field.ty {
            ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(path)))
            | ProtoType::Value(ProtoValueType::Message(path)) => registry
                .get_message(path)
                .expect("message not found")
                .rust_path(&message.package),
            ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) => oneof.rust_path(&message.package),
            _ => anyhow::bail!("flattened fields must be messages or oneofs"),
        };

        message_config.field_attribute(field_name, parse_quote!(#[serde(flatten)]));

        if field.options.visibility.has_input() {
            let flattened_identifier = quote! {
                <#flattened_ty_path as ::tinc::__private::IdentifierFor>::Identifier
            };

            field_builder.deserializer_fields.push(quote! {
                <#flattened_identifier as ::tinc::__private::Identifier>::OPTIONS
            });
            field_builder.field_enum_variants.push(quote! {
                #ident(#flattened_identifier)
            });
            field_builder.field_enum_name_fn.push(quote! {
                #field_enum_ident::#ident(flatten) => ::tinc::__private::Identifier::name(flatten)
            });
            field_builder.field_enum_from_str_flattened_fn.push(quote! {
                #ident
            });
        }
    } else if field.options.visibility.has_input() {
        field_builder.deserializer_fields.push(quote! {
            &[#json_name]
        });
        field_builder.field_enum_variants.push(quote! {
            #ident
        });
        field_builder.field_enum_name_fn.push(quote! {
            #field_enum_ident::#ident => #json_name
        });
        field_builder.field_enum_from_str_fn.push(quote! {
            #json_name => #field_enum_ident::#ident
        });
    }

    if !field.options.visibility.has_output() {
        message_config.field_attribute(field_name, parse_quote!(#[serde(skip_serializing)]));
    } else if matches!(field.options.json_omittable, ProtoFieldJsonOmittable::True) {
        message_config.field_attribute(
            field_name,
            parse_quote!(#[serde(skip_serializing_if = "::tinc::__private::serde_ser_skip_default")]),
        );
    }

    if !field.options.visibility.has_input() {
        return Ok(());
    }

    match &field.ty {
        ProtoType::Value(ProtoValueType::Enum(path))
        | ProtoType::Modified(
            ProtoModifiedValueType::Optional(ProtoValueType::Enum(path))
            | ProtoModifiedValueType::Map(_, ProtoValueType::Enum(path))
            | ProtoModifiedValueType::Repeated(ProtoValueType::Enum(path)),
        ) => {
            let enum_opts = registry.get_enum(path).expect("enum not found");
            let path_str = enum_opts
                .rust_path(message.full_name.trim_last_segment())
                .to_token_stream()
                .to_string();

            let serialize_with = format!("::tinc::__private::serialize_enum::<{path_str}, _, _>");

            message_config.field_attribute(field_name, parse_quote!(#[serde(serialize_with = #serialize_with)]));
            message_config.field_attribute(field_name, parse_quote!(#[tinc(enum = #path_str)]));
        }
        ProtoType::Value(ProtoValueType::WellKnown(_))
        | ProtoType::Modified(
            ProtoModifiedValueType::Optional(ProtoValueType::WellKnown(_))
            | ProtoModifiedValueType::Map(_, ProtoValueType::WellKnown(_))
            | ProtoModifiedValueType::Repeated(ProtoValueType::WellKnown(_)),
        ) => {
            message_config.field_attribute(
                field_name,
                parse_quote!(#[serde(serialize_with = "::tinc::__private::serialize_well_known")]),
            );
        }
        ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) => {
            handle_oneof(package, field_name, oneof, registry)?;
        }
        _ => {}
    };

    let field_ident = field.rust_ident();

    let mut tracker = quote! {
        &mut tracker.#field_ident
    };

    let mut value = quote! {
        &mut self.#field_ident
    };

    // When a field is not nullable but prost generates an option<T>, we need to
    // remove the option before deserializing otherwise null will be a valid input.
    if matches!(field.ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_)))
        && (!field.options.nullable || field.options.flatten)
    {
        tracker = quote! {
            (#tracker).get_or_insert_default()
        };

        value = quote! {
            (#value).get_or_insert_default()
        };
    }

    if field.options.flatten {
        field_builder.deserializer_fn.push(quote! {
            #field_enum_ident::#ident(field) => {
                let _token = ::tinc::__private::ProtoPathToken::push_field(#field_name);
                if let Err(error) = ::tinc::__private::TrackerDeserializeIdentifier::<'de>::deserialize(
                    (#tracker).get_or_insert_default(),
                    #value,
                    field,
                    deserializer,
                ) {
                    return ::tinc::__private::report_de_error(error);
                }
            }
        });
    } else {
        field_builder.deserializer_fn.push(quote! {
            #field_enum_ident::#ident => {
                let _token = ::tinc::__private::ProtoPathToken::push_field(#field_name);
                let tracker = #tracker;

                if !::tinc::__private::tracker_allow_duplicates(tracker.as_ref()) {
                    return ::tinc::__private::report_tracked_error(
                        ::tinc::__private::TrackedError::duplicate_field(),
                    );
                }

                if let Err(error) = ::tinc::__private::TrackerDeserializer::deserialize(
                    tracker.get_or_insert_default(),
                    #value,
                    deserializer,
                ) {
                    return ::tinc::__private::report_de_error(error);
                }
            }
        });
    }

    let push_field_token = if field.options.flatten {
        quote! {
            let _token = ::tinc::__private::ProtoPathToken::push_field(#field_name);
        }
    } else {
        quote! {
            let _token = ::tinc::__private::SerdePathToken::push_field(
                ::tinc::__private::Identifier::name(&#field_enum_ident::#ident),
            );
            let _token = ::tinc::__private::ProtoPathToken::push_field(#field_name);
        }
    };

    let missing = if matches!(field.options.json_omittable, ProtoFieldJsonOmittable::False) && !field.options.flatten {
        quote! {
            #push_field_token
            ::tinc::__private::report_tracked_error(
                ::tinc::__private::TrackedError::missing_field(),
            )?;
        }
    } else {
        quote! {}
    };

    let compiler = Compiler::new(registry);

    let mut cel_validation_fn = Vec::new();

    let evaluate_expr = |compiler: &Compiler, expr: &CelExpression| {
        let resolved = compiler.resolve(&expr.expression).context("cel expression")?;
        let message = if !expr.message.args.is_empty() {
            let message_fmt = &expr.message.format;
            let args = expr
                .message
                .args
                .iter()
                .enumerate()
                .map(|(idx, (raw_expr, expr))| {
                    let ident = format_ident!("arg_{idx}");
                    let resolved = compiler.resolve(expr).context("resolving fmt arg")?;
                    Ok(quote! {
                        #ident = (
                            || {
                                ::core::result::Result::Ok::<_, ::tinc::__private::cel::CelError>(#resolved)
                            }
                        )().map_err(|err| {
                            ::tinc::__private::ValidationError::Expression {
                                error: err.to_string().into_boxed_str(),
                                field: ::tinc::__private::ProtoPathToken::current_path().into_boxed_str(),
                                expression: #raw_expr,
                            }
                        })?
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            quote! { format!(#message_fmt, #(#args),*) }
        } else {
            let message_fmt = &expr.message.format;
            quote! { #message_fmt }
        };

        let expr_str = &expr.raw_expr;

        anyhow::Ok(quote! {
            if !::tinc::__private::cel::to_bool({
                (|| {
                    ::core::result::Result::Ok::<_, ::tinc::__private::cel::CelError>(#resolved)
                })().map_err(|err| {
                    ::tinc::__private::ValidationError::Expression {
                        error: err.to_string().into_boxed_str(),
                        field: ::tinc::__private::ProtoPathToken::current_path().into_boxed_str(),
                        expression: #expr_str,
                    }
                })?
            }) {
                ::tinc::__private::report_tracked_error(
                    ::tinc::__private::TrackedError::invalid_field(#message)
                )?;
            }
        })
    };

    {
        let mut compiler = compiler.child();
        let (value_match, field_type) = if let ProtoType::Modified(ProtoModifiedValueType::Optional(ty)) = &field.ty {
            (quote!(Some(value)), ProtoType::Value(ty.clone()))
        } else {
            (quote!(value), field.ty.clone())
        };

        if let ProtoType::Value(ProtoValueType::Enum(path))
        | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path))) = &field.ty
        {
            compiler.register_function(functions::Enum(Some(path.clone())));
        }

        let is_message = matches!(field_type, ProtoType::Value(ProtoValueType::Message(_)));

        compiler.add_variable(
            "input",
            CompiledExpr {
                expr: parse_quote!(value),
                ty: CelType::Proto(field_type),
            },
        );
        let mut exprs = field
            .options
            .cel_exprs
            .field
            .iter()
            .map(|expr| evaluate_expr(&compiler, expr))
            .collect::<anyhow::Result<Vec<_>>>()?;

        if is_message {
            exprs.push(quote! {
                if ::tinc::__private::cel::CelMode::current().is_proto() {
                    ::tinc::__private::ValidateMessage::validate(value)?;
                }
            })
        }

        if !exprs.is_empty() {
            cel_validation_fn.push(quote! {{
                #[allow(irrefutable_let_patterns)]
                if let #value_match = &self.#field_ident {
                    #push_field_token
                    #(#exprs)*
                }
            }});
        }
    }

    match &field.ty {
        ProtoType::Modified(ProtoModifiedValueType::Map(key, value))
            if !field.options.cel_exprs.map_key.is_empty()
                || !field.options.cel_exprs.map_value.is_empty()
                || matches!(value, ProtoValueType::Message(_)) =>
        {
            let key_exprs = {
                let mut compiler = compiler.child();

                if let ProtoValueType::Enum(path) = key {
                    compiler.register_function(functions::Enum(Some(path.clone())));
                }

                compiler.add_variable(
                    "input",
                    CompiledExpr {
                        expr: parse_quote!(key),
                        ty: CelType::Proto(ProtoType::Value(key.clone())),
                    },
                );
                field
                    .options
                    .cel_exprs
                    .map_key
                    .iter()
                    .map(|expr| evaluate_expr(&compiler, expr))
                    .collect::<anyhow::Result<Vec<_>>>()?
            };

            let is_message = matches!(value, ProtoValueType::Message(_));

            let mut value_exprs = {
                let mut compiler = compiler.child();
                if let ProtoValueType::Enum(path) = value {
                    compiler.register_function(functions::Enum(Some(path.clone())));
                }
                compiler.add_variable(
                    "input",
                    CompiledExpr {
                        expr: parse_quote!(value),
                        ty: CelType::Proto(ProtoType::Value(value.clone())),
                    },
                );
                field
                    .options
                    .cel_exprs
                    .map_value
                    .iter()
                    .map(|expr| evaluate_expr(&compiler, expr))
                    .collect::<anyhow::Result<Vec<_>>>()?
            };

            if is_message {
                value_exprs.push(quote! {
                    if ::tinc::__private::cel::CelMode::current().is_proto() {
                        ::tinc::__private::ValidateMessage::validate(value)?;
                    }
                });
            }

            cel_validation_fn.push(quote! {{
                #push_field_token
                for (key, value) in &self.#field_ident {
                    let _token = ::tinc::__private::SerdePathToken::push_key(key);
                    let _token = ::tinc::__private::ProtoPathToken::push_key(key);
                    #(#key_exprs)*
                    #(#value_exprs)*
                }
            }});
        }
        ProtoType::Modified(ProtoModifiedValueType::Repeated(item))
            if !field.options.cel_exprs.repeated_item.is_empty() || matches!(item, ProtoValueType::Message(_)) =>
        {
            let is_message = matches!(item, ProtoValueType::Message(_));
            let mut compiler = compiler.child();
            if let ProtoValueType::Enum(path) = item {
                compiler.register_function(functions::Enum(Some(path.clone())));
            }
            compiler.add_variable(
                "input",
                CompiledExpr {
                    expr: parse_quote!(item),
                    ty: CelType::Proto(ProtoType::Value(item.clone())),
                },
            );

            let mut exprs = field
                .options
                .cel_exprs
                .repeated_item
                .iter()
                .map(|expr| evaluate_expr(&compiler, expr))
                .collect::<anyhow::Result<Vec<_>>>()?;

            if is_message {
                exprs.push(quote! {
                    if ::tinc::__private::cel::CelMode::current().is_proto() {
                        ::tinc::__private::ValidateMessage::validate(item)?;
                    }
                });
            }

            cel_validation_fn.push(quote! {{
                for (idx, item) in self.#field_ident.iter().enumerate() {
                    let _token = ::tinc::__private::SerdePathToken::push_index(idx);
                    let _token = ::tinc::__private::ProtoPathToken::push_index(idx);
                    #(#exprs)*
                }
            }});
        }
        _ => {}
    }

    field_builder.verify_deserialize_fn.push(quote! {
        if let Some(tracker) = tracker.#field_ident.as_mut() {
            #(#cel_validation_fn)*
            #push_field_token
            ::tinc::__private::TrackerValidation::validate(tracker, &self.#field_ident)?;
        } else {
            #missing
        }
    });

    field_builder.cel_validation_fn.extend(cel_validation_fn);

    Ok(())
}

pub(super) fn handle_message(
    message: &ProtoMessageType,
    package: &mut Package,
    registry: &ProtoTypeRegistry,
) -> anyhow::Result<()> {
    let message_config = package.message_config(&message.full_name);

    message_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde::Serialize)]));
    message_config.attribute(parse_quote!(#[serde(crate = "::tinc::reexports::serde")]));
    message_config.attribute(parse_quote!(#[derive(::tinc::__private::Tracker)]));

    let field_enum_ident = quote::format_ident!("___field_enum");

    let mut field_enum_variants = Vec::new();
    let mut field_enum_name_fn = Vec::new();
    let mut field_enum_from_str_fn = Vec::new();
    let mut field_enum_from_str_flattened_fn = Vec::new();
    let mut deserializer_fields = Vec::new();
    let mut deserializer_fn = Vec::new();
    let mut verify_deserialize_fn = Vec::new();
    let mut cel_validation_fn = Vec::new();

    for (field_name, field) in message.fields.iter() {
        handle_message_field(
            package,
            field_name,
            field,
            FieldBuilder {
                deserializer_fields: &mut deserializer_fields,
                field_enum_variants: &mut field_enum_variants,
                field_enum_name_fn: &mut field_enum_name_fn,
                field_enum_from_str_fn: &mut field_enum_from_str_fn,
                field_enum_from_str_flattened_fn: &mut field_enum_from_str_flattened_fn,
                deserializer_fn: &mut deserializer_fn,
                verify_deserialize_fn: &mut verify_deserialize_fn,
                cel_validation_fn: &mut cel_validation_fn,
            },
            &field_enum_ident,
            registry,
        )?;
    }

    let message_path = message.rust_path(&message.package);
    let message_ident = message_path.segments.last().unwrap().ident.clone();

    package.push_item(parse_quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables)]
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
                fn validate(
                    &self,
                    mut tracker: &mut <Self::Tracker as ::tinc::__private::TrackerWrapper>::Tracker,
                ) -> Result<(), ::tinc::__private::ValidationError>
                {
                    ::tinc::__private::cel::CelMode::Json.set();

                    #(#verify_deserialize_fn)*

                    ::core::result::Result::Ok(())
                }
            }

            impl ::tinc::__private::Expected for #message_path {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!(#message_ident))
                }
            }

            impl ::tinc::__private::ValidateMessage for #message_path {
                fn validate(&self) -> ::core::result::Result<(), ::tinc::__private::ValidationError> {
                    ::tinc::__private::cel::CelMode::Proto.set();

                    #(#cel_validation_fn)*

                    ::core::result::Result::Ok(())
                }
            }
        };
    });

    Ok(())
}

pub(super) fn handle_enum(enum_: &ProtoEnumType, package: &mut Package) -> anyhow::Result<()> {
    let enum_path = enum_.rust_path(&enum_.package);
    let enum_ident = enum_path.segments.last().unwrap().ident.clone();
    let enum_config = package.enum_config(&enum_.full_name);

    if enum_.options.repr_enum {
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_repr::Serialize_repr)]));
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_repr::Deserialize_repr)]));
    } else {
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde::Serialize)]));
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde::Deserialize)]));
    }

    enum_config.attribute(parse_quote!(#[serde(crate = "::tinc::reexports::serde")]));

    let mut to_json_matchers = if !enum_.options.repr_enum {
        Vec::new()
    } else {
        vec![quote! {
            item => ::tinc::__private::cel::CelValueConv::conv(item as i32)
        }]
    };

    for (name, variant) in &enum_.variants {
        if !enum_.options.repr_enum {
            let json_name = &variant.options.json_name;
            enum_config.variant_attribute(name, parse_quote!(#[serde(rename = #json_name)]));
            let ident = &variant.rust_ident;
            to_json_matchers.push(quote! {
                #enum_path::#ident => ::tinc::__private::cel::CelValueConv::conv(#json_name)
            })
        }

        match variant.options.visibility {
            ProtoVisibility::InputOnly => {
                enum_config.variant_attribute(name, parse_quote!(#[serde(skip_serializing)]));
            }
            ProtoVisibility::OutputOnly => {
                enum_config.variant_attribute(name, parse_quote!(#[serde(skip_deserializing)]));
            }
            ProtoVisibility::Skip => {
                enum_config.variant_attribute(name, parse_quote!(#[serde(skip)]));
            }
            _ => {}
        }
    }

    let proto_path = enum_.full_name.as_ref();

    package.push_item(parse_quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables)]
        const _: () = {
            impl ::tinc::__private::Expected for #enum_path {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "an enum of `")?;
                    write!(formatter, stringify!(#enum_ident))?;
                    write!(formatter, "`")
                }
            }

            #[::tinc::reexports::linkme::distributed_slice(::tinc::__private::cel::TINC_CEL_ENUM_VTABLE)]
            #[linkme(crate = ::tinc::reexports::linkme)]
            static ENUM_VTABLE: ::tinc::__private::cel::EnumVtable = ::tinc::__private::cel::EnumVtable {
                proto_path: #proto_path,
                is_valid: |tag| {
                    <#enum_path as std::convert::TryFrom<i32>>::try_from(tag).is_ok()
                },
                to_json: |tag| {
                    match <#enum_path as std::convert::TryFrom<i32>>::try_from(tag) {
                        Ok(value) => match value {
                            #(#to_json_matchers),*
                        }
                        Err(_) => ::tinc::__private::cel::CelValue::Null,
                    }
                },
                to_proto: |tag| {
                    match <#enum_path as std::convert::TryFrom<i32>>::try_from(tag) {
                        Ok(value) => ::tinc::__private::cel::CelValue::String(::tinc::__private::cel::CelString::Borrowed(value.as_str_name())),
                        Err(_) => ::tinc::__private::cel::CelValue::Null,
                    }
                }
            };
        };
    });

    Ok(())
}
