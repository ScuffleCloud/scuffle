//! A proc-macro to derive the `#[metrics]` attribute and the
//! `#[derive(MetricEnum)]` attribute.
//!
//! For more information checkout the [`scuffle-metrics`](../scuffle_metrics)
//! crate.
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable, unit tests
//! are not yet fully implemented.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![deny(missing_docs)]
#![deny(unsafe_code)]

use enum_impl::metric_enum_impl;
use metrics_impl::metrics_impl;
use proc_macro::TokenStream;

mod enum_impl;
mod metrics_impl;

/// A macro used to create metric handlers.
///
/// You can change the crate by specifying `#[metrics(crate_path = "...")]`.
///
/// Module Attributes:
///
/// - `crate_path`: The `scuffle_metrics` crate path.
/// - `rename`: The name of the metric container.
///
/// Function Attributes:
///
/// - `crate_path`: The `scuffle_metrics` crate path.
/// - `builder`: The builder to use for the metric.
/// - `unit`: The unit of the metric.
/// - `rename`: The name of the metric.
///
/// Function Arguments Attributes:
///
/// - `rename`: The name of the argument.
///
/// When using the module, you do not need to attribute each function with the
/// `#[metrics]` attribute. All non function definitions are ignored.
///
/// # Module Example
///
/// ```rust
/// #[scuffle_metrics::metrics]
/// mod example {
///     use scuffle_metrics::{MetricEnum, collector::CounterU64};
///
///     #[derive(MetricEnum)]
///     pub enum Kind {
///         Http,
///         Grpc,
///     }
///
///     #[metrics(unit = "requests")]
///     pub fn request(kind: Kind) -> CounterU64;
/// }
///
/// // Increment the counter
/// example::request(example::Kind::Http).incr();
/// ```
///
/// # Function Example
///
/// ```rust
/// # use scuffle_metrics::{MetricEnum, collector::CounterU64};
/// # #[derive(MetricEnum)]
/// # pub enum Kind {
/// #     Http,
/// #     Grpc,
/// # }
/// #[scuffle_metrics::metrics(unit = "requests")]
/// pub fn request(kind: Kind) -> CounterU64;
///
/// // Increment the counter
/// request(Kind::Http).incr();
/// ```
#[proc_macro_attribute]
pub fn metrics(args: TokenStream, input: TokenStream) -> TokenStream {
    match metrics_impl(args, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implements a conversion `Into<opentelemetry::Value>` for the enum.
/// This allows the enum to be used as a metric label.
///
/// You can change the crate by specifying `#[metrics(crate_path = "...")]`.
///
/// Enum Attributes:
///
/// - `crate_path`: The `scuffle_metrics` crate path.
///
/// Enum Variant Attributes:
///
/// - `rename`: The name of the metric.
#[proc_macro_derive(MetricEnum, attributes(metrics))]
pub fn metric_enum(input: TokenStream) -> TokenStream {
    match metric_enum_impl(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
