use anyhow::Context;
use quote::{ToTokens, quote};
use syn::parse_quote;

use super::Package;
use super::cel::compiler::{CompiledExpr, Compiler};
use super::cel::types::CelType;
use super::cel::{CelExpression, eval_message_fmt, functions};
use crate::types::{
    ProtoEnumType, ProtoFieldOptions, ProtoFieldSerdeOmittable, ProtoMessageField, ProtoMessageType, ProtoModifiedValueType,
    ProtoOneOfType, ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoVisibility, Tagged,
};

fn handle_oneof(
    package: &mut Package,
    field_name: &str,
    oneof: &ProtoOneOfType,
    registry: &ProtoTypeRegistry,
    visibility: ProtoVisibility,
) -> anyhow::Result<()> {
    let message_config = package.message_config(&oneof.message);
    message_config.field_attribute(field_name, parse_quote!(#[tinc(oneof)]));

    let oneof_config = message_config.oneof_config(field_name);

    if visibility.has_output() {
        oneof_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_derive::Serialize)]));
    }

    oneof_config.attribute(parse_quote!(#[derive(::tinc::__private::Tracker)]));

    let variant_identifier_ident = quote::format_ident!("___identifier");
    let mut oneof_identifier_for_ident = variant_identifier_ident.clone();
    let mut variant_idents = Vec::new();
    let mut variant_name_fn = Vec::new();
    let mut variant_from_str_fn = Vec::new();
    let mut variant_fields = Vec::new();
    let mut variant_enum_ident = Vec::new();
    let mut deserializer_impl = Vec::new();
    let mut validate_message_impl = Vec::new();

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

    for (field_name, field) in &oneof.fields {
        anyhow::ensure!(!field.options.flatten, "oneof fields cannot be flattened");

        let ident = quote::format_ident!("__field_{field_name}");
        let serde_name = &field.options.serde_name;

        oneof_config.field_attribute(field_name, parse_quote!(#[serde(rename = #serde_name)]));
        if visibility.has_output() && !field.options.visibility.has_output() {
            oneof_config.field_attribute(field_name, parse_quote!(#[serde(skip_serializing)]));
        }

        if field.options.visibility.has_input() {
            variant_idents.push(ident.clone());
            variant_name_fn.push(quote! {
                #variant_identifier_ident::#ident => #serde_name
            });
            variant_from_str_fn.push(quote! {
                #serde_name => #variant_identifier_ident::#ident
            });
            variant_fields.push(quote! {
                #serde_name
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
                                        ::tinc::__private::Identifier::name(&Self::tracker_to_identifier(tracker)),
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
                                        ::tinc::__private::Identifier::name(&Self::value_to_identifier(value)),
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

            let cel_validation_fn = cel_expressions(
                registry,
                &ProtoType::Value(field.ty.clone()),
                &field.full_name,
                &field.options,
                quote!(*value),
                quote!(tracker),
            )?;

            let serde_name = if let Some(tagged) = &oneof.options.tagged {
                tagged.content.as_str()
            } else {
                field.options.serde_name.as_str()
            };

            validate_message_impl.push(quote! {
                (Self::#enum_ident(value)) => {
                    let _token = ::tinc::__private::ProtoPathToken::push_field(#field_name);
                    let _token = ::tinc::__private::SerdePathToken::push_field(#serde_name);
                    let tracker = match tracker {
                        ::core::option::Option::Some(___Tracker::#enum_ident(tracker)) => ::core::option::Option::Some(tracker),
                        ::core::option::Option::Some(t) => return ::core::result::Result::Err(
                            ::tinc::reexports::serde::de::Error::custom(format!(
                                "tracker and value do not match: {:?} != {:?}",
                                ::tinc::__private::Identifier::name(&<Self as ::tinc::__private::TrackedOneOfDeserializer<'_>>::tracker_to_identifier(t)),
                                ::tinc::__private::Identifier::name(&<Self as ::tinc::__private::TrackedOneOfDeserializer<'_>>::value_to_identifier(self)),
                            )),
                        ),
                        ::core::option::Option::None => ::core::option::Option::None,
                    };
                    #(#cel_validation_fn)*
                }
            });
        }

        match &field.ty {
            ProtoValueType::Enum(path) => {
                let path_str = registry
                    .resolve_rust_path(&oneof.message, path)
                    .expect("enum not found")
                    .to_token_stream()
                    .to_string();

                if field.options.visibility.has_output() {
                    let serialize_with = format!("::tinc::__private::serialize_enum::<{path_str}, _, _>");
                    oneof_config.field_attribute(field_name, parse_quote!(#[serde(serialize_with = #serialize_with)]));
                }

                oneof_config.field_attribute(field_name, parse_quote!(#[tinc(enum = #path_str)]));
            }
            ProtoValueType::WellKnown(_) | ProtoValueType::Bytes => {
                if field.options.visibility.has_output() {
                    oneof_config.field_attribute(
                        field_name,
                        parse_quote!(#[serde(serialize_with = "::tinc::__private::serialize_well_known")]),
                    );
                }
            }
            ProtoValueType::Float | ProtoValueType::Double => {
                #[cfg(feature = "non_finite_floats")]
                {
                    if field.options.visibility.has_output() {
                        oneof_config.field_attribute(
                            field_name,
                            parse_quote!(#[serde(serialize_with = "::tinc::__private::serialize_floats_with_non_finite")]),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    let message = registry.get_message(&oneof.message).expect("message not found");

    let oneof_path = oneof.rust_path(&message.package);
    let oneof_ident = oneof_path.segments.last().unwrap().ident.clone();

    package.push_item(parse_quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables, unused_parens)]
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

                fn tracker_to_identifier(v: &___Tracker) -> #variant_identifier_ident {
                    match v {
                        #(___Tracker::#variant_enum_ident(_) => #variant_identifier_ident::#variant_idents),*
                    }
                }

                fn value_to_identifier(v: &#oneof_path) -> #variant_identifier_ident {
                    match v {
                        #(#oneof_path::#variant_enum_ident(_) => #variant_identifier_ident::#variant_idents),*
                    }
                }
            }

            impl ::tinc::__private::TincValidate for #oneof_path {
                fn validate(&self, tracker: Option<&<#oneof_path as ::tinc::__private::TrackerFor>::Tracker>) -> ::core::result::Result<(), ::tinc::__private::ValidationError> {
                    let tracker = tracker.and_then(|t| t.as_ref());
                    match self {
                        #(#validate_message_impl)*
                        #[allow(unreachable_patterns)]
                        _ => {}
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
    let serde_name = &field.options.serde_name;

    let message_config = package.message_config(&field.message);

    message_config.field_attribute(field_name, parse_quote!(#[serde(rename = #serde_name)]));

    let message = registry.get_message(&field.message).expect("message not found");

    let ident = quote::format_ident!("__field_{field_name}");
    if field.options.flatten {
        let flattened_ty_path = match &field.ty {
            ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(path)))
            | ProtoType::Value(ProtoValueType::Message(path)) => {
                registry.resolve_rust_path(&message.package, path).expect("message not found")
            }
            ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) => oneof.rust_path(&message.package),
            _ => anyhow::bail!("flattened fields must be messages or oneofs"),
        };

        if field.options.visibility.has_output() {
            message_config.field_attribute(field_name, parse_quote!(#[serde(flatten)]));
        }

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
            &[#serde_name]
        });
        field_builder.field_enum_variants.push(quote! {
            #ident
        });
        field_builder.field_enum_name_fn.push(quote! {
            #field_enum_ident::#ident => #serde_name
        });
        field_builder.field_enum_from_str_fn.push(quote! {
            #serde_name => #field_enum_ident::#ident
        });
    }

    if field.options.visibility.has_output() {
        if matches!(field.options.serde_omittable, ProtoFieldSerdeOmittable::True) {
            message_config.field_attribute(
                field_name,
                parse_quote!(#[serde(skip_serializing_if = "::tinc::__private::serde_ser_skip_default")]),
            );
        }
    } else {
        message_config.field_attribute(field_name, parse_quote!(#[serde(skip_serializing)]));
    }

    match field.ty.value_type() {
        Some(ProtoValueType::Enum(path)) => {
            let path_str = registry
                .resolve_rust_path(message.full_name.trim_last_segment(), path)
                .expect("enum not found")
                .to_token_stream()
                .to_string();

            if field.options.visibility.has_output() {
                let serialize_with = format!("::tinc::__private::serialize_enum::<{path_str}, _, _>");
                message_config.field_attribute(field_name, parse_quote!(#[serde(serialize_with = #serialize_with)]));
            }

            message_config.field_attribute(field_name, parse_quote!(#[tinc(enum = #path_str)]));
        }
        Some(ProtoValueType::WellKnown(_) | ProtoValueType::Bytes) => {
            if field.options.visibility.has_output() {
                message_config.field_attribute(
                    field_name,
                    parse_quote!(#[serde(serialize_with = "::tinc::__private::serialize_well_known")]),
                );
            }
        }
        Some(ProtoValueType::Float | ProtoValueType::Double) => {
            #[cfg(feature = "non_finite_floats")]
            {
                if field.options.visibility.has_output() {
                    message_config.field_attribute(
                        field_name,
                        parse_quote!(#[serde(serialize_with = "::tinc::__private::serialize_floats_with_non_finite")]),
                    );
                }
            }
        }
        _ => {}
    }

    if let ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) = &field.ty {
        handle_oneof(package, field_name, oneof, registry, field.options.visibility)?;
    }

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

    if field.options.visibility.has_input() {
        if field.options.flatten {
            field_builder.deserializer_fn.push(quote! {
                #field_enum_ident::#ident(field) => {
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
    }

    let push_field_token = if !field.options.flatten {
        quote! {
            let _token = ::tinc::__private::SerdePathToken::push_field(
                ::tinc::__private::Identifier::name(&#field_enum_ident::#ident),
            );
        }
    } else {
        quote! {}
    };

    let missing = if matches!(field.options.serde_omittable, ProtoFieldSerdeOmittable::False) && !field.options.flatten {
        quote! {
            ::tinc::__private::report_tracked_error(
                ::tinc::__private::TrackedError::missing_field(),
            )?;
        }
    } else {
        quote! {}
    };

    let mut tracker_access = quote!(tracker.and_then(|t| t.#field_ident.as_ref()));
    if matches!(field.ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_))) {
        tracker_access = quote!(#tracker_access.and_then(|t| t.as_ref()))
    }

    if field.options.visibility.has_input() {
        let cel_validation_fn = cel_expressions(
            registry,
            &field.ty,
            &field.full_name,
            &field.options,
            quote!(self.#field_ident),
            tracker_access,
        )?;

        field_builder.cel_validation_fn.push(quote!({
            let _token = ::tinc::__private::ProtoPathToken::push_field(#field_name);
            #push_field_token

            // TODO: this seems wrong. I feel as if we should validate it even if omittable is true.
            if tracker.is_none_or(|t| t.#field_ident.is_some()) {
                #(#cel_validation_fn)*
            } else {
                #missing
            }
        }));
    }

    Ok(())
}

fn cel_expressions(
    registry: &ProtoTypeRegistry,
    ty: &ProtoType,
    field_full_name: &str,
    options: &ProtoFieldOptions,
    value_accessor: proc_macro2::TokenStream,
    tracker_accessor: proc_macro2::TokenStream,
) -> anyhow::Result<Vec<proc_macro2::TokenStream>> {
    let compiler = Compiler::new(registry);
    let mut cel_validation_fn = Vec::new();

    let evaluate_expr = |ctx: &Compiler, expr: &CelExpression| {
        let mut ctx = ctx.child();
        if let Some(this) = expr.this.clone() {
            ctx.add_variable("this", CompiledExpr::constant(this));
        }
        let parsed = cel_parser::parse(&expr.expression).context("expression parse")?;
        let resolved = ctx.resolve(&parsed).context("cel expression")?;
        let expr_str = &expr.expression;
        let message = eval_message_fmt(field_full_name, &expr.message, &ctx).context("message")?;

        anyhow::Ok(quote! {
            if !::tinc::__private::cel::to_bool({
                (|| {
                    ::core::result::Result::Ok::<_, ::tinc::__private::cel::CelError>(#resolved)
                })().map_err(|err| {
                    ::tinc::__private::ValidationError::Expression {
                        error: err.to_string().into_boxed_str(),
                        field: #field_full_name,
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
        let (value_match, field_type) = match ty {
            ProtoType::Modified(ProtoModifiedValueType::Optional(ty)) => (quote!(Some(value)), ProtoType::Value(ty.clone())),
            ty @ ProtoType::Modified(ProtoModifiedValueType::OneOf(_)) => (quote!(Some(value)), ty.clone()),
            _ => (quote!(value), ty.clone()),
        };

        if let ProtoType::Value(ProtoValueType::Enum(path))
        | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path))) = ty
        {
            compiler.register_function(functions::Enum(Some(path.clone())));
        }

        let recursive_validate = matches!(
            field_type,
            ProtoType::Value(ProtoValueType::Message(_)) | ProtoType::Modified(ProtoModifiedValueType::OneOf(_))
        );

        compiler.add_variable(
            "input",
            CompiledExpr::runtime(CelType::Proto(field_type), parse_quote!(value)),
        );
        let mut exprs = options
            .cel_exprs
            .field
            .iter()
            .map(|expr| evaluate_expr(&compiler, expr))
            .collect::<anyhow::Result<Vec<_>>>()?;

        if recursive_validate {
            exprs.push(quote! {
                ::tinc::__private::TincValidate::validate(value, #tracker_accessor)?;
            })
        }

        if !exprs.is_empty() {
            cel_validation_fn.push(quote! {{
                #[allow(irrefutable_let_patterns)]
                if let #value_match = &#value_accessor {
                    #(#exprs)*
                }
            }});
        }

        if !options.nullable
            && matches!(
                &ty,
                ProtoType::Modified(ProtoModifiedValueType::Optional(_) | ProtoModifiedValueType::OneOf(_))
            )
        {
            cel_validation_fn.push(quote! {{
                if #value_accessor.is_none() {
                    ::tinc::__private::report_tracked_error(
                        ::tinc::__private::TrackedError::missing_field()
                    )?;
                }
            }})
        }
    }

    match ty {
        ProtoType::Modified(ProtoModifiedValueType::Map(key, value))
            if !options.cel_exprs.map_key.is_empty()
                || !options.cel_exprs.map_value.is_empty()
                || matches!(value, ProtoValueType::Message(_)) =>
        {
            let key_exprs = {
                let mut compiler = compiler.child();

                if let ProtoValueType::Enum(path) = key {
                    compiler.register_function(functions::Enum(Some(path.clone())));
                }

                compiler.add_variable(
                    "input",
                    CompiledExpr::runtime(CelType::Proto(ProtoType::Value(key.clone())), parse_quote!(key)),
                );
                options
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
                    CompiledExpr::runtime(CelType::Proto(ProtoType::Value(value.clone())), parse_quote!(value)),
                );
                options
                    .cel_exprs
                    .map_value
                    .iter()
                    .map(|expr| evaluate_expr(&compiler, expr))
                    .collect::<anyhow::Result<Vec<_>>>()?
            };

            if is_message {
                value_exprs.push(quote!({
                    let tracker = match #tracker_accessor {
                        ::core::option::Option::Some(t) => Some(t.get(key).expect("map tracker state missing item, this is a bug report it.")),
                        ::core::option::Option::None => None
                    };
                    ::tinc::__private::TincValidate::validate(value, tracker)?;
                }));
            }

            cel_validation_fn.push(quote! {{
                for (key, value) in &#value_accessor {
                    let _token = ::tinc::__private::ProtoPathToken::push_field(key);
                    let _token = ::tinc::__private::SerdePathToken::push_field(key);
                    #(#key_exprs)*
                    #(#value_exprs)*
                }
            }});
        }
        ProtoType::Modified(ProtoModifiedValueType::Repeated(item))
            if !options.cel_exprs.repeated_item.is_empty() || matches!(item, ProtoValueType::Message(_)) =>
        {
            let is_message = matches!(item, ProtoValueType::Message(_));
            let mut compiler = compiler.child();
            if let ProtoValueType::Enum(path) = item {
                compiler.register_function(functions::Enum(Some(path.clone())));
            }
            compiler.add_variable(
                "input",
                CompiledExpr::runtime(CelType::Proto(ProtoType::Value(item.clone())), parse_quote!(item)),
            );

            let mut exprs = options
                .cel_exprs
                .repeated_item
                .iter()
                .map(|expr| evaluate_expr(&compiler, expr))
                .collect::<anyhow::Result<Vec<_>>>()?;

            if is_message {
                exprs.push(quote!({
                    let tracker = match #tracker_accessor {
                        ::core::option::Option::Some(t) => Some(t.get(idx).expect("repeated tracker state missing item, this is a bug report it.")),
                        ::core::option::Option::None => None
                    };
                    ::tinc::__private::TincValidate::validate(item, tracker)?;
                }));
            }

            cel_validation_fn.push(quote! {{
                for (idx, item) in #value_accessor.iter().enumerate() {
                    let _token = ::tinc::__private::ProtoPathToken::push_index(idx);
                    let _token = ::tinc::__private::SerdePathToken::push_index(idx);
                    #(#exprs)*
                }
            }});
        }
        _ => {}
    }

    Ok(cel_validation_fn)
}

pub(super) fn handle_message(
    message: &ProtoMessageType,
    package: &mut Package,
    registry: &ProtoTypeRegistry,
) -> anyhow::Result<()> {
    let message_config = package.message_config(&message.full_name);

    message_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_derive::Serialize)]));
    message_config.attribute(parse_quote!(#[serde(crate = "::tinc::reexports::serde")]));
    message_config.attribute(parse_quote!(#[derive(::tinc::__private::Tracker)]));

    let field_enum_ident = quote::format_ident!("___field_enum");

    let mut field_enum_variants = Vec::new();
    let mut field_enum_name_fn = Vec::new();
    let mut field_enum_from_str_fn = Vec::new();
    let mut field_enum_from_str_flattened_fn = Vec::new();
    let mut deserializer_fields = Vec::new();
    let mut deserializer_fn = Vec::new();
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
                cel_validation_fn: &mut cel_validation_fn,
            },
            &field_enum_ident,
            registry,
        )?;
    }

    let message_path = registry
        .resolve_rust_path(&message.package, &message.full_name)
        .expect("message not found");
    let message_ident = message_path.segments.last().unwrap().ident.clone();

    package.push_item(parse_quote! {
        #[allow(clippy::all, dead_code, unused_imports, unused_variables, unused_parens)]
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

            type ___Tracker = <<#message_path as ::tinc::__private::TrackerFor>::Tracker as ::tinc::__private::TrackerWrapper>::Tracker;

            impl<'de> ::tinc::__private::TrackedStructDeserializer<'de> for #message_path {
                #[allow(unused_mut, dead_code)]
                fn deserialize<D>(
                    &mut self,
                    field: Self::Identifier,
                    mut tracker: &mut ___Tracker,
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
            }

            impl ::tinc::__private::Expected for #message_path {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!(#message_ident))
                }
            }

            impl ::tinc::__private::TincValidate for #message_path {
                fn validate(&self, tracker: Option<&<#message_path as ::tinc::__private::TrackerFor>::Tracker>) -> ::core::result::Result<(), ::tinc::__private::ValidationError> {
                    let tracker = tracker.map(|t| &**t);
                    #(#cel_validation_fn)*
                    ::core::result::Result::Ok(())
                }
            }
        };
    });

    Ok(())
}

pub(super) fn handle_enum(enum_: &ProtoEnumType, package: &mut Package, registry: &ProtoTypeRegistry) -> anyhow::Result<()> {
    let enum_path = registry
        .resolve_rust_path(&enum_.package, &enum_.full_name)
        .expect("enum not found");
    let enum_ident = enum_path.segments.last().unwrap().ident.clone();
    let enum_config = package.enum_config(&enum_.full_name);

    if enum_.options.repr_enum {
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_repr::Serialize_repr)]));
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_repr::Deserialize_repr)]));
    } else {
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_derive::Serialize)]));
        enum_config.attribute(parse_quote!(#[derive(::tinc::reexports::serde_derive::Deserialize)]));
    }

    enum_config.attribute(parse_quote!(#[serde(crate = "::tinc::reexports::serde")]));

    let mut to_serde_matchers = if !enum_.options.repr_enum {
        Vec::new()
    } else {
        vec![quote! {
            item => ::tinc::__private::cel::CelValueConv::conv(item as i32)
        }]
    };

    for (name, variant) in &enum_.variants {
        if !enum_.options.repr_enum {
            let serde_name = &variant.options.serde_name;
            enum_config.variant_attribute(name, parse_quote!(#[serde(rename = #serde_name)]));
            let ident = &variant.rust_ident;
            to_serde_matchers.push(quote! {
                #enum_path::#ident => ::tinc::__private::cel::CelValueConv::conv(#serde_name)
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
        #[allow(clippy::all, dead_code, unused_imports, unused_variables, unused_parens)]
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
                to_serde: |tag| {
                    match <#enum_path as std::convert::TryFrom<i32>>::try_from(tag) {
                        Ok(value) => match value {
                            #(#to_serde_matchers),*
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
