//! Core/Authentication server for <https://scuffle.cloud/>.
//!
//! ## Authentication
//!
//! TODO
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
// tonic::Status emits this warning
#![allow(clippy::result_large_err)]

use std::net::SocketAddr;

use diesel_async::AsyncPgConnection;

mod captcha;
mod chrono_datetime_ext;
mod id;
mod middleware;
mod models;
mod request_ext;
mod result_ext;
mod schema;
pub mod services;
mod utils;

pub trait CoreConfig: scuffle_bootstrap::Global + scuffle_signal::SignalConfig + Sync + Send + 'static {
    fn bind(&self) -> SocketAddr;
    fn db(
        &self,
    ) -> impl Future<Output = anyhow::Result<diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncPgConnection>>> + Send;
    fn http_client(&self) -> &reqwest::Client;
    fn turnstile_secret_key(&self) -> &str;
}
