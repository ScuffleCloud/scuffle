use std::collections::BTreeMap;

use anyhow::Context;
use base64::Engine;
use indexmap::IndexMap;
use openapiv3_1::{Object, Ref, Schema, Type};
use proc_macro2::TokenStream;
use quote::quote;
use tinc_cel::{CelValue, NumberTy};

use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerTarget, ConstantCompiledExpr};
use crate::codegen::cel::{CelExpression, CelExpressions, functions};
use crate::codegen::utils::field_ident_from_str;
use crate::types::{ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoWellKnownType};

fn cel_to_json(cel: &CelValue<'static>, type_registry: &ProtoTypeRegistry) -> anyhow::Result<serde_json::Value> {
    match cel {
        CelValue::Null => Ok(serde_json::Value::Null),
        CelValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        CelValue::Map(map) => Ok(serde_json::Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if let CelValue::String(key) = key {
                        Ok((key.to_string(), cel_to_json(value, type_registry)?))
                    } else {
                        anyhow::bail!("map keys must be a string")
                    }
                })
                .collect::<anyhow::Result<_>>()?,
        )),
        CelValue::List(list) => Ok(serde_json::Value::Array(
            list.iter()
                .map(|i| cel_to_json(i, type_registry))
                .collect::<anyhow::Result<_>>()?,
        )),
        CelValue::String(s) => Ok(serde_json::Value::String(s.to_string())),
        CelValue::Number(NumberTy::F64(f)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*f).context("f64 is not a valid float")?,
        )),
        CelValue::Number(NumberTy::I64(i)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_i128(*i as i128).context("i64 is not a valid int")?,
        )),
        CelValue::Number(NumberTy::U64(u)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_u128(*u as u128).context("u64 is not a valid uint")?,
        )),
        CelValue::Duration(duration) => Ok(serde_json::Value::String(duration.to_string())),
        CelValue::Timestamp(timestamp) => Ok(serde_json::Value::String(timestamp.to_rfc3339())),
        CelValue::Bytes(bytes) => Ok(serde_json::Value::String(
            base64::engine::general_purpose::STANDARD.encode(bytes),
        )),
        CelValue::Enum(cel_enum) => {
            let enum_ty = type_registry
                .get_enum(&cel_enum.tag)
                .with_context(|| format!("couldnt find enum {}", cel_enum.tag.as_ref()))?;
            if enum_ty.options.repr_enum {
                Ok(serde_json::Value::from(cel_enum.value))
            } else {
                let variant = enum_ty
                    .variants
                    .values()
                    .find(|v| v.value == cel_enum.value)
                    .with_context(|| format!("{} has no value for {}", cel_enum.tag.as_ref(), cel_enum.value))?;
                Ok(serde_json::Value::from(variant.options.serde_name.clone()))
            }
        }
    }
}

fn parse_resolve(compiler: &Compiler, expr: &str) -> anyhow::Result<CelValue<'static>> {
    let expr = cel_parser::parse(expr).context("parse")?;
    let resolved = compiler.resolve(&expr).context("resolve")?;
    match resolved {
        CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(value),
        CompiledExpr::Runtime(_) => anyhow::bail!("expression needs runtime evaluation"),
    }
}

fn handle_expr(mut ctx: Compiler, ty: &ProtoType, expr: &CelExpression) -> anyhow::Result<Vec<Schema>> {
    ctx.set_target(CompilerTarget::Serde);

    if let Some(this) = expr.this.clone() {
        ctx.add_variable("this", CompiledExpr::constant(this));
    }

    if let Some(ProtoValueType::Enum(path)) = ty.value_type() {
        ctx.register_function(functions::Enum(Some(path.clone())));
    }

    let mut schemas = Vec::new();
    for schema in &expr.jsonschemas {
        let value = parse_resolve(&ctx, schema)?;
        let value = cel_to_json(&value, ctx.registry())?;
        if !value.is_null() {
            schemas.push(serde_json::from_value(value).context("bad openapi schema")?);
        }
    }

    Ok(schemas)
}

#[derive(Debug)]
enum ExcludePaths {
    True,
    Child(BTreeMap<String, ExcludePaths>),
}

#[derive(Debug, Clone, Copy)]
enum BytesEncoding {
    Base64,
    Binary,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BodyMethod<'a> {
    Text,
    Json,
    Binary(Option<&'a str>),
}

impl BodyMethod<'_> {
    fn bytes_encoding(&self) -> BytesEncoding {
        match self {
            BodyMethod::Binary(_) => BytesEncoding::Binary,
            _ => BytesEncoding::Base64,
        }
    }

    fn deserialize_method(&self) -> syn::Ident {
        match self {
            BodyMethod::Text => syn::parse_quote!(deserialize_body_text),
            BodyMethod::Binary(_) => syn::parse_quote!(deserialize_body_bytes),
            BodyMethod::Json => syn::parse_quote!(deserialize_body_json),
        }
    }

    fn content_type(&self) -> &str {
        match self {
            BodyMethod::Binary(ct) => ct.unwrap_or(self.default_content_type()),
            _ => self.default_content_type(),
        }
    }

