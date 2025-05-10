use std::collections::BTreeMap;

use anyhow::Context;
use indexmap::IndexMap;
use openapi::{
    BytesEncoding, GenerateDirection, exclude_path, generate_optimized, generate_path_parameter, generate_query_parameter,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, parse_quote};
use tinc_pb::http_endpoint_options;

use super::Package;
use super::cel::CelExpressions;
use super::utils::{field_ident_from_str, type_ident_from_str};
use crate::types::{
    Comments, ProtoMessageType, ProtoModifiedValueType, ProtoPath, ProtoService, ProtoServiceMethod,
    ProtoServiceMethodEndpoint, ProtoServiceMethodIo, ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoWellKnownType,
};

mod openapi;
mod optimize;

pub(crate) use openapi::SchemaRegistry;

enum BodyMethod {
    Bytes,
    Json,
    Text,
}

impl BodyMethod {
    fn deserialize_func(&self) -> TokenStream {
        match self {
            Self::Bytes => quote!(deserialize_body_bytes),
            Self::Json => quote!(deserialize_body_json),
            Self::Text => quote!(deserialize_body_text),
        }
    }

    fn from_io(io: &ProtoServiceMethodIo) -> anyhow::Result<Self> {
        match &io {
            ProtoServiceMethodIo::Single(ProtoValueType::Bytes) => Ok(Self::Bytes),
            ProtoServiceMethodIo::Single(ProtoValueType::String) => Ok(Self::Text),
            ProtoServiceMethodIo::Single(_) => Ok(Self::Json),
            ProtoServiceMethodIo::Stream(_) => {
                anyhow::bail!("currently streams are not supported for tinc methods")
            }
        }
    }

    fn from_ty(ty: &ProtoType) -> Self {
        match ty {
            ProtoType::Value(ProtoValueType::Bytes)
            | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Bytes)) => BodyMethod::Bytes,
            ProtoType::Value(ProtoValueType::String)
            | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::String)) => BodyMethod::Text,
            _ => BodyMethod::Json,
        }
    }

    fn content_type(&self) -> &'static str {
        match self {
            BodyMethod::Bytes => "application/octet-stream",
            BodyMethod::Json => "application/json",
            BodyMethod::Text => "text/plain",
        }
    }
}

struct PathFields {
    defs: Vec<proc_macro2::TokenStream>,
    mappings: Vec<proc_macro2::TokenStream>,
    param_schemas: BTreeMap<String, (ProtoValueType, CelExpressions)>,
}

struct FieldExtract {
    tokens: proc_macro2::TokenStream,
    ty: ProtoType,
    cel: CelExpressions,
    is_optional: bool,
}

