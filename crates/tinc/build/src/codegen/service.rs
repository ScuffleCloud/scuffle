use std::collections::BTreeMap;

use anyhow::Context;
use quote::quote;
use syn::{Ident, parse_quote};
use tinc_pb::http_endpoint_options;

use crate::codegen::{field_ident_from_str, ident_from_str, type_ident_from_str};
use crate::extensions::{
    Extensions, FieldKind, FieldModifier, FieldType, MessageOpts, MethodIo, MethodOpts, PrimitiveKind, ServiceOpts,
    WellKnownType,
};

// Define a helper enum for input message options.
enum IoOptions<'a> {
    Message(String, &'a MessageOpts),
    WellKnown(WellKnownType),
}

struct PathFields {
    defs: Vec<proc_macro2::TokenStream>,
    mappings: Vec<proc_macro2::TokenStream>,
}

fn field_extractor_generator(
    field_str: &str,
    extensions: &Extensions,
    message: &MessageOpts,
) -> anyhow::Result<(proc_macro2::TokenStream, FieldKind)> {
    let mut next_message = Some(message);
    let mut is_optional = false;
    let mut kind = None;
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

        kind = Some(field.kind.clone());
        mapping = quote! {{
            let (tracker, target) = #mapping;
            #optional_unwrap
            let tracker = tracker.#field_ident.get_or_insert_default();
            let target = &mut target.#field_ident;
            (tracker, target)
        }};

        is_optional = matches!(field.kind.modifier(), Some(FieldModifier::Optional));
        if matches!(field.kind.modifier(), Some(FieldModifier::Optional) | None) {
            next_message = field.kind.message_name().and_then(|key| extensions.messages().get(key))
        } else {
            next_message = None;
        }
    }

    Ok((mapping, kind.unwrap()))
}

