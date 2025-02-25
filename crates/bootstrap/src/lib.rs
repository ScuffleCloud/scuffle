//! A utility crate for creating binaries.
//!
//! Refer to [`Global`], [`Service`], and [`main`] for more information.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use std::sync::Arc;
//!
//! /// Our global state
//! struct Global;
//!
//! // Required by the signal service
//! impl scuffle_signal::SignalConfig for Global {}
//!
//! impl scuffle_bootstrap::global::GlobalWithoutConfig for Global {
//!     async fn init() -> anyhow::Result<Arc<Self>> {
//!         Ok(Arc::new(Self))
//!     }
//! }
//!
//! /// Our own custom service
//! struct MySvc;
//!
//! impl scuffle_bootstrap::service::Service<Global> for MySvc {
//!     async fn run(self, global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
//!         # let _ = global;
//!         println!("running");
//!
//!         // Do some work here
//!
//!         // Wait for the context to be cacelled by the signal service
//!         ctx.done().await;
//!         Ok(())
//!     }
//! }
//!
//! // This generates the main function which runs all the services
//! scuffle_bootstrap::main! {
//!     Global {
//!         scuffle_signal::SignalSvc,
//!         MySvc,
//!     }
//! }
//! ```
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod config;
pub mod global;
pub mod service;

pub use config::ConfigParser;
#[doc(hidden)]
pub use config::EmptyConfig;
pub use global::{Global, GlobalWithoutConfig};
pub use service::Service;

#[doc(hidden)]
pub mod prelude {
    pub use {anyhow, futures, scuffle_bootstrap_derive, scuffle_context, tokio};
}

/// This macro is used to generate the main function for a given global type
/// and service types. It will run all the services in parallel and wait for
/// them to finish before exiting.
///
/// # Example
///
/// ```rust
/// # use std::sync::Arc;
/// # struct MyGlobal;
/// # struct MyService;
/// # impl scuffle_bootstrap::global::GlobalWithoutConfig for MyGlobal {
/// #     async fn init() -> anyhow::Result<Arc<Self>> {
/// #         Ok(Arc::new(Self))
/// #     }
/// # }
/// # impl scuffle_bootstrap::service::Service<MyGlobal> for MyService {
/// #     async fn run(self, global: Arc<MyGlobal>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
/// #         println!("running");
/// #         ctx.done().await;
/// #         Ok(())
/// #     }
/// # }
/// # impl scuffle_signal::SignalConfig for MyGlobal {
/// # }
/// scuffle_bootstrap::main! {
///     MyGlobal {
///         scuffle_signal::SignalSvc,
///         MyService,
///     }
/// }
/// ```
///
/// # See Also
///
/// - [`Service`](crate::Service)
/// - [`Global`](crate::Global)
// We wrap the macro here so that the doc tests can be run & that the docs resolve for `Service` & `Global`
#[macro_export]
macro_rules! main {
    ($($body:tt)*) => {
        $crate::prelude::scuffle_bootstrap_derive::main! { $($body)* }
    };
}
