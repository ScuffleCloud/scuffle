use anyhow::Context;
use indexmap::IndexMap;
use openapi::{BodyMethod, GeneratedBody, GeneratedParams, InputGenerator, OutputGenerator};
use openapiv3_1::HttpMethod;
use quote::{format_ident, quote};
use syn::{Ident, parse_quote};
use tinc_pb_prost::http_endpoint_options;

use super::Package;
use super::utils::{field_ident_from_str, type_ident_from_str};
use crate::types::{
    Comments, ProtoPath, ProtoService, ProtoServiceMethod, ProtoServiceMethodEndpoint, ProtoServiceMethodIo,
    ProtoTypeRegistry, ProtoValueType,
};

mod openapi;

struct GeneratedMethod {
    function_body: proc_macro2::TokenStream,
    openapi: openapiv3_1::path::PathItem,
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
        types: &ProtoTypeRegistry,
        components: &mut openapiv3_1::Components,
    ) -> anyhow::Result<GeneratedMethod> {
        let (http_method_oa, path) = match &endpoint.method {
            tinc_pb_prost::http_endpoint_options::Method::Get(path) => (openapiv3_1::HttpMethod::Get, path),
            tinc_pb_prost::http_endpoint_options::Method::Post(path) => (openapiv3_1::HttpMethod::Post, path),
            tinc_pb_prost::http_endpoint_options::Method::Put(path) => (openapiv3_1::HttpMethod::Put, path),
            tinc_pb_prost::http_endpoint_options::Method::Delete(path) => (openapiv3_1::HttpMethod::Delete, path),
            tinc_pb_prost::http_endpoint_options::Method::Patch(path) => (openapiv3_1::HttpMethod::Patch, path),
        };

        let trimmed_path = path.trim_start_matches('/');
        let full_path = if let Some(prefix) = &service.options.prefix {
            format!("/{}/{}", prefix.trim_end_matches('/'), trimmed_path)
        } else {
            format!("/{trimmed_path}")
        };

        let http_method = quote::format_ident!("{http_method_oa}");
        let tracker_ident = quote::format_ident!("tracker");
        let target_ident = quote::format_ident!("target");
        let state_ident = quote::format_ident!("state");
        let mut openapi = openapiv3_1::path::Operation::new();
        let mut generator = InputGenerator::new(
            types,
            components,
            package,
            method.input.value_type().clone(),
            tracker_ident.clone(),
            target_ident.clone(),
            state_ident.clone(),
        );

        let GeneratedParams {
            tokens: path_tokens,
            params,
        } = generator.generate_path_parameter(&full_path)?;
        openapi.parameters(params);

        let is_get_or_delete = matches!(http_method_oa, HttpMethod::Get | HttpMethod::Delete);
        let request = endpoint.request.as_ref().and_then(|req| req.mode.clone()).unwrap_or_else(|| {
            if is_get_or_delete {
                http_endpoint_options::request::Mode::Query(http_endpoint_options::request::QueryParams::default())
            } else {
                http_endpoint_options::request::Mode::Json(http_endpoint_options::request::JsonBody::default())
            }
        });

        let request_tokens = match request {
            http_endpoint_options::request::Mode::Query(http_endpoint_options::request::QueryParams { field }) => {
                let GeneratedParams { tokens, params } = generator.generate_query_parameter(field.as_deref())?;
                openapi.parameters(params);
                tokens
            }
            http_endpoint_options::request::Mode::Binary(http_endpoint_options::request::BinaryBody {
                field,
                content_type_accepts,
                content_type_field,
            }) => {
                let GeneratedBody { tokens, body } = generator.generate_body(
                    &method.cel,
                    BodyMethod::Binary(content_type_accepts.as_deref()),
                    field.as_deref(),
                    content_type_field.as_deref(),
                )?;
                openapi.request_body = Some(body);
                tokens
            }
            http_endpoint_options::request::Mode::Json(http_endpoint_options::request::JsonBody { field }) => {
                let GeneratedBody { tokens, body } =
                    generator.generate_body(&method.cel, BodyMethod::Json, field.as_deref(), None)?;
                openapi.request_body = Some(body);
                tokens
            }
            http_endpoint_options::request::Mode::Text(http_endpoint_options::request::TextBody { field }) => {
                let GeneratedBody { tokens, body } =
                    generator.generate_body(&method.cel, BodyMethod::Text, field.as_deref(), None)?;
                openapi.request_body = Some(body);
                tokens
            }
        };

        let input_path = match &method.input {
            ProtoServiceMethodIo::Single(input) => types.resolve_rust_path(package, input.proto_path()),
            ProtoServiceMethodIo::Stream(_) => anyhow::bail!("currently streaming is not supported by tinc methods."),
        };

        let service_method_name = field_ident_from_str(name);

        let response = endpoint
            .response
            .as_ref()
            .and_then(|resp| resp.mode.clone())
            .unwrap_or_else(
                || http_endpoint_options::response::Mode::Json(http_endpoint_options::response::Json::default()),
            );

        let response_ident = quote::format_ident!("response");
        let builder_ident = quote::format_ident!("builder");
        let mut generator = OutputGenerator::new(
            types,
            components,
            method.output.value_type().clone(),
            response_ident.clone(),
            builder_ident.clone(),
        );

        let GeneratedBody {
            body: response,
            tokens: response_tokens,
        } = match response {
            http_endpoint_options::response::Mode::Binary(http_endpoint_options::response::Binary {
                field,
                content_type_accepts,
                content_type_field,
            }) => generator.generate_body(
                BodyMethod::Binary(content_type_accepts.as_deref()),
                field.as_deref(),
                content_type_field.as_deref(),
            )?,
            http_endpoint_options::response::Mode::Json(http_endpoint_options::response::Json { field }) => {
                generator.generate_body(BodyMethod::Json, field.as_deref(), None)?
            }
            http_endpoint_options::response::Mode::Text(http_endpoint_options::response::Text { field }) => {
                generator.generate_body(BodyMethod::Text, field.as_deref(), None)?
            }
        };

        openapi.response("200", response);

        let validate = if matches!(method.input.value_type(), ProtoValueType::Message(_)) {
            quote! {
                if let Err(err) = ::tinc::__private::TincValidate::validate_http(&#target_ident, #state_ident, &#tracker_ident) {
                    return err;
                }
            }
        } else {
            quote!()
        };

        let function_impl = quote! {
            let mut #state_ident = ::tinc::__private::TrackerSharedState::default();
            let mut #tracker_ident = <<#input_path as ::tinc::__private::TrackerFor>::Tracker as ::core::default::Default>::default();
            let mut #target_ident = <#input_path as ::core::default::Default>::default();

            #path_tokens
            #request_tokens

            #validate

            let request = ::tinc::reexports::tonic::Request::from_parts(
                ::tinc::reexports::tonic::metadata::MetadataMap::from_headers(parts.headers),
                parts.extensions,
                target,
            );

            let (metadata, #response_ident, extensions) = match service.inner.#service_method_name(request).await {
                ::core::result::Result::Ok(response) => response.into_parts(),
                ::core::result::Result::Err(status) => return ::tinc::__private::handle_tonic_status(&status),
            };

            let mut response = {
                let mut #builder_ident = ::tinc::reexports::http::Response::builder();
                match #response_tokens {
                    ::core::result::Result::Ok(v) => v,
                    ::core::result::Result::Err(err) => return ::tinc::__private::handle_response_build_error(err),
                }
            };

            response.headers_mut().extend(metadata.into_headers());
            *response.extensions_mut() = extensions;

            response
        };

        Ok(GeneratedMethod {
            function_body: function_impl,
            http_method,
            openapi: openapiv3_1::PathItem::new(http_method_oa, openapi),
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
    pub openapi: openapiv3_1::OpenApi,
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
    pub codec_path: Option<ProtoPath>,
    pub input: ProtoServiceMethodIo,
    pub output: ProtoServiceMethodIo,
    pub comments: Comments,
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

    let mut components = openapiv3_1::Components::new();
    let mut paths = openapiv3_1::Paths::builder();

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
        for (idx, endpoint) in method.endpoints.iter().enumerate() {
            let gen_method =
                GeneratedMethod::new(name, &package_name, service, method, endpoint, registry, &mut components)?;
            let function_name = quote::format_ident!("{name}_{idx}");

            method_tokens.push(gen_method.method_handler(
                &function_name,
                &server_module_name,
                &pascal_name,
                &tinc_struct_name,
            ));
            route_tokens.push(gen_method.route(&function_name));
            paths = paths.path(gen_method.path, gen_method.openapi);
        }

        let codec_path = if matches!(method.input.value_type(), ProtoValueType::Message(_)) {
            let input_path = registry.resolve_rust_path(&package_name, method.input.value_type().proto_path());
            let output_path = registry.resolve_rust_path(&package_name, method.output.value_type().proto_path());
            let codec_ident = format_ident!("{name}Codec");
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
            Some(ProtoPath::new(format!("{package_name}.{codec_ident}")))
        } else {
            None
        };

        methods.insert(
            name.clone(),
            ProcessedServiceMethod {
                codec_path,
                input: method.input.clone(),
                output: method.output.clone(),
                comments: method.comments.clone(),
            },
        );
    }

    let openapi = openapiv3_1::OpenApi::builder().components(components).paths(paths).build();

    let json_openapi = openapi.to_json().context("invalid openapi schema generation")?;

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
                irrefutable_let_patterns,
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

                fn openapi_schema_str(&self) -> &'static str {
                    #json_openapi
                }
            }

            #(#method_codecs)*
        }
    });

    package.services.push(ProcessedService {
        full_name: service.full_name.clone(),
        package: service.package.clone(),
        comments: service.comments.clone(),
        openapi,
        methods,
    });

    Ok(())
}