    fn default_content_type(&self) -> &'static str {
        match self {
            BodyMethod::Binary(_) => "application/octet-stream",
            BodyMethod::Json => "application/json",
            BodyMethod::Text => "text/plain",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GenerateDirection {
    Input,
    Output,
}

struct FieldExtract {
    tokens: proc_macro2::TokenStream,
    ty: ProtoType,
    cel: CelExpressions,
    is_optional: bool,
}

fn input_field_getter_gen(
    registry: &ProtoTypeRegistry,
    ty: &ProtoValueType,
    mut mapping: TokenStream,
    field_str: &str,
) -> anyhow::Result<FieldExtract> {
    let ProtoValueType::Message(path) = ty else {
        anyhow::bail!("cannot extract field on non-message type: {field_str}");
    };

    let mut next_message = Some(registry.get_message(path).unwrap());
    let mut is_optional = false;
    let mut kind = None;
    let mut cel = None;
    for part in field_str.split('.') {
        let Some(field) = next_message.and_then(|message| message.fields.get(part)) else {
            anyhow::bail!("message does not have field: {field_str}");
        };

        let field_ident = field_ident_from_str(part);

        let optional_unwrap = is_optional.then(|| {
            quote! {
                let mut tracker = tracker.get_or_insert_default();
                let mut target = target.get_or_insert_default();
            }
        });

        kind = Some(&field.ty);
        cel = Some(&field.options.cel_exprs);
        mapping = quote! {{
            let (tracker, target) = #mapping;
            #optional_unwrap
            let tracker = tracker.#field_ident.get_or_insert_default();
            let target = &mut target.#field_ident;
            (tracker, target)
        }};

        is_optional = matches!(
            field.ty,
            ProtoType::Modified(ProtoModifiedValueType::Optional(_) | ProtoModifiedValueType::OneOf(_))
        );
        next_message = match &field.ty {
            ProtoType::Value(ProtoValueType::Message(path))
            | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(path))) => {
                Some(registry.get_message(path).unwrap())
            }
            _ => None,
        }
    }

    Ok(FieldExtract {
        tokens: mapping,
        ty: kind.unwrap().clone(),
        cel: cel.unwrap().clone(),
        is_optional,
    })
}

fn output_field_getter_gen(
    registry: &ProtoTypeRegistry,
    ty: &ProtoValueType,
    mut mapping: TokenStream,
    field_str: &str,
) -> anyhow::Result<FieldExtract> {
    let ProtoValueType::Message(path) = ty else {
        anyhow::bail!("cannot extract field on non-message type: {field_str}");
    };

    let mut next_message = Some(registry.get_message(path).unwrap());
    let mut was_optional = false;
    let mut kind = None;
    let mut cel = None;
    for part in field_str.split('.') {
        let Some(field) = next_message.and_then(|message| message.fields.get(part)) else {
            anyhow::bail!("message does not have field: {field_str}");
        };

        let field_ident = field_ident_from_str(part);

        kind = Some(&field.ty);
        cel = Some(&field.options.cel_exprs);
        let is_optional = matches!(
            field.ty,
            ProtoType::Modified(ProtoModifiedValueType::Optional(_) | ProtoModifiedValueType::OneOf(_))
        );

        mapping = match (is_optional, was_optional) {
            (true, true) => quote!(#mapping.and_then(|m| m.#field_ident.as_ref())),
            (false, true) => quote!(#mapping.map(|m| &m.#field_ident)),
            (true, false) => quote!(#mapping.#field_ident.as_ref()),
            (false, false) => quote!(&#mapping.#field_ident),
        };

        was_optional = was_optional || is_optional;

        next_message = match &field.ty {
            ProtoType::Value(ProtoValueType::Message(path))
            | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(path))) => {
                Some(registry.get_message(path).unwrap())
            }
            _ => None,
        }
    }

    Ok(FieldExtract {
        cel: cel.unwrap().clone(),
        ty: kind.unwrap().clone(),
        is_optional: was_optional,
        tokens: mapping,
    })
}

fn parse_route(route: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = route.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            continue;
        }

        // Skip escaped '{{'
        if let Some(&'{') = chars.peek() {
            chars.next();
            continue;
        }

        let mut param = String::new();
        for c in &mut chars {
            if c == '}' {
                params.push(param);
                break;
            }

            param.push(c);
        }
    }

    params
}

struct PathFields {
    defs: Vec<proc_macro2::TokenStream>,
    mappings: Vec<proc_macro2::TokenStream>,
    param_schemas: IndexMap<String, (ProtoValueType, CelExpressions)>,
}

