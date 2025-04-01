use std::collections::BTreeMap;

use anyhow::Context;
use convert_case::{Boundary, Case, Casing};
use quote::quote;
use syn::{Ident, parse_quote};

use crate::extensions::{Extensions, FieldKind, MessageOpts, MethodIo, MethodOpts, ServiceOpts, WellKnownType};

fn ident_from_str(s: impl AsRef<str>) -> Ident {
    syn::parse_str(s.as_ref()).unwrap_or_else(|_| Ident::new_raw(s.as_ref(), proc_macro2::Span::call_site()))
}

// Define a helper enum for input message options.
enum IoOptions<'a> {
    Message(String, &'a MessageOpts),
    WellKnown(WellKnownType),
}

impl IoOptions<'_> {
    fn path(&self, package: &str) -> syn::Path {
        match self {
            IoOptions::Message(name, _) => {
                let path = object_type_path(name.as_str(), package);
                parse_quote! { super::#path }
            }
            IoOptions::WellKnown(well_known) => {
                let id = ident_from_str(well_known.path());
                parse_quote! { #id }
            }
        }
    }

    fn has_content(&self, excluding: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        let excluding: Vec<_> = excluding.into_iter().collect();

        match self {
            IoOptions::Message(_, message) => message
                .fields
                .keys()
                .any(|name| excluding.iter().all(|ex| ex.as_ref() != name)),
            IoOptions::WellKnown(WellKnownType::Empty) => false,
            IoOptions::WellKnown(_) => true,
        }
    }

    fn has_fields(&self, fields: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        let fields: Vec<_> = fields.into_iter().collect();
        if fields.is_empty() {
            return true;
        }

        match self {
            IoOptions::Message(_, message) => fields.into_iter().all(|field| message.fields.contains_key(field.as_ref())),
            IoOptions::WellKnown(WellKnownType::Struct) => true,
            IoOptions::WellKnown(_) => false,
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
            *part = part.with_boundaries(&Boundary::digit_letter()).to_case(Case::Snake);
        }

        parts[len - 1] = parts[len - 1].to_case(Case::Pascal);
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

// Helper to parse headers.
fn parse_header(
    header: &tinc_pb::http_endpoint_options::Header,
    input_message: &IoOptions,
) -> anyhow::Result<proc_macro2::TokenStream> {
    use tinc_pb::http_endpoint_options::header;

    let header_name = header.name.as_str();
    let field_str = header.field.as_str();

    let field_type = input_message
        .field_type(field_str)
        .with_context(|| format!("header field {} not found in input message", field_str))?;

    let encoding = header.encoding.clone().unwrap_or_else(|| match field_type.strip_option() {
        FieldKind::Map(_, _) | FieldKind::Message(_) | FieldKind::WellKnown(WellKnownType::Struct) => {
            header::Encoding::ContentType(header::ContentType::FormUrlEncoded as i32)
        }
        _ => header::Encoding::ContentType(header::ContentType::Text as i32),
    });

    let header_value = match encoding {
        header::Encoding::Delimiter(delimiter) => {
            quote! {
                ::tinc::helpers::header_decode::text(&parts.headers, #header_name, #field_str, ::core::option::Option::Some(#delimiter))
            }
        }
        header::Encoding::ContentType(content_type) => {
            let content_type = header::ContentType::try_from(content_type)
                .with_context(|| format!("invalid header content type value: {}", content_type))?;

            match content_type {
                header::ContentType::FormUrlEncoded => {
                    quote! {
                        ::tinc::helpers::header_decode::form_url_encoded(&parts.headers, #header_name, #field_str)
                    }
                }
                header::ContentType::Json => {
                    quote! {
                        ::tinc::helpers::header_decode::json(&parts.headers, #header_name, #field_str)
                    }
                }
                header::ContentType::Unspecified | header::ContentType::Text => {
                    quote! {
                        ::tinc::helpers::header_decode::text(&parts.headers, #header_name, #field_str, ::core::option::Option::None)
                    }
                }
            }
        }
        header::Encoding::Param(param) => {
            quote! {
                ::tinc::helpers::header_decode::param(&parts.headers, #header_name, #param, #field_str)
            }
        }
    };

    Ok(quote! {
        input.merge(match #header_value {
            Ok(input) => input,
            Err(err) => return err,
        });
    })
}

struct GeneratedMethod {
    function_body: proc_macro2::TokenStream,
    http_method: Ident,
    path: String,
}

fn generate_content_constants<'a>(
    content_types: impl IntoIterator<Item = (&'a tinc_pb::http_endpoint_options::ContentType, &'a Ident)>,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let const_cts = content_types
        .into_iter()
        .map(|(content_type, ct_ident)| {
            let content_type_strs = content_type
                .accept
                .iter()
                .map(|mime| {
                    Ok(mime
                        .parse::<mediatype::MediaTypeBuf>()
                        .context("invalid mime type")?
                        .to_string())
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            Ok(quote! {
                static #ct_ident: ::std::sync::LazyLock<::tinc::reexports::headers_accept::Accept> =
                    ::std::sync::LazyLock::new(|| {
                        ::std::iter::FromIterator::from_iter([
                            #(
                                ::tinc::reexports::mediatype::MediaTypeBuf::from_string(#content_type_strs.to_owned())
                                    .expect("invalid mime type this is a bug, please report it")
                            ),*
                        ])
                    });
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        #(#const_cts)*
    })
}

fn generate_content_matchers<'a>(
    content_types: impl IntoIterator<Item = (&'a tinc_pb::http_endpoint_options::ContentType, &'a Ident)>,
    input_message: &IoOptions,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let matchers = content_types
        .into_iter()
        .map(|(content_type, ct_ident)| {
            let headers = content_type
                .header
                .iter()
                .map(|header| parse_header(header, input_message))
                .collect::<anyhow::Result<Vec<_>>>()?;

            let merge = content_type
                .content
                .as_ref()
                .and_then(|content| match content {
                    tinc_pb::http_endpoint_options::content_type::Content::Body(field) => Some(quote! {
                        input.merge(
                            ::std::iter::once((::core::convert::Into::into(#field), body))
                        )
                    }),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    quote! {
                        match body {
                            ::tinc::value::Value::Object(object) => {
                                input.merge(object);
                            }
                            _ => return ::tinc::helpers::bad_request_not_object(body),
                        }
                    }
                });

            Ok(quote! {
                if let Some(accept) = #ct_ident.negotiate([content_type]) {
                    #(#headers)*

                    match ::tinc::helpers::parse_body(accept, body).await {
                        Ok(body) => #merge,
                        Err(err) => return err,
                    }
                }
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        #(#matchers else)*
    })
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
            Some(tinc_pb::http_endpoint_options::Method::Custom(method)) => (method.method.as_str(), &method.path),
            _ => return Ok(None),
        };

        let trimmed_path = path.trim_start_matches('/');
        let full_path = if let Some(prefix) = &service.opts.prefix {
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

        anyhow::ensure!(
            input_message.has_fields(&params),
            "input message {} has missing fields: {:?}",
            name,
            params
        );

        let endpoint_headers = endpoint
            .header
            .iter()
            .map(|header| parse_header(header, &input_message))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let path_params = if !params.is_empty() {
            quote! {
                match ::tinc::helpers::parse_path(&mut parts).await {
                    Ok(path_params) => {
                        input.merge(path_params);
                    },
                    Err(err) => return err,
                }
            }
        } else {
            quote! {}
        };

        let is_get_or_delete = matches!(http_method_str, "get" | "delete");
        let use_query_string = endpoint.query_string.unwrap_or(is_get_or_delete);
        let query_string = use_query_string.then(|| {
            quote! {
                match ::tinc::helpers::parse_query(&mut parts).await {
                    Ok(query_string) => {
                        input.merge(query_string);
                    },
                    Err(err) => return err,
                }
            }
        });

        let use_request_body = endpoint.request_body.unwrap_or(!is_get_or_delete) && input_message.has_content(&params);
        let request_body = use_request_body
            .then(|| {
                let mut content_types = endpoint.content_type.clone();
                if content_types.is_empty() {
                    content_types.push(tinc_pb::http_endpoint_options::ContentType {
                        accept: vec!["application/json".to_string()],
                        content: None,
                        header: Vec::new(),
                        multipart: None,
                    });
                }

                let ct_idents: Vec<_> = content_types
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| ident_from_str(format!("ACCEPT_{idx}")))
                    .collect();

                let zipped_iter = content_types.iter().zip(ct_idents.iter());

                let constants = generate_content_constants(zipped_iter.clone())?;
                let matchers = generate_content_matchers(zipped_iter, &input_message)?;

                anyhow::Ok(quote! {
                    let content_type = match ::tinc::helpers::header_decode::content_type(&parts.headers) {
                        Ok(content_type) => content_type,
                        Err(err) => return err,
                    };

                    if let Some(content_type) = &content_type {
                        #constants

                        #matchers {
                            return ::tinc::helpers::no_valid_content_type(content_type, &[#(&*#ct_idents),*]);
                        }
                    }
                })
            })
            .transpose()?;

        let input_path = input_message.path(service.package.as_str());
        let service_method_name = ident_from_str(name.to_case(Case::Snake));

        let function_impl = quote! {
            let mut input = ::tinc::value::Object::new();

            #path_params

            #query_string

            #(#endpoint_headers)*

            #request_body

            let input: #input_path = match ::tinc::helpers::decode_input(input) {
                Ok(input) => input,
                Err(err) => return err,
            };

            let request = ::tinc::reexports::tonic::Request::from_parts(
                ::tinc::reexports::tonic::metadata::MetadataMap::from_headers(parts.headers),
                parts.extensions,
                ::core::convert::Into::into(input),
            );

            let (metadata, body, extensions) = match service.inner.#service_method_name(request).await {
                ::core::result::Result::Ok(response) => response.into_parts(),
                ::core::result::Result::Err(status) => {
                    todo!("todo map errors: {:?}", status);
                }
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

    let snake_name = name.to_case(Case::Snake);
    let pascal_name = name.to_case(Case::Pascal);

    let service_trait = ident_from_str(&pascal_name);
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

            methods.push(method.method_handler(&function_name, &server_module_name, &service_trait, &tinc_struct_name));
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
                    Self { inner: self.inner.clone() }
                }
            }

            impl<T> ::std::fmt::Debug for #tinc_struct_name<T> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, stringify!(#tinc_struct_name))
                }
            }

            impl<T> ::tinc::TincService for #tinc_struct_name<T>
            where
                T: super::#server_module_name::#service_trait
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
