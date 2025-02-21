//! A proc-macro to generate the main function for the application.
//!
//! For more information checkout the [scuffle-bootstrap](../README.md) crate.
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable, unit tests are not yet fully implemented.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

use proc_macro::TokenStream;

mod main_impl;

/// Can be used to generate the main function for the application.
#[proc_macro]
pub fn main(input: TokenStream) -> TokenStream {
    handle_error(main_impl::impl_main(input.into()))
}

fn handle_error(input: Result<proc_macro2::TokenStream, syn::Error>) -> TokenStream {
    match input {
        Ok(value) => value.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let input = quote::quote! {
            MyGlobal {
                MyService,
            }
        };

        let output = match main_impl::impl_main(input) {
            Ok(value) => value,
            Err(err) => err.to_compile_error(),
        };

        let syntax_tree = prettyplease::unparse(&syn::parse_file(&output.to_string()).unwrap());

        insta::assert_snapshot!(syntax_tree, @r##"
        #[automatically_derived]
        fn main() -> ::scuffle_bootstrap::prelude::anyhow::Result<()> {
            #[doc(hidden)]
            pub const fn impl_global<G: ::scuffle_bootstrap::global::Global>() {}
            const _: () = impl_global::<MyGlobal>();
            ::scuffle_bootstrap::prelude::anyhow::Context::context(
                <MyGlobal as ::scuffle_bootstrap::global::Global>::pre_init(),
                "pre_init",
            )?;
            let runtime = <MyGlobal as ::scuffle_bootstrap::global::Global>::tokio_runtime();
            let config = ::scuffle_bootstrap::prelude::anyhow::Context::context(
                runtime
                    .block_on(
                        <<MyGlobal as ::scuffle_bootstrap::global::Global>::Config as ::scuffle_bootstrap::config::ConfigParser>::parse(),
                    ),
                "config parse",
            )?;
            let ctx_handle = ::scuffle_bootstrap::prelude::scuffle_context::Handler::global();
            let mut shared_global = ::core::option::Option::None;
            let mut services_vec = ::std::vec::Vec::<
                ::scuffle_bootstrap::service::NamedFuture<
                    ::scuffle_bootstrap::prelude::tokio::task::JoinHandle<anyhow::Result<()>>,
                >,
            >::new();
            let result = runtime
                .block_on(async {
                    let global = <MyGlobal as ::scuffle_bootstrap::global::Global>::init(config)
                        .await?;
                    shared_global = ::core::option::Option::Some(global.clone());
                    {
                        #[doc(hidden)]
                        pub async fn spawn_service(
                            svc: impl ::scuffle_bootstrap::service::Service<MyGlobal>,
                            global: &::std::sync::Arc<MyGlobal>,
                            ctx_handle: &::scuffle_bootstrap::prelude::scuffle_context::Handler,
                            name: &'static str,
                        ) -> anyhow::Result<
                            Option<
                                ::scuffle_bootstrap::service::NamedFuture<
                                    ::scuffle_bootstrap::prelude::tokio::task::JoinHandle<
                                        anyhow::Result<()>,
                                    >,
                                >,
                            >,
                        > {
                            let name = ::scuffle_bootstrap::service::Service::<
                                MyGlobal,
                            >::name(&svc)
                                .unwrap_or_else(|| name);
                            if ::scuffle_bootstrap::prelude::anyhow::Context::context(
                                ::scuffle_bootstrap::service::Service::<
                                    MyGlobal,
                                >::enabled(&svc, &global)
                                    .await,
                                name,
                            )? {
                                Ok(
                                    Some(
                                        ::scuffle_bootstrap::service::NamedFuture::new(
                                            name,
                                            ::scuffle_bootstrap::prelude::tokio::spawn(
                                                ::scuffle_bootstrap::service::Service::<
                                                    MyGlobal,
                                                >::run(svc, global.clone(), ctx_handle.context()),
                                            ),
                                        ),
                                    ),
                                )
                            } else {
                                Ok(None)
                            }
                        }
                        let res = spawn_service(MyService, &global, &ctx_handle, "MyService")
                            .await;
                        if let Some(spawned) = res? {
                            services_vec.push(spawned);
                        }
                    }
                    <MyGlobal as ::scuffle_bootstrap::global::Global>::on_services_start(&global)
                        .await?;
                    macro_rules! handle_service_exit {
                        ($remaining:ident) => {
                            { let ((name, result), _, remaining) =
                            ::scuffle_bootstrap::prelude::futures::future::select_all($remaining)
                            . await; let result =
                            ::scuffle_bootstrap::prelude::anyhow::Context::context(::scuffle_bootstrap::prelude::anyhow::Context::context(result,
                            name) ?, name); < MyGlobal as ::scuffle_bootstrap::global::Global >
                            ::on_service_exit(& global, name, result). await ?; remaining }
                        };
                    }
                    let mut remaining = handle_service_exit!(services_vec);
                    while !remaining.is_empty() {
                        remaining = handle_service_exit!(remaining);
                    }
                    ::scuffle_bootstrap::prelude::anyhow::Ok(())
                });
            let ::core::option::Option::Some(global) = shared_global else {
                return result;
            };
            runtime
                .block_on(
                    <MyGlobal as ::scuffle_bootstrap::global::Global>::on_exit(&global, result),
                )
        }
        "##);
    }
}