fn path_struct(
    registry: &ProtoTypeRegistry,
    ty: &ProtoValueType,
    package: &str,
    fields: &[String],
    mapping: TokenStream,
) -> anyhow::Result<PathFields> {
    let mut defs = Vec::new();
    let mut mappings = Vec::new();
    let mut param_schemas = IndexMap::new();

    let match_single_ty = |ty: &ProtoValueType| {
        Some(match &ty {
            ProtoValueType::Enum(path) => {
                let path = registry.resolve_rust_path(package, path).expect("enum not found");
                quote! {
                    #path
                }
            }
            ProtoValueType::Bool => quote! {
                ::core::primitive::bool
            },
            ProtoValueType::Float => quote! {
                ::core::primitive::f32
            },
            ProtoValueType::Double => quote! {
                ::core::primitive::f64
            },
            ProtoValueType::Int32 => quote! {
                ::core::primitive::i32
            },
            ProtoValueType::Int64 => quote! {
                ::core::primitive::i64
            },
            ProtoValueType::UInt32 => quote! {
                ::core::primitive::u32
            },
            ProtoValueType::UInt64 => quote! {
                ::core::primitive::u64
            },
            ProtoValueType::String => quote! {
                ::std::string::String
            },
            ProtoValueType::WellKnown(ProtoWellKnownType::Duration) => quote! {
                ::tinc::__private::well_known::Duration
            },
            ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp) => quote! {
                ::tinc::__private::well_known::Timestamp
            },
            ProtoValueType::WellKnown(ProtoWellKnownType::Value) => quote! {
                ::tinc::__private::well_known::Value
            },
            _ => return None,
        })
    };

    match &ty {
        ProtoValueType::Message(_) => {
            for (idx, field) in fields.iter().enumerate() {
                let field_str = field.as_ref();
                let path_field_ident = quote::format_ident!("field_{idx}");
                let FieldExtract {
                    cel,
                    tokens,
                    ty,
                    is_optional,
                } = input_field_getter_gen(registry, ty, mapping.clone(), field_str)?;

                let setter = if is_optional {
                    quote! {
                        tracker.get_or_insert_default();
                        target.insert(path.#path_field_ident.into());
                    }
                } else {
                    quote! {
                        *target = path.#path_field_ident.into();
                    }
                };

                mappings.push(quote! {{
                    let (tracker, target) = #tokens;
                    #setter;
                }});

                let ty = match ty {
                    ProtoType::Modified(ProtoModifiedValueType::Optional(value)) | ProtoType::Value(value) => Some(value),
                    _ => None,
                };

                let Some(tokens) = ty.as_ref().and_then(match_single_ty) else {
                    anyhow::bail!("type cannot be mapped: {ty:?}");
                };

                let ty = ty.unwrap();

                param_schemas.insert(field.clone(), (ty, cel));

                defs.push(quote! {
                    #[serde(rename = #field_str)]
                    #path_field_ident: #tokens
                });
            }
        }
        ty => {
            let Some(ty) = match_single_ty(ty) else {
                anyhow::bail!("type cannot be mapped: {ty:?}");
            };

            if fields.len() != 1 {
                anyhow::bail!("well-known type can only have one field");
            }

            if fields[0] != "value" {
                anyhow::bail!("well-known type can only have field 'value'");
            }

            mappings.push(quote! {{
                let (_, target) = #mapping;
                *target = path.value.into();
            }});

            defs.push(quote! {
                #[serde(rename = "value")]
                value: #ty
            });
        }
    }

    Ok(PathFields {
        defs,
        mappings,
        param_schemas,
    })
}

pub(super) struct InputGenerator<'a> {
    used_paths: BTreeMap<String, ExcludePaths>,
    types: &'a ProtoTypeRegistry,
    components: &'a mut openapiv3_1::Components,
    package: &'a str,
    root_ty: ProtoValueType,
    tracker_ident: syn::Ident,
    target_ident: syn::Ident,
    state_ident: syn::Ident,
}

#[derive(Default)]
pub(super) struct GeneratedParams {
    pub tokens: TokenStream,
    pub params: Vec<openapiv3_1::path::Parameter>,
}

pub(super) struct GeneratedBody<B> {
    pub tokens: TokenStream,
    pub body: B,
}

impl<'a> InputGenerator<'a> {
    pub(super) fn new(
        types: &'a ProtoTypeRegistry,
        components: &'a mut openapiv3_1::Components,
        package: &'a str,
        ty: ProtoValueType,
        tracker_ident: syn::Ident,
        target_ident: syn::Ident,
        state_ident: syn::Ident,
    ) -> Self {
        Self {
            components,
            types,
            used_paths: BTreeMap::new(),
            package,
            root_ty: ty,
            target_ident,
            tracker_ident,
            state_ident,
        }
    }
}

pub(super) struct OutputGenerator<'a> {
    types: &'a ProtoTypeRegistry,
    components: &'a mut openapiv3_1::Components,
    root_ty: ProtoValueType,
    response_ident: syn::Ident,
    builder_ident: syn::Ident,
}

impl<'a> OutputGenerator<'a> {
    pub(super) fn new(
        types: &'a ProtoTypeRegistry,
        components: &'a mut openapiv3_1::Components,
        ty: ProtoValueType,
        response_ident: syn::Ident,
        builder_ident: syn::Ident,
    ) -> Self {
        Self {
            components,
            types,
            root_ty: ty,
            response_ident,
            builder_ident,
        }
    }
}