fn tracker_field_extractor_generator(
    field_str: &str,
    registry: &ProtoTypeRegistry,
    message: &ProtoMessageType,
) -> anyhow::Result<FieldExtract> {
    let mut next_message = Some(message);
    let mut is_optional = false;
    let mut kind = None;
    let mut cel = None;
    let mut mapping = quote! {(&mut tracker, &mut target)};
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

fn field_extractor_generator(
    field_str: &str,
    registry: &ProtoTypeRegistry,
    message: &ProtoMessageType,
) -> anyhow::Result<FieldExtract> {
    let mut next_message = Some(message);
    let mut was_optional = false;
    let mut kind = None;
    let mut cel = None;
    let mut mapping = quote!(&body);
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
            (false, false) => quote!(#mapping.#field_ident),
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

fn path_struct(
    ty: &ProtoValueType,
    package: &str,
    fields: &[String],
    registry: &ProtoTypeRegistry,
) -> anyhow::Result<PathFields> {
    let mut defs = Vec::new();
    let mut mappings = Vec::new();
    let mut param_schemas = BTreeMap::new();

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
        ProtoValueType::Message(message) => {
            let message = registry.get_message(message).expect("message not found");

            for (idx, field) in fields.iter().enumerate() {
                let field_str = field.as_ref();
                let path_field_ident = quote::format_ident!("field_{idx}");
                let FieldExtract { cel, tokens, ty, .. } = tracker_field_extractor_generator(field_str, registry, message)?;

                let setter = match &ty {
                    ProtoType::Modified(ProtoModifiedValueType::Optional(_)) => quote! {
                        tracker.get_or_insert_default();
                        target.insert(path.#path_field_ident.into());
                    },
                    _ => quote! {
                        *target = path.#path_field_ident.into();
                    },
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

            mappings.push(quote! {
                *target = path.value.into();
            });

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

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct OpenAPIMethod {
    pub path: String,
    pub http_verb: String,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub parameters: Vec<serde_json::Value>,
}

struct GeneratedMethod {
    function_body: proc_macro2::TokenStream,
    openapi: OpenAPIMethod,
    http_method: Ident,
    path: String,
}

impl GeneratedMethod {
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: &str,
        package: &str,
        service: &ProtoService,
        method: &ProtoServiceMethod,
        endpoint: &ProtoServiceMethodEndpoint,
        registry: &ProtoTypeRegistry,
        schema: &mut SchemaRegistry,
    ) -> anyhow::Result<GeneratedMethod> {
        let (http_method_str, path) = match &endpoint.method {
            tinc_pb::http_endpoint_options::Method::Get(path) => ("get", path),
            tinc_pb::http_endpoint_options::Method::Post(path) => ("post", path),
            tinc_pb::http_endpoint_options::Method::Put(path) => ("put", path),
            tinc_pb::http_endpoint_options::Method::Delete(path) => ("delete", path),
            tinc_pb::http_endpoint_options::Method::Patch(path) => ("patch", path),
        };

        let trimmed_path = path.trim_start_matches('/');
        let full_path = if let Some(prefix) = &service.options.prefix {
            format!("/{}/{}", prefix.trim_end_matches('/'), trimmed_path)
        } else {
            format!("/{trimmed_path}")
        };

        let http_method = quote::format_ident!("{http_method_str}");
        let mut used_paths = BTreeMap::new();
        let params = parse_route(&full_path);
        params.iter().try_for_each(|param| exclude_path(&mut used_paths, param))?;

        let mut openapi = OpenAPIMethod {
            http_verb: http_method_str.to_owned(),
            path: full_path.clone(),
            input: None,
            output: None,
            parameters: Vec::new(),
        };

        let path_params = if !params.is_empty() {
            let PathFields {
                defs,
                mappings,
                param_schemas,
            } = path_struct(method.input.value_type(), package, &params, registry)
                .with_context(|| format!("failed to generate path struct for method: {name}"))?;

            openapi
                .parameters
                .extend(generate_path_parameter(registry, schema, &param_schemas)?);

            quote!({
                let mut tracker = &mut tracker;
                let mut target = &mut target;

                #[derive(::tinc::reexports::serde::Deserialize)]
                #[allow(non_snake_case, dead_code)]
                struct PathContent {
                    #(#defs),*
                }

                let path = match ::tinc::__private::deserialize_path::<PathContent>(&mut parts).await {
                    Ok(path) => path,
                    Err(err) => return err,
                };

                #(#mappings)*
            })
        } else {
            quote! {}
        };

        let is_get_or_delete = matches!(http_method_str, "get" | "delete");
        let input = endpoint.input.clone().unwrap_or_else(|| {
            if is_get_or_delete {
                http_endpoint_options::Input::Query(http_endpoint_options::QueryParams::default())
            } else {
                http_endpoint_options::Input::Body(http_endpoint_options::RequestBody::default())
            }
        });

        let input = match input {
            http_endpoint_options::Input::Query(http_endpoint_options::QueryParams { field }) => {
                let extract = match &method.input {
                    ProtoServiceMethodIo::Single(ProtoValueType::Message(path)) if field.is_empty() => {
                        openapi.parameters.extend(generate_query_parameter(
                            registry,
                            schema,
                            registry
                                .get_message(path)
                                .with_context(|| format!("missing message: {path}"))?,
                            &used_paths,
                        )?);
                        quote! {}
                    }
                    ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) => {
                        exclude_path(&mut used_paths, &field).context("query field")?;
                        let message = registry.get_message(message).expect("message not found");
                        let FieldExtract { tokens, ty, .. } = tracker_field_extractor_generator(&field, registry, message)?;
                        match ty {
                            ProtoType::Value(ProtoValueType::Message(path))
                            | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Message(path))) => {
                                openapi.parameters.extend(generate_query_parameter(
                                    registry,
                                    schema,
                                    registry
                                        .get_message(&path)
                                        .with_context(|| format!("missing message: {path}"))?,
                                    &BTreeMap::new(),
                                )?)
                            }
                            _ => anyhow::bail!("query string input requires a message type as the method input"),
                        }
                        tokens
                    }
                    ProtoServiceMethodIo::Stream(_) => anyhow::bail!("streams currently are not supported by tinc"),
                    ProtoServiceMethodIo::Single(
                        ProtoValueType::Bool
                        | ProtoValueType::String
                        | ProtoValueType::Bytes
                        | ProtoValueType::Int32
                        | ProtoValueType::Int64
                        | ProtoValueType::UInt32
                        | ProtoValueType::UInt64
                        | ProtoValueType::Double
                        | ProtoValueType::Float
                        | ProtoValueType::WellKnown(_)
                        | ProtoValueType::Enum(_),
                    ) => anyhow::bail!("query string input requires a message type as the method input"),
                };

                quote!({
                    let mut tracker = &mut tracker;
                    let mut target = &mut target;

                    #extract
                    if let Err(err) = ::tinc::__private::deserialize_query_string(
                        &parts,
                        tracker,
                        target,
                        &mut state,
                    ) {
                        return err;
                    }
                })
            }
            http_endpoint_options::Input::Body(http_endpoint_options::RequestBody {
                field,
                content_type_field,
            }) => {
                let content_type = if !content_type_field.is_empty() {
                    exclude_path(&mut used_paths, &content_type_field).context("content-type field")?;

                    if content_type_field == field {
                        anyhow::bail!("content type field cannot be the same as the body field");
                    }

                    let FieldExtract { tokens, ty, .. } = match &method.input {
                        ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) => {
                            let message = registry.get_message(message).expect("message not found");
                            tracker_field_extractor_generator(&field, registry, message)?
                        }
                        _ => anyhow::bail!("content_type_field is only supported on methods who have a message input."),
                    };

                    let modifier = match &ty {
                        ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::String)) => quote! {
                            let (mut tracker, mut target) = #tokens;
                            tracker.get_or_insert_default();
                            target.insert(content_type.into());
                        },
                        ProtoType::Value(ProtoValueType::String) => quote! {
                            let (_, mut target) = #tokens;
                            *target = content_type.into();
                        },
                        _ => anyhow::bail!("content type field must be a string: {ty:?}"),
                    };

                    quote!({
                        if let Some(content_type) = parts.headers.get(::tinc::reexports::http::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()) {
                            #modifier
                        }
                    })
                } else {
                    quote!()
                };

                let (tokens, method) = if field.is_empty() {
                    let body_method = BodyMethod::from_io(&method.input)?;
                    openapi.input = Some(generate_optimized(
                        registry,
                        schema,
                        ProtoType::Value(method.input.value_type().clone()),
                        &CelExpressions {
                            field: method.cel.clone(),
                            ..Default::default()
                        },
                        &used_paths,
                        GenerateDirection::Input,
                        match body_method {
                            BodyMethod::Bytes => BytesEncoding::Binary,
                            BodyMethod::Json | BodyMethod::Text => BytesEncoding::Base64,
                        },
                    )?);
                    (quote! {}, body_method)
                } else if let ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) = &method.input {
                    exclude_path(&mut used_paths, &field).context("body field")?;
                    let message = registry.get_message(message).expect("message not found");
                    let FieldExtract { tokens, ty, cel, .. } = tracker_field_extractor_generator(&field, registry, message)?;
                    let body_method = BodyMethod::from_ty(&ty);
                    openapi.input = Some(generate_optimized(
                        registry,
                        schema,
                        ty,
                        &cel,
                        &BTreeMap::new(),
                        GenerateDirection::Input,
                        match body_method {
                            BodyMethod::Bytes => BytesEncoding::Binary,
                            BodyMethod::Json | BodyMethod::Text => BytesEncoding::Base64,
                        },
                    )?);
                    (tokens, body_method)
                } else {
                    anyhow::bail!("nested fields are not supported on non message types.");
                };

                let de_func = method.deserialize_func();

                let body = quote!({
                    #tokens
                    if let Err(err) = ::tinc::__private::#de_func(&parts, body, tracker, target, &mut state).await {
                        return err;
                    }
                });

                quote!({
                    let mut tracker = &mut tracker;
                    let mut target = &mut target;

                    #body
                    #content_type
                })
            }
        };

        let input_path = match &method.input {
            ProtoServiceMethodIo::Single(input) => registry.resolve_rust_path(package, input.proto_path()),
            ProtoServiceMethodIo::Stream(_) => anyhow::bail!("currently streaming is not supported by tinc methods."),
        };

        let service_method_name = field_ident_from_str(name);

        let response = endpoint.response.clone().unwrap_or_default();

        let (tokens, output_body_method, optional) = if response.field.is_empty() {
            let body_method = BodyMethod::from_io(&method.output)?;
            openapi.output = Some(generate_optimized(
                registry,
                schema,
                ProtoType::Value(method.output.value_type().clone()),
                &CelExpressions::default(),
                &BTreeMap::default(),
                GenerateDirection::Output,
                match body_method {
                    BodyMethod::Bytes => BytesEncoding::Binary,
                    BodyMethod::Json | BodyMethod::Text => BytesEncoding::Base64,
                },
            )?);
            (quote!(&body), body_method, false)
        } else if let ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) = &method.output {
            let message = registry.get_message(message).expect("message not found");
            let FieldExtract {
                tokens, ty, is_optional, ..
            } = field_extractor_generator(&response.field, registry, message)?;
            let body_method = BodyMethod::from_ty(&ty);
            openapi.output = Some(generate_optimized(
                registry,
                schema,
                ProtoType::Value(method.output.value_type().clone()),
                &CelExpressions::default(),
                &BTreeMap::default(),
                GenerateDirection::Output,
                match body_method {
                    BodyMethod::Bytes => BytesEncoding::Binary,
                    BodyMethod::Json | BodyMethod::Text => BytesEncoding::Base64,
                },
            )?);
            (tokens, body_method, is_optional)
        } else {
            anyhow::bail!("nested fields are not supported on non message types.");
        };

        let ct = output_body_method.content_type();
        let ct = quote!({
            response_builder = response_builder.header(::tinc::reexports::http::header::CONTENT_TYPE, #ct);
        });

        let content_type = if !response.content_type_field.is_empty() {
            let ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) = &method.output else {
                anyhow::bail!("content-type field can only be used on message types.");
            };

            let message = registry.get_message(message).expect("message not found");
            let FieldExtract { tokens, ty, .. } = field_extractor_generator(&response.field, registry, message)?;
            let matcher = if optional { quote!(Some(ct)) } else { quote!(ct) };
            if !matches!(
                ty,
                ProtoType::Value(ProtoValueType::String)
                    | ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::String))
            ) {
                anyhow::bail!("content-type field must be a string");
            }

            quote!({
                if let #matcher = #tokens {
                    response_builder = response_builder.header(::tinc::reexports::http::header::CONTENT_TYPE, ct);
                } else #ct
            })
        } else {
            ct
        };

        let response = {
            let matcher = if optional { quote!(Some(body)) } else { quote!(body) };
            let body = match output_body_method {
                BodyMethod::Text | BodyMethod::Bytes => quote!({
                    if let #matcher = #tokens {
                        ::tinc::reexports::axum::body::Body::from(body.clone())
                    } else {
                        ::tinc::reexports::axum::body::Body::empty()
                    }
                }),
                BodyMethod::Json => quote!({
                    let mut writer = ::tinc::reexports::bytes::BufMut::writer(
                        ::tinc::reexports::bytes::BytesMut::with_capacity(128)
                    );
                    match ::tinc::reexports::serde_json::to_writer(&mut writer, #tokens) {
                        ::core::result::Result::Ok(()) => {},
                        ::core::result::Result::Err(err) => return ::tinc::__private::handle_response_build_error(err),
                    }
                    ::tinc::reexports::axum::body::Body::from(writer.into_inner().freeze())
                }),
            };

            quote!({
                let mut response_builder = ::tinc::reexports::http::Response::builder();
                #content_type
                match response_builder
                    .body(#body) {
                        ::core::result::Result::Ok(v) => v,
                        ::core::result::Result::Err(err) => return ::tinc::__private::handle_response_build_error(err),
                    }
            })
        };

        let function_impl = quote! {
            let mut state = ::tinc::__private::TrackerSharedState::default();
            let mut tracker = <<#input_path as ::tinc::__private::TrackerFor>::Tracker as ::core::default::Default>::default();
            let mut target = <#input_path as ::core::default::Default>::default();

            #path_params

            #input

            if let Err(err) = ::tinc::__private::TincValidate::validate_http(&target, state, &tracker) {
                return err;
            }

            let request = ::tinc::reexports::tonic::Request::from_parts(
                ::tinc::reexports::tonic::metadata::MetadataMap::from_headers(parts.headers),
                parts.extensions,
                target,
            );

            let (metadata, body, extensions) = match service.inner.#service_method_name(request).await {
                ::core::result::Result::Ok(response) => response.into_parts(),
                ::core::result::Result::Err(status) => return ::tinc::__private::handle_tonic_status(&status),
            };

            let mut response = #response;
            response.headers_mut().extend(metadata.into_headers());
            *response.extensions_mut() = extensions;

            response
        };

        Ok(GeneratedMethod {
            function_body: function_impl,
            http_method,
            openapi,
            path: full_path,
        })
    }

    pub(crate) fn method_handler(
        &self,
        function_name: &Ident,
        server_module_name: &Ident,
        service_trait: &Ident,
        tinc_struct_name: &Ident,
    ) -> proc_macro2::TokenStream {
        let function_impl = &self.function_body;

        quote! {
            #[allow(non_snake_case, unused_mut, dead_code, unused_variables, unused_parens)]
            async fn #function_name<T>(
                ::tinc::reexports::axum::extract::State(service): ::tinc::reexports::axum::extract::State<#tinc_struct_name<T>>,
                request: ::tinc::reexports::axum::extract::Request,
            ) -> ::tinc::reexports::axum::response::Response
            where
                T: super::#server_module_name::#service_trait,
            {
                let (mut parts, body) = ::tinc::reexports::axum::RequestExt::with_limited_body(request).into_parts();
                #function_impl
            }
        }
    }

    pub(crate) fn route(&self, function_name: &Ident) -> proc_macro2::TokenStream {
        let path = &self.path;
        let http_method = &self.http_method;

        quote! {
            .route(#path, ::tinc::reexports::axum::routing::#http_method(#function_name::<T>))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProcessedService {
    pub full_name: ProtoPath,
    pub package: ProtoPath,
    pub comments: Comments,
    pub methods: IndexMap<String, ProcessedServiceMethod>,
}

impl ProcessedService {
    pub(crate) fn name(&self) -> &str {
        self.full_name
            .strip_prefix(&*self.package)
            .unwrap_or(&self.full_name)
            .trim_matches('.')
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProcessedServiceMethod {
    pub codec_path: ProtoPath,
    pub input: ProtoServiceMethodIo,
    pub output: ProtoServiceMethodIo,
    pub comments: Comments,
    pub openapi: Vec<OpenAPIMethod>,
}

pub(super) fn handle_service(
    service: &ProtoService,
    package: &mut Package,
    registry: &ProtoTypeRegistry,
    schema: &mut SchemaRegistry,
) -> anyhow::Result<()> {
    let name = service
        .full_name
        .strip_prefix(&*service.package)
        .and_then(|s| s.strip_prefix('.'))
        .unwrap_or(&*service.full_name);

    let snake_name = field_ident_from_str(name);
    let pascal_name = type_ident_from_str(name);

    let tinc_module_name = quote::format_ident!("{snake_name}_tinc");
    let server_module_name = quote::format_ident!("{snake_name}_server");
    let tinc_struct_name = quote::format_ident!("{pascal_name}Tinc");

    let mut method_tokens = Vec::new();
    let mut route_tokens = Vec::new();
    let mut method_codecs = Vec::new();
    let mut methods = IndexMap::new();

    let package_name = format!("{}.{tinc_module_name}", service.package);

    for (name, method) in service.methods.iter() {
        let mut openapi = Vec::new();
        for (idx, endpoint) in method.endpoints.iter().enumerate() {
            let gen_method = GeneratedMethod::new(name, &package_name, service, method, endpoint, registry, schema)?;
            let function_name = quote::format_ident!("{name}_{idx}");

            method_tokens.push(gen_method.method_handler(
                &function_name,
                &server_module_name,
                &pascal_name,
                &tinc_struct_name,
            ));
            route_tokens.push(gen_method.route(&function_name));
            openapi.push(gen_method.openapi);
        }

        let codec_ident = format_ident!("{name}Codec");
        let input_path = registry.resolve_rust_path(&package_name, method.input.value_type().proto_path());
        let output_path = registry.resolve_rust_path(&package_name, method.output.value_type().proto_path());

        method_codecs.push(quote! {
            #[derive(Debug, Clone, Default)]
            #[doc(hidden)]
            pub struct #codec_ident<C>(C);

            #[allow(clippy::all, dead_code, unused_imports, unused_variables, unused_parens)]
            const _: () = {
                #[derive(Debug, Clone, Default)]
                pub struct Encoder<E>(E);
                #[derive(Debug, Clone, Default)]
                pub struct Decoder<D>(D);

                impl<C> ::tinc::reexports::tonic::codec::Codec for #codec_ident<C>
                where
                    C: ::tinc::reexports::tonic::codec::Codec<Encode = #output_path, Decode = #input_path>
                {
                    type Encode = C::Encode;
                    type Decode = C::Decode;

                    type Encoder = C::Encoder;
                    type Decoder = Decoder<C::Decoder>;

                    fn encoder(&mut self) -> Self::Encoder {
                        ::tinc::reexports::tonic::codec::Codec::encoder(&mut self.0)
                    }

                    fn decoder(&mut self) -> Self::Decoder {
                        Decoder(
                            ::tinc::reexports::tonic::codec::Codec::decoder(&mut self.0)
                        )
                    }
                }

                impl<D> ::tinc::reexports::tonic::codec::Decoder for Decoder<D>
                where
                    D: ::tinc::reexports::tonic::codec::Decoder<Item = #input_path, Error = ::tinc::reexports::tonic::Status>
                {
                    type Item = D::Item;
                    type Error = ::tinc::reexports::tonic::Status;

                    fn decode(&mut self, buf: &mut ::tinc::reexports::tonic::codec::DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
                        match ::tinc::reexports::tonic::codec::Decoder::decode(&mut self.0, buf) {
                            ::core::result::Result::Ok(::core::option::Option::Some(item)) => {
                                ::tinc::__private::TincValidate::validate_tonic(&item)?;
                                ::core::result::Result::Ok(::core::option::Option::Some(item))
                            },
                            ::core::result::Result::Ok(::core::option::Option::None) => ::core::result::Result::Ok(::core::option::Option::None),
                            ::core::result::Result::Err(err) => ::core::result::Result::Err(err),
                        }
                    }

                    fn buffer_settings(&self) -> ::tinc::reexports::tonic::codec::BufferSettings {
                        ::tinc::reexports::tonic::codec::Decoder::buffer_settings(&self.0)
                    }
                }
            };
        });

        methods.insert(
            name.clone(),
            ProcessedServiceMethod {
                codec_path: ProtoPath::new(format!("{package_name}.{codec_ident}")),
                input: method.input.clone(),
                output: method.output.clone(),
                openapi,
                comments: method.comments.clone(),
            },
        );
    }

    package.push_item(parse_quote! {
        /// This module was automatically generated by `tinc`.
        pub mod #tinc_module_name {
            #![allow(
                unused_variables,
                dead_code,
                missing_docs,
                clippy::wildcard_imports,
                clippy::let_unit_value,
                unused_parens,
            )]

            /// A tinc service struct that exports gRPC routes via an axum router.
            pub struct #tinc_struct_name<T> {
                inner: ::std::sync::Arc<T>,
            }

            impl<T> #tinc_struct_name<T> {
                /// Create a new tinc service struct from a service implementation.
                pub fn new(inner: T) -> Self {
                    Self { inner: ::std::sync::Arc::new(inner) }
                }

                /// Create a new tinc service struct from an existing `Arc`.
                pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
                    Self { inner }
                }
            }

            impl<T> ::std::clone::Clone for #tinc_struct_name<T> {
                fn clone(&self) -> Self {
                    Self { inner: ::std::clone::Clone::clone(&self.inner) }
                }
            }

            impl<T> ::std::fmt::Debug for #tinc_struct_name<T> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, stringify!(#tinc_struct_name))
                }
            }

            impl<T> ::tinc::TincService for #tinc_struct_name<T>
            where
                T: super::#server_module_name::#pascal_name
            {
                fn into_router(self) -> ::tinc::reexports::axum::Router {
                    #(#method_tokens)*

                    ::tinc::reexports::axum::Router::new()
                        #(#route_tokens)*
                        .with_state(self)
                }
            }

            #(#method_codecs)*
        }
    });

    package.services.push(ProcessedService {
        full_name: service.full_name.clone(),
        package: service.package.clone(),
        comments: service.comments.clone(),
        methods,
    });

    Ok(())
}
