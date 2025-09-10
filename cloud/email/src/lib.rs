//! Email service for <https://scuffle.cloud/>.
//!
//! This service is only used internally.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
// tonic::Status emits this warning
#![allow(clippy::result_large_err)]

use std::net::SocketAddr;

pub mod services;

pub trait EmailConfig:
    scuffle_bootstrap::Global
    + scuffle_signal::SignalConfig
    + scuffle_bootstrap_telemetry::TelemetryConfig
    + Sync
    + Send
    + 'static
{
    fn service_name(&self) -> &str;
    fn bind(&self) -> SocketAddr;
}