impl InputGenerator<'_> {
    fn consume_field(&mut self, field: &str) -> anyhow::Result<()> {
        let mut parts = field.split('.').peekable();
        let first_part = parts.next().expect("parts empty").to_owned();

        // Start with the first part of the path
        let mut current_map = self.used_paths.entry(first_part).or_insert(if parts.peek().is_none() {
            ExcludePaths::True
        } else {
            ExcludePaths::Child(BTreeMap::new())
        });

        // Iterate over the remaining parts of the path
        while let Some(part) = parts.next() {
            match current_map {
                ExcludePaths::True => anyhow::bail!("duplicate path: {field}"),
                ExcludePaths::Child(map) => {
                    current_map = map.entry(part.to_owned()).or_insert(if parts.peek().is_none() {
                        ExcludePaths::True
                    } else {
                        ExcludePaths::Child(BTreeMap::new())
                    });
                }
            }
        }

        anyhow::ensure!(matches!(current_map, ExcludePaths::True), "duplicate path: {field}");

        Ok(())
    }

    fn base_extract(&self) -> TokenStream {
        let tracker = &self.tracker_ident;
        let target = &self.target_ident;
        quote!((&mut #tracker, &mut #target))
    }

    pub(super) fn generate_query_parameter(&mut self, field: Option<&str>) -> anyhow::Result<GeneratedParams> {
        let mut params = Vec::new();

        let extract = if let Some(field) = field {
            input_field_getter_gen(self.types, &self.root_ty, self.base_extract(), field)?
        } else {
            FieldExtract {
                // openapi cannot have cross-field expressions on parameters. so it doesnt matter
                // if we keep the cel exprs.
                cel: CelExpressions::default(),
                tokens: self.base_extract(),
                is_optional: false,
                ty: ProtoType::Value(self.root_ty.clone()),
            }
        };

        let exclude_paths = if let Some(field) = field {
            match self.used_paths.get(field) {
                Some(ExcludePaths::Child(c)) => Some(c),
                Some(ExcludePaths::True) => anyhow::bail!("{field} is already used by another operation"),
                None => None,
            }
        } else {
            Some(&self.used_paths)
        };

        if extract.ty.nested() {
            anyhow::bail!("query string cannot be used on nested types.")
        }

        let message_ty = match extract.ty.value_type() {
            Some(ProtoValueType::Message(path)) => self.types.get_message(path).unwrap(),
            Some(ProtoValueType::WellKnown(ProtoWellKnownType::Empty)) => {
                return Ok(GeneratedParams::default());
            }
            _ => anyhow::bail!("query string can only be used on message types."),
        };

        for (name, field) in &message_ty.fields {
            let exclude_paths = match exclude_paths.and_then(|exclude_paths| exclude_paths.get(name)) {
                Some(ExcludePaths::True) => continue,
                Some(ExcludePaths::Child(child)) => Some(child),
                None => None,
            };
            params.push(
                openapiv3_1::path::Parameter::builder()
                    .name(field.options.serde_name.clone())
                    .required(!field.options.serde_omittable.is_true())
                    .explode(true)
                    .style(openapiv3_1::path::ParameterStyle::DeepObject)
                    .schema(generate(
                        self.components,
                        self.types,
                        exclude_paths.unwrap_or(&BTreeMap::new()),
                        &field.options.cel_exprs,
                        field.ty.clone(),
                        GenerateDirection::Input,
                        BytesEncoding::Base64,
                    )?)
                    .parameter_in(openapiv3_1::path::ParameterIn::Query)
                    .build(),
            )
        }

        let extract = &extract.tokens;
        let state_ident = &self.state_ident;

        Ok(GeneratedParams {
            params,
            tokens: quote!({
                let (mut tracker, mut target) = #extract;
                if let Err(err) = ::tinc::__private::deserialize_query_string(
                    &parts,
                    tracker,
                    target,
                    &mut #state_ident,
                ) {
                    return err;
                }
            }),
        })
    }

    pub(super) fn generate_path_parameter(&mut self, path: &str) -> anyhow::Result<GeneratedParams> {
        let params = parse_route(path);
        if params.is_empty() {
            return Ok(GeneratedParams::default());
        }

        let PathFields {
            defs,
            mappings,
            param_schemas,
        } = path_struct(self.types, &self.root_ty, self.package, &params, self.base_extract())?;
        let mut params = Vec::new();

        for (path, (ty, cel)) in param_schemas {
            self.consume_field(&path)?;

            params.push(
                openapiv3_1::path::Parameter::builder()
                    .name(path)
                    .required(true)
                    .schema(generate(
                        self.components,
                        self.types,
                        &BTreeMap::new(),
                        &cel,
                        ProtoType::Value(ty.clone()),
                        GenerateDirection::Input,
                        BytesEncoding::Base64,
                    )?)
                    .parameter_in(openapiv3_1::path::ParameterIn::Path)
                    .build(),
            )
        }

        Ok(GeneratedParams {
            params,
            tokens: quote!({
                #[derive(::tinc::reexports::serde::Deserialize)]
                #[allow(non_snake_case, dead_code)]
                struct ____PathContent {
                    #(#defs),*
                }

                let path = match ::tinc::__private::deserialize_path::<____PathContent>(&mut parts).await {
                    Ok(path) => path,
                    Err(err) => return err,
                };

                #(#mappings)*
            }),
        })
    }

    pub(super) fn generate_body(
        &mut self,
        cel: &[CelExpression],
        body_method: BodyMethod,
        field: Option<&str>,
        content_type_field: Option<&str>,
    ) -> anyhow::Result<GeneratedBody<openapiv3_1::request_body::RequestBody>> {
        let content_type = if let Some(content_type_field) = content_type_field {
            self.consume_field(content_type_field)?;
            let extract = input_field_getter_gen(self.types, &self.root_ty, self.base_extract(), content_type_field)?;

            anyhow::ensure!(
                matches!(extract.ty.value_type(), Some(ProtoValueType::String)),
                "content-type must be a string type"
            );

            anyhow::ensure!(!extract.ty.nested(), "content-type cannot be nested");

            let modifier = if extract.is_optional {
                quote! {
                    tracker.get_or_insert_default();
                    target.insert(ct.into());
                }
            } else {
                quote! {
                    let _ = tracker;
                    *target = ct.into();
                }
            };

            let extract = extract.tokens;

            quote! {
                if let Some(ct) = parts.headers.get(::tinc::reexports::http::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()) {
                    let (mut tracker, mut target) = #extract;
                    #modifier
                }
            }
        } else {
            TokenStream::new()
        };

        let exclude_paths = if let Some(field) = field {
            match self.used_paths.get(field) {
                Some(ExcludePaths::Child(c)) => Some(c),
                Some(ExcludePaths::True) => anyhow::bail!("{field} is already used by another operation"),
                None => None,
            }
        } else {
            Some(&self.used_paths)
        };

        let extract = if let Some(field) = field {
            input_field_getter_gen(self.types, &self.root_ty, self.base_extract(), field)?
        } else {
            FieldExtract {
                cel: CelExpressions {
                    field: cel.to_vec(),
                    ..Default::default()
                },
                is_optional: false,
                tokens: self.base_extract(),
                ty: ProtoType::Value(self.root_ty.clone()),
            }
        };

        match body_method {
            BodyMethod::Json => {}
            BodyMethod::Binary(_) => {
                anyhow::ensure!(
                    matches!(extract.ty.value_type(), Some(ProtoValueType::Bytes)),
                    "binary bodies must be on bytes fields."
                );

                anyhow::ensure!(!extract.ty.nested(), "binary bodies cannot be nested");
            }
            BodyMethod::Text => {
                anyhow::ensure!(
                    matches!(extract.ty.value_type(), Some(ProtoValueType::String)),
                    "text bodies must be on string fields."
                );

                anyhow::ensure!(!extract.ty.nested(), "text bodies cannot be nested");
            }
        }

        let func = body_method.deserialize_method();
        let tokens = &extract.tokens;
        let state_ident = &self.state_ident;

        Ok(GeneratedBody {
            tokens: quote! {{
                #content_type
                let (tracker, target) = #tokens;
                if let Err(err) = ::tinc::__private::#func(&parts, body, tracker, target, &mut #state_ident).await {
                    return err;
                }
            }},
            body: openapiv3_1::request_body::RequestBody::builder()
                .content(
                    body_method.content_type(),
                    openapiv3_1::content::Content::new(Some(generate(
                        self.components,
                        self.types,
                        exclude_paths.unwrap_or(&BTreeMap::new()),
                        &extract.cel,
                        extract.ty,
                        GenerateDirection::Input,
                        body_method.bytes_encoding(),
                    )?)),
                )
                .build(),
        })
    }
}