impl IoOptions<'_> {
    fn path(&self, package: &str) -> syn::Path {
        match self {
            IoOptions::Message(name, _) => {
                let path = object_type_path(name.as_str(), package);
                parse_quote! { super::#path }
            }
            IoOptions::WellKnown(well_known) => {
                todo!("handle well-known types");
            }
        }
    }

    fn path_struct(
        &self,
        extensions: &Extensions,
        fields: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> anyhow::Result<Option<PathFields>> {
        let fields: Vec<_> = fields.into_iter().collect();
        if fields.is_empty() {
            return Ok(None);
        }

        let mut defs = Vec::new();
        let mut mappings = Vec::new();

        match self {
            IoOptions::Message(_, message) => {
                for (idx, field) in fields.iter().enumerate() {
                    let field_str = field.as_ref();
                    let path_field_ident = ident_from_str(format!("field_{idx}"));

                    let (path_mapping, field_kind) = field_extractor_generator(field_str, extensions, message)?;

                    let setter = match field_kind.modifier() {
                        Some(FieldModifier::Optional) => quote! {
                            tracker.get_or_insert_default();
                            target.insert(path.#path_field_ident.into());
                        },
                        _ => quote! {
                            *target = path.#path_field_ident.into();
                        },
                    };

                    mappings.push(quote! {{
                        let (tracker, target) = #path_mapping;
                        #setter;
                    }});

                    let ty = match field_kind.field_type() {
                        FieldType::Enum(path) => {
                            let path = object_type_path(path.as_str(), message.package.as_str());
                            quote! {
                                super::#path
                            }
                        }
                        FieldType::Message(m) => anyhow::bail!("message type cannot be mapped: {m}"),
                        FieldType::Primitive(prim) | FieldType::WellKnown(WellKnownType::Primitive(prim)) => match prim {
                            PrimitiveKind::Bool => quote! {
                                ::core::primitive::bool
                            },
                            PrimitiveKind::F32 => quote! {
                                ::core::primitive::f32
                            },
                            PrimitiveKind::F64 => quote! {
                                ::core::primitive::f64
                            },
                            PrimitiveKind::I32 => quote! {
                                ::core::primitive::i32
                            },
                            PrimitiveKind::I64 => quote! {
                                ::core::primitive::i64
                            },
                            PrimitiveKind::U32 => quote! {
                                ::core::primitive::u32
                            },
                            PrimitiveKind::U64 => quote! {
                                ::core::primitive::u64
                            },
                            PrimitiveKind::String => quote! {
                                ::std::string::String
                            },
                            PrimitiveKind::Bytes => anyhow::bail!("bytes type cannot be mapped"),
                        },
                        FieldType::WellKnown(wk) => match wk {
                            WellKnownType::Duration => quote! {
                                ::tinc::__private::well_known::Duration
                            },
                            WellKnownType::Timestamp => quote! {
                                ::tinc::__private::well_known::Timestamp
                            },
                            WellKnownType::Value => quote! {
                                ::tinc::__private::well_known::Value
                            },
                            t => {
                                anyhow::bail!("well-known type cannot be mapped: {t:?}");
                            }
                        },
                        FieldType::OneOf(_) => {
                            anyhow::bail!("oneof type cannot be mapped");
                        }
                    };

                    defs.push(quote! {
                        #[serde(rename = #field_str)]
                        #path_field_ident: #ty
                    });
                }
            }
            IoOptions::WellKnown(wk) => {
                if fields.len() != 1 {
                    anyhow::bail!("well-known type can only have one field");
                }

                let field = &fields[0];
                if field.as_ref() != "value" {
                    anyhow::bail!("well-known type can only have field 'value'");
                }

                mappings.push(quote! {
                    *target = path.value.into();
                });

                match wk {
                    WellKnownType::Duration => {
                        defs.push(quote! {
                            #[serde(rename = "value")]
                            value: ::tinc::__private::well_known::Duration
                        });
                    }
                    WellKnownType::Timestamp => {
                        defs.push(quote! {
                            #[serde(rename = "value")]
                            value: ::tinc::__private::well_known::Timestamp
                        });
                    }
                    WellKnownType::Value => {
                        defs.push(quote! {
                            #[serde(rename = "value")]
                            value: ::tinc::__private::well_known::Value
                        });
                    }
                    t => anyhow::bail!("well-known type cannot be mapped: {t:?}"),
                }
            }
        }

        Ok(Some(PathFields { defs, mappings }))
    }

    fn field_extract(
        &self,
        field_str: &str,
        extensions: &Extensions,
    ) -> anyhow::Result<(proc_macro2::TokenStream, FieldKind)> {
        match self {
            IoOptions::Message(_, message) => {
                let (path_mapping, field_kind) = field_extractor_generator(field_str, extensions, message)?;
                Ok((path_mapping, field_kind))
            }
            IoOptions::WellKnown(_) => {
                anyhow::bail!("well-known type cannot be mapped");
            }
        }
    }

    fn field_type(&self, field: &str) -> Option<&FieldKind> {
        match self {
            IoOptions::Message(_, message) => message.fields.get(field).map(|field| &field.kind),
            IoOptions::WellKnown(WellKnownType::Struct) => Some(&FieldKind::WellKnown(WellKnownType::Value)),
            IoOptions::WellKnown(_) => None,
        }
    }
}

fn object_type_path(key: &str, package: &str) -> syn::Path {
    let mut parts: Vec<String> = key
        .strip_prefix(package)
        .unwrap_or(key)
        .split('.')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    let len = parts.len();
    if len > 1 {
        for part in &mut parts[..len - 1] {
            *part = super::field_ident_from_str(&part).to_string();
        }

        parts[len - 1] = super::type_ident_from_str(&parts[len - 1]).to_string();
    }

    syn::parse_str::<syn::Path>(&parts.join("::")).unwrap()
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

struct GeneratedMethod {
    function_body: proc_macro2::TokenStream,
    http_method: Ident,
    path: String,
}

impl GeneratedMethod {
    fn new(
        name: &str,
        method: &MethodOpts,
        service: &ServiceOpts,
        extensions: &Extensions,
        endpoint: &tinc_pb::HttpEndpointOptions,
    ) -> anyhow::Result<Option<GeneratedMethod>> {
        let (http_method_str, path) = match endpoint.method.as_ref() {
            Some(tinc_pb::http_endpoint_options::Method::Get(path)) => ("get", path),
            Some(tinc_pb::http_endpoint_options::Method::Post(path)) => ("post", path),
            Some(tinc_pb::http_endpoint_options::Method::Put(path)) => ("put", path),
            Some(tinc_pb::http_endpoint_options::Method::Delete(path)) => ("delete", path),
            Some(tinc_pb::http_endpoint_options::Method::Patch(path)) => ("patch", path),
            Some(tinc_pb::http_endpoint_options::Method::Custom(method)) => {
                todo!("custom method not implemented: {:?}", method);
                // (method.method.as_str(), &method.path)
            }
            _ => return Ok(None),
        };

        let trimmed_path = path.trim_start_matches('/');
        let full_path = if let Some(prefix) = &service.prefix {
            format!("/{}/{}", prefix.trim_end_matches('/'), trimmed_path)
        } else {
            format!("/{}", trimmed_path)
        };

        let http_method = ident_from_str(http_method_str);
        let params = parse_route(&full_path);

        // Determine the input message type.
        let input_message = match &method.input {
            MethodIo::Message(name) => {
                let input_message = extensions.messages().get(name).expect("input message not found");
                IoOptions::Message(name.clone(), input_message)
            }
            MethodIo::WellKnown(well_known) => IoOptions::WellKnown(*well_known),
        };

        let path_struct = input_message
            .path_struct(extensions, &params)
            .with_context(|| format!("failed to generate path struct for method: {name}"))?;

        let path_params = path_struct.map(|PathFields { defs, mappings }| {
            quote! {{
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
            }}
        });

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
                let extract = if field.is_empty() {
                    quote! {}
                } else {
                    let (extract, _) = input_message.field_extract(&field, extensions)?;
                    extract
                };
                quote! {{
                    let mut tracker = &mut tracker;
                    let mut target = &mut target;

                    #extract
                    if let Err(err) = ::tinc::__private::deserialize_query_string(
                        &parts,
                        tracker,
                        target
                    ) {
                        return err;
                    }
                }}
            }
            http_endpoint_options::Input::Body(http_endpoint_options::RequestBody {
                field,
                content_type_field,
            }) => {
                let content_type = if !content_type_field.is_empty() {
                    if content_type_field == field {
                        anyhow::bail!("content type field cannot be the same as the body field");
                    }

                    let (extract, kind) = input_message.field_extract(&content_type_field, extensions)?;
                    if !matches!(kind.field_type(), FieldType::Primitive(PrimitiveKind::String)) {
                        anyhow::bail!("content type field must be a string");
                    }

                    let modifier = match kind.modifier() {
                        Some(FieldModifier::Optional) => quote! {
                            let (mut tracker, mut target) = #extract;
                            tracker.get_or_insert_default();
                            target.insert(content_type.into());
                        },
                        None => quote! {
                            let (_, mut target) = #extract;
                            *target = content_type.into();
                        },
                        _ => anyhow::bail!("content type field cannot be repeated or map"),
                    };

                    quote! {{
                        if let Some(content_type) = parts.headers.get(::tinc::reexports::http::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()) {
                            #extract
                            #modifier
                        }
                    }}
                } else {
                    quote! {}
                };

                let (extract, is_raw_bytes) = if field.is_empty() {
                    (
                        quote! {},
                        matches!(
                            input_message,
                            IoOptions::WellKnown(WellKnownType::Primitive(PrimitiveKind::Bytes))
                        ),
                    )
                } else {
                    let (extract, kind) = input_message.field_extract(&field, extensions)?;
                    (
                        extract,
                        matches!(
                            (kind.field_type(), kind.modifier()),
                            (
                                FieldType::Primitive(PrimitiveKind::Bytes),
                                Some(FieldModifier::Optional) | None
                            )
                        ),
                    )
                };

                let de_func = if is_raw_bytes {
                    quote! {
                        deserialize_body_bytes
                    }
                } else {
                    quote! {
                        deserialize_body_json
                    }
                };

                let body = quote! {{
                    #extract
                    if let Err(err) = ::tinc::__private::#de_func(&parts, body, tracker, target, &mut state).await {
                        return err;
                    }
                }};

                quote! {{
                    let mut tracker = &mut tracker;
                    let mut target = &mut target;

                    #body
                    #content_type
                }}
            }
        };

        let input_path = input_message.path(service.package.as_str());
        let service_method_name = field_ident_from_str(name);

        let function_impl = quote! {
            let mut state = ::tinc::__private::TrackerSharedState::default();
            let mut tracker = <<#input_path as ::tinc::__private::TrackerFor>::Tracker as ::core::default::Default>::default();
            let mut target = <#input_path as ::core::default::Default>::default();

            #path_params

            #input

            let request = ::tinc::reexports::tonic::Request::from_parts(
                ::tinc::reexports::tonic::metadata::MetadataMap::from_headers(parts.headers),
                parts.extensions,
                target,
            );

            let (metadata, body, extensions) = match service.inner.#service_method_name(request).await {
                ::core::result::Result::Ok(response) => response.into_parts(),
                ::core::result::Result::Err(status) => return ::tinc::__private::error::handle_status(&status),
            };

            let mut response = ::tinc::reexports::axum::response::IntoResponse::into_response(
                ::tinc::reexports::axum::extract::Json(body),
            );

            *response.headers_mut() = metadata.into_headers();
            *response.extensions_mut() = extensions;

            response
        };

        Ok(Some(GeneratedMethod {
            function_body: function_impl,
            http_method,
            path: full_path,
        }))
    }

    pub fn method_handler(
        &self,
        function_name: &Ident,
        server_module_name: &Ident,
        service_trait: &Ident,
        tinc_struct_name: &Ident,
    ) -> proc_macro2::TokenStream {
        let function_impl = &self.function_body;

        quote! {
            #[allow(non_snake_case, unused_mut, dead_code, unused_variables)]
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

    pub fn route(&self, function_name: &Ident) -> proc_macro2::TokenStream {
        let path = &self.path;
        let http_method = &self.http_method;

        quote! {
            .route(#path, ::tinc::reexports::axum::routing::#http_method(#function_name::<T>))
        }
    }
}

pub(super) fn handle_service(
    service_key: &str,
    service: &ServiceOpts,
    extensions: &Extensions,
    _: &mut tonic_build::Config,
    modules: &mut BTreeMap<String, Vec<syn::Item>>,
) -> anyhow::Result<()> {
    const EXPECT_FMT: &str = "service should be in the format <package>.<service>";

    let name = service_key
        .strip_prefix(service.package.as_str())
        .and_then(|s| s.strip_prefix('.'))
        .expect(EXPECT_FMT);

    let snake_name = field_ident_from_str(name);
    let pascal_name = type_ident_from_str(name);

    let tinc_module_name = ident_from_str(format!("{}_tinc", snake_name));
    let server_module_name = ident_from_str(format!("{}_server", snake_name));
    let tinc_struct_name = ident_from_str(format!("{}Tinc", pascal_name));

    let mut methods = Vec::new();
    let mut routes = Vec::new();

    for (name, method) in service.methods.iter() {
        for (idx, endpoint) in method.opts.iter().enumerate() {
            let Some(method) = GeneratedMethod::new(name, method, service, extensions, endpoint)? else {
                continue;
            };

            let function_name = ident_from_str(format!("{name}_{idx}"));

            methods.push(method.method_handler(&function_name, &server_module_name, &pascal_name, &tinc_struct_name));
            routes.push(method.route(&function_name));
        }
    }

    modules.entry(service.package.clone()).or_default().push(parse_quote! {
        /// This module was automatically generated by `tinc`.
        pub mod #tinc_module_name {
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
                    #(#methods)*

                    ::tinc::reexports::axum::Router::new()
                        #(#routes)*
                        .with_state(self)
                }
            }
        }
    });

    Ok(())
}
