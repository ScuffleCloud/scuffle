use anyhow::Context;
use quote::quote;
use syn::{Ident, parse_quote};
use tinc_pb::http_endpoint_options;

use super::Package;
use super::utils::{field_ident_from_str, type_ident_from_str};
use crate::types::{
    ProtoMessageType, ProtoModifiedValueType, ProtoService, ProtoServiceMethod, ProtoServiceMethodEndpoint,
    ProtoServiceMethodIo, ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoWellKnownType,
};

struct PathFields {
    defs: Vec<proc_macro2::TokenStream>,
    mappings: Vec<proc_macro2::TokenStream>,
}

fn field_extractor_generator(
    field_str: &str,
    registry: &ProtoTypeRegistry,
    message: &ProtoMessageType,
) -> anyhow::Result<(proc_macro2::TokenStream, ProtoType)> {
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

        kind = Some(field.ty.clone());
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

    Ok((mapping, kind.unwrap()))
}

fn path_struct(
    ty: &ProtoValueType,
    package: &str,
    fields: &[String],
    registry: &ProtoTypeRegistry,
) -> anyhow::Result<PathFields> {
    let mut defs = Vec::new();
    let mut mappings = Vec::new();

    let match_single_ty = |ty: &ProtoValueType| {
        Some(match &ty {
            ProtoValueType::Enum(path) => {
                let path = registry.get_enum(path).expect("enum not found").rust_path(package);
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
                let (path_mapping, ty) = field_extractor_generator(field_str, registry, message)?;

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
                    let (tracker, target) = #path_mapping;
                    #setter;
                }});

                let ty = match ty {
                    ProtoType::Modified(ProtoModifiedValueType::Optional(value)) | ProtoType::Value(value) => {
                        match_single_ty(&value)
                    }
                    _ => None,
                };

                let Some(ty) = ty else {
                    anyhow::bail!("type cannot be mapped: {ty:?}");
                };

                defs.push(quote! {
                    #[serde(rename = #field_str)]
                    #path_field_ident: #ty
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

    Ok(PathFields { defs, mappings })
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
        package: &str,
        service: &ProtoService,
        method: &ProtoServiceMethod,
        endpoint: &ProtoServiceMethodEndpoint,
        registry: &ProtoTypeRegistry,
    ) -> anyhow::Result<GeneratedMethod> {
        let (http_method_str, path) = match &endpoint.method {
            tinc_pb::http_endpoint_options::Method::Get(path) => ("get", path),
            tinc_pb::http_endpoint_options::Method::Post(path) => ("post", path),
            tinc_pb::http_endpoint_options::Method::Put(path) => ("put", path),
            tinc_pb::http_endpoint_options::Method::Delete(path) => ("delete", path),
            tinc_pb::http_endpoint_options::Method::Patch(path) => ("patch", path),
            tinc_pb::http_endpoint_options::Method::Custom(method) => {
                todo!("custom method not implemented: {:?}", method);
                // (method.method.as_str(), &method.path)
            }
        };

        let trimmed_path = path.trim_start_matches('/');
        let full_path = if let Some(prefix) = &service.options.prefix {
            format!("/{}/{}", prefix.trim_end_matches('/'), trimmed_path)
        } else {
            format!("/{trimmed_path}")
        };

        let http_method = quote::format_ident!("{http_method_str}");
        let params = parse_route(&full_path);

        let path_params = if !params.is_empty() {
            let PathFields { defs, mappings } = path_struct(method.input.value_type(), package, &params, registry)
                .with_context(|| format!("failed to generate path struct for method: {name}"))?;

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
                    ProtoServiceMethodIo::Single(ProtoValueType::Message(_)) if field.is_empty() => quote! {},
                    ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) => {
                        let message = registry.get_message(message).expect("message not found");
                        let (extract, _) = field_extractor_generator(&field, registry, message)?;
                        extract
                    }
                    _ => todo!("handle other types"),
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

                    let (extract, kind) = match &method.input {
                        ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) => {
                            let message = registry.get_message(message).expect("message not found");
                            field_extractor_generator(&field, registry, message)?
                        }
                        _ => todo!("handle other types"),
                    };

                    let modifier = match &kind {
                        ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::String)) => quote! {
                            let (mut tracker, mut target) = #extract;
                            tracker.get_or_insert_default();
                            target.insert(content_type.into());
                        },
                        ProtoType::Value(ProtoValueType::String) => quote! {
                            let (_, mut target) = #extract;
                            *target = content_type.into();
                        },
                        _ => anyhow::bail!("content type field must be a string: {kind:?}"),
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
                        matches!(&method.input, ProtoServiceMethodIo::Single(ProtoValueType::Bytes)),
                    )
                } else {
                    let (extract, ty) = match &method.input {
                        ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) => {
                            let message = registry.get_message(message).expect("message not found");
                            field_extractor_generator(&field, registry, message)?
                        }
                        _ => todo!("handle other types"),
                    };
                    (
                        extract,
                        matches!(
                            ty,
                            ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Bytes))
                                | ProtoType::Value(ProtoValueType::Bytes)
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

        let input_path = match &method.input {
            ProtoServiceMethodIo::Single(ProtoValueType::Message(message)) => {
                registry.get_message(message).expect("message not found").rust_path(package)
            }
            _ => todo!("handle other types"),
        };

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
                ::core::result::Result::Err(status) => return ::tinc::__private::handle_tonic_status(&status),
            };

            let mut response = ::tinc::reexports::axum::response::IntoResponse::into_response(
                ::tinc::reexports::axum::extract::Json(body),
            );

            *response.headers_mut() = metadata.into_headers();
            *response.extensions_mut() = extensions;

            response
        };

        Ok(GeneratedMethod {
            function_body: function_impl,
            http_method,
            path: full_path,
        })
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
    service: &ProtoService,
    package: &mut Package,
    registry: &ProtoTypeRegistry,
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

    let mut methods = Vec::new();
    let mut routes = Vec::new();

    let package_name = format!("{}.{tinc_module_name}", service.package);

    for (name, method) in service.methods.iter() {
        for (idx, endpoint) in method.endpoints.iter().enumerate() {
            let method = GeneratedMethod::new(name, &package_name, service, method, endpoint, registry)?;

            let function_name = quote::format_ident!("{name}_{idx}");

            methods.push(method.method_handler(&function_name, &server_module_name, &pascal_name, &tinc_struct_name));
            routes.push(method.route(&function_name));
        }
    }

    package.push_item(parse_quote! {
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