impl OutputGenerator<'_> {
    fn base_extract(&self) -> TokenStream {
        let response_ident = &self.response_ident;
        quote!((&#response_ident))
    }

    pub(super) fn generate_body(
        &mut self,
        body_method: BodyMethod,
        field: Option<&str>,
        content_type_field: Option<&str>,
    ) -> anyhow::Result<GeneratedBody<openapiv3_1::response::Response>> {
        let builder_ident = &self.builder_ident;

        let content_type = if let Some(content_type_field) = content_type_field {
            let extract = output_field_getter_gen(self.types, &self.root_ty, self.base_extract(), content_type_field)?;

            anyhow::ensure!(
                matches!(extract.ty.value_type(), Some(ProtoValueType::String)),
                "content-type must be a string type"
            );

            anyhow::ensure!(!extract.ty.nested(), "content-type cannot be nested");

            let modifier = if extract.is_optional { quote!(Some(ct)) } else { quote!(ct) };

            let extract = extract.tokens;
            let default_ct = body_method.default_content_type();

            quote! {
                if let #modifier = #extract {
                    #builder_ident.header(::tinc::reexports::http::header::CONTENT_TYPE, ct)
                } else {
                    #builder_ident.header(::tinc::reexports::http::header::CONTENT_TYPE, #default_ct)
                }
            }
        } else {
            let default_ct = body_method.default_content_type();
            quote! {
                #builder_ident.header(::tinc::reexports::http::header::CONTENT_TYPE, #default_ct)
            }
        };

        let extract = if let Some(field) = field {
            output_field_getter_gen(self.types, &self.root_ty, self.base_extract(), field)?
        } else {
            FieldExtract {
                cel: CelExpressions::default(),
                is_optional: false,
                tokens: self.base_extract(),
                ty: ProtoType::Value(self.root_ty.clone()),
            }
        };

        let tokens = extract.tokens;

        let tokens = match body_method {
            BodyMethod::Json => quote!({
                let mut writer = ::tinc::reexports::bytes::BufMut::writer(
                    ::tinc::reexports::bytes::BytesMut::with_capacity(128)
                );
                match ::tinc::reexports::serde_json::to_writer(&mut writer, #tokens) {
                    ::core::result::Result::Ok(()) => {},
                    ::core::result::Result::Err(err) => return ::tinc::__private::handle_response_build_error(err),
                }
                (#content_type)
                    .body(::tinc::reexports::axum::body::Body::from(writer.into_inner().freeze()))
            }),
            BodyMethod::Binary(_) => {
                anyhow::ensure!(
                    matches!(extract.ty.value_type(), Some(ProtoValueType::Bytes)),
                    "binary bodies must be on bytes fields."
                );

                anyhow::ensure!(!extract.ty.nested(), "binary bodies cannot be nested");

                let matcher = if extract.is_optional {
                    quote!(Some(bytes))
                } else {
                    quote!(bytes)
                };

                quote!({
                    (#content_type)
                        .body(if let #matcher = #tokens {
                            ::tinc::reexports::axum::body::Body::from(bytes.clone())
                        } else {
                            ::tinc::reexports::axum::body::Body::empty()
                        })
                })
            }
            BodyMethod::Text => {
                anyhow::ensure!(
                    matches!(extract.ty.value_type(), Some(ProtoValueType::String)),
                    "text bodies must be on string fields."
                );

                anyhow::ensure!(!extract.ty.nested(), "text bodies cannot be nested");

                let matcher = if extract.is_optional {
                    quote!(Some(text))
                } else {
                    quote!(text)
                };

                quote!({
                    (#content_type)
                        .body(if let #matcher = #tokens {
                            ::tinc::reexports::axum::body::Body::from(text.clone())
                        } else {
                            ::tinc::reexports::axum::body::Body::empty()
                        })
                })
            }
        };

        Ok(GeneratedBody {
            tokens,
            body: openapiv3_1::Response::builder()
                .content(
                    body_method.content_type(),
                    openapiv3_1::Content::new(Some(generate(
                        self.components,
                        self.types,
                        &BTreeMap::new(),
                        &extract.cel,
                        extract.ty,
                        GenerateDirection::Output,
                        body_method.bytes_encoding(),
                    )?)),
                )
                .description("")
                .build(),
        })
    }
}

fn generate(
    components: &mut openapiv3_1::Components,
    types: &ProtoTypeRegistry,
    used_paths: &BTreeMap<String, ExcludePaths>,
    cel: &CelExpressions,
    ty: ProtoType,
    direction: GenerateDirection,
    bytes: BytesEncoding,
) -> anyhow::Result<Schema> {
    fn internal_generate(
        components: &mut openapiv3_1::Components,
        types: &ProtoTypeRegistry,
        used_paths: &BTreeMap<String, ExcludePaths>,
        cel: &CelExpressions,
        ty: ProtoType,
        direction: GenerateDirection,
        bytes: BytesEncoding,
    ) -> anyhow::Result<Schema> {
        let mut schemas = Vec::new();

        let compiler = Compiler::new(types);
        if !matches!(ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_))) {
            for expr in &cel.field {
                schemas.extend(handle_expr(compiler.child(), &ty, expr)?);
            }
        }

        schemas.push(match ty {
            ProtoType::Modified(ProtoModifiedValueType::Map(key, value)) => Schema::object(
                Object::builder()
                    .schema_type(Type::Object)
                    .property_names(match key {
                        ProtoValueType::String => {
                            let mut schemas = Vec::with_capacity(1 + cel.map_key.len());

                            for expr in &cel.map_key {
                                schemas.extend(handle_expr(compiler.child(), &ProtoType::Value(key.clone()), expr)?);
                            }

                            schemas.push(Schema::object(Object::builder().schema_type(Type::String)));

                            Object::all_ofs(schemas)
                        }
                        ProtoValueType::Int32 | ProtoValueType::Int64 => {
                            Object::builder().schema_type(Type::String).pattern("^-?[0-9]+$").build()
                        }
                        ProtoValueType::UInt32 | ProtoValueType::UInt64 => {
                            Object::builder().schema_type(Type::String).pattern("^[0-9]+$").build()
                        }
                        ProtoValueType::Bool => Object::builder()
                            .schema_type(Type::String)
                            .enum_values(["true", "false"])
                            .build(),
                        _ => Object::builder().schema_type(Type::String).build(),
                    })
                    .additional_properties({
                        let mut schemas = Vec::with_capacity(1 + cel.map_value.len());
                        for expr in &cel.map_value {
                            schemas.extend(handle_expr(compiler.child(), &ProtoType::Value(value.clone()), expr)?);
                        }

                        schemas.push(internal_generate(
                            components,
                            types,
                            &BTreeMap::new(),
                            &CelExpressions::default(),
                            ProtoType::Value(value),
                            direction,
                            bytes,
                        )?);

                        Object::all_ofs(schemas)
                    })
                    .build(),
            ),
            ProtoType::Modified(ProtoModifiedValueType::Repeated(item)) => Schema::object(
                Object::builder()
                    .schema_type(Type::Array)
                    .items(internal_generate(
                        components,
                        types,
                        used_paths,
                        cel,
                        ProtoType::Value(item),
                        direction,
                        bytes,
                    )?)
                    .build(),
            ),
            ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) => Schema::object(
                Object::builder()
                    .schema_type(Type::Object)
                    .title(oneof.full_name.to_string())
                    .one_ofs(if let Some(tagged) = oneof.options.tagged {
                        oneof
                            .fields
                            .into_iter()
                            .filter(|(_, field)| match direction {
                                GenerateDirection::Input => field.options.visibility.has_input(),
                                GenerateDirection::Output => field.options.visibility.has_output(),
                            })
                            .map(|(name, field)| {
                                let ty = internal_generate(
                                    components,
                                    types,
                                    &BTreeMap::new(),
                                    &field.options.cel_exprs,
                                    ProtoType::Value(field.ty),
                                    direction,
                                    bytes,
                                )?;

                                anyhow::Ok(Schema::object(
                                    Object::builder()
                                        .schema_type(Type::Object)
                                        .title(name)
                                        .description(field.comments.to_string())
                                        .properties({
                                            let mut properties = IndexMap::new();
                                            properties.insert(
                                                tagged.tag.clone(),
                                                Schema::object(
                                                    Object::builder()
                                                        .schema_type(Type::String)
                                                        .const_value(field.options.serde_name)
                                                        .build(),
                                                ),
                                            );
                                            properties.insert(tagged.content.clone(), ty);
                                            properties
                                        })
                                        .unevaluated_properties(false)
                                        .build(),
                                ))
                            })
                            .collect::<anyhow::Result<Vec<_>>>()?
                    } else {
                        oneof
                            .fields
                            .into_iter()
                            .filter(|(_, field)| match direction {
                                GenerateDirection::Input => field.options.visibility.has_input(),
                                GenerateDirection::Output => field.options.visibility.has_output(),
                            })
                            .map(|(name, field)| {
                                let ty = internal_generate(
                                    components,
                                    types,
                                    &BTreeMap::new(),
                                    &field.options.cel_exprs,
                                    ProtoType::Value(field.ty),
                                    direction,
                                    bytes,
                                )?;

                                anyhow::Ok(Schema::object(
                                    Object::builder()
                                        .schema_type(Type::Object)
                                        .title(name)
                                        .description(field.comments.to_string())
                                        .properties({
                                            let mut properties = IndexMap::new();
                                            properties.insert(field.options.serde_name, ty);
                                            properties
                                        })
                                        .unevaluated_properties(false)
                                        .build(),
                                ))
                            })
                            .collect::<anyhow::Result<Vec<_>>>()?
                    })
                    .unevaluated_properties(false)
                    .build(),
            ),
            ProtoType::Modified(ProtoModifiedValueType::Optional(value)) => Schema::object(
                Object::builder()
                    .one_ofs([
                        Schema::object(Object::builder().schema_type(Type::Null).build()),
                        internal_generate(components, types, used_paths, cel, ProtoType::Value(value), direction, bytes)?,
                    ])
                    .build(),
            ),
            ProtoType::Value(ProtoValueType::Bool) => Schema::object(Object::builder().schema_type(Type::Boolean).build()),
            ProtoType::Value(ProtoValueType::Bytes) => Schema::object(
                Object::builder()
                    .schema_type(Type::String)
                    .content_encoding(match bytes {
                        BytesEncoding::Base64 => "base64",
                        BytesEncoding::Binary => "binary",
                    })
                    .build(),
            ),
            ProtoType::Value(ProtoValueType::Double | ProtoValueType::Float) => {
                Schema::object(Object::builder().schema_type(Type::Number).build())
            }
            ProtoType::Value(ProtoValueType::Int32) => Schema::object(Object::int32()),
            ProtoType::Value(ProtoValueType::UInt32) => Schema::object(Object::uint32()),
            ProtoType::Value(ProtoValueType::Int64) => Schema::object(Object::int64()),
            ProtoType::Value(ProtoValueType::UInt64) => Schema::object(Object::uint64()),
            ProtoType::Value(ProtoValueType::String) => Schema::object(Object::builder().schema_type(Type::String).build()),
            ProtoType::Value(ProtoValueType::Enum(enum_path)) => {
                let ety = types
                    .get_enum(&enum_path)
                    .with_context(|| format!("missing enum: {enum_path}"))?;
                let schema_name = if ety
                    .variants
                    .values()
                    .any(|v| v.options.visibility.has_input() != v.options.visibility.has_output())
                {
                    format!("{direction:?}.{enum_path}")
                } else {
                    enum_path.to_string()
                };

                if !components.schemas.contains_key(enum_path.as_ref()) {
                    components.add_schema(
                        schema_name.clone(),
                        Schema::object(
                            Object::builder()
                                .schema_type(if ety.options.repr_enum { Type::Integer } else { Type::String })
                                .enum_values(
                                    ety.variants
                                        .values()
                                        .filter(|v| match direction {
                                            GenerateDirection::Input => v.options.visibility.has_input(),
                                            GenerateDirection::Output => v.options.visibility.has_output(),
                                        })
                                        .map(|v| {
                                            if ety.options.repr_enum {
                                                serde_json::Value::from(v.value)
                                            } else {
                                                serde_json::Value::from(v.options.serde_name.clone())
                                            }
                                        })
                                        .collect::<Vec<_>>(),
                                )
                                .title(enum_path.to_string())
                                .description(ety.comments.to_string())
                                .build(),
                        ),
                    );
                }

                Schema::object(Ref::from_schema_name(schema_name))
            }
            ref ty @ ProtoType::Value(ProtoValueType::Message(ref message_path)) => {
                let message_ty = types
                    .get_message(message_path)
                    .with_context(|| format!("missing message: {message_path}"))?;

                let schema_name = if message_ty
                    .fields
                    .values()
                    .any(|v| v.options.visibility.has_input() != v.options.visibility.has_output())
                {
                    format!("{direction:?}.{message_path}")
                } else {
                    message_path.to_string()
                };

                if !components.schemas.contains_key(&schema_name) || !used_paths.is_empty() {
                    if used_paths.is_empty() {
                        components.schemas.insert(schema_name.clone(), Schema::Bool(false));
                    }
                    let mut properties = IndexMap::new();
                    let mut required = Vec::new();
                    let mut schemas = Vec::with_capacity(1);

                    for expr in &message_ty.options.cel {
                        schemas.extend(handle_expr(compiler.child(), ty, expr)?);
                    }

                    for (name, field) in message_ty.fields.iter().filter(|(_, field)| match direction {
                        GenerateDirection::Input => field.options.visibility.has_input(),
                        GenerateDirection::Output => field.options.visibility.has_output(),
                    }) {
                        let exclude_paths = match used_paths.get(name) {
                            Some(ExcludePaths::True) => continue,
                            Some(ExcludePaths::Child(child)) => Some(child),
                            None => None,
                        };
                        if !field.options.serde_omittable.is_true() {
                            required.push(field.options.serde_name.clone());
                        }

                        let ty = match (!field.options.nullable || field.options.flatten, &field.ty) {
                            (true, ProtoType::Modified(ProtoModifiedValueType::Optional(ty))) => {
                                ProtoType::Value(ty.clone())
                            }
                            _ => field.ty.clone(),
                        };

                        let field_schema = internal_generate(
                            components,
                            types,
                            exclude_paths.unwrap_or(&BTreeMap::new()),
                            &field.options.cel_exprs,
                            ty,
                            direction,
                            bytes,
                        )?;

                        if field.options.flatten {
                            schemas.push(field_schema);
                        } else {
                            let schema = if field.options.nullable
                                && !matches!(&field.ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_)))
                            {
                                Schema::object(
                                    Object::builder()
                                        .one_ofs([Object::builder().schema_type(Type::Null).build().into(), field_schema])
                                        .build(),
                                )
                            } else {
                                field_schema
                            };

                            properties.insert(
                                field.options.serde_name.clone(),
                                Schema::object(Object::all_ofs([
                                    schema,
                                    Schema::object(Object::builder().description(field.comments.to_string()).build()),
                                ])),
                            );
                        }
                    }

                    schemas.push(Schema::object(
                        Object::builder()
                            .schema_type(Type::Object)
                            .title(message_path.to_string())
                            .description(message_ty.comments.to_string())
                            .properties(properties)
                            .required(required)
                            .unevaluated_properties(false)
                            .build(),
                    ));

                    if used_paths.is_empty() {
                        components.add_schema(schema_name.clone(), Object::all_ofs(schemas).into_optimized());
                        Schema::object(Ref::from_schema_name(schema_name))
                    } else {
                        Schema::object(Object::all_ofs(schemas))
                    }
                } else {
                    Schema::object(Ref::from_schema_name(schema_name))
                }
            }
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp)) => {
                Schema::object(Object::builder().schema_type(Type::String).format("date-time").build())
            }
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Duration)) => {
                Schema::object(Object::builder().schema_type(Type::String).format("duration").build())
            }
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Empty)) => Schema::object(
                Object::builder()
                    .schema_type(Type::Object)
                    .unevaluated_properties(false)
                    .build(),
            ),
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::ListValue)) => {
                Schema::object(Object::builder().schema_type(Type::Array).build())
            }
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Value)) => Schema::object(
                Object::builder()
                    .schema_type(vec![
                        Type::Null,
                        Type::Boolean,
                        Type::Object,
                        Type::Array,
                        Type::Number,
                        Type::String,
                    ])
                    .build(),
            ),
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Struct)) => {
                Schema::object(Object::builder().schema_type(Type::Object).build())
            }
            ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Any)) => Schema::object(
                Object::builder()
                    .schema_type(Type::Object)
                    .property("@type", Object::builder().schema_type(Type::String))
                    .build(),
            ),
        });

        Ok(Schema::object(Object::all_ofs(schemas)))
    }

    internal_generate(components, types, used_paths, cel, ty, direction, bytes).map(|schema| schema.into_optimized())
}
