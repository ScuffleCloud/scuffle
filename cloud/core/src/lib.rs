//! Core/Authentication server for <https://scuffle.cloud/>.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`

use std::net::SocketAddr;

use diesel_async::AsyncPgConnection;

pub mod captcha;
pub mod id;
pub mod middleware;
pub mod models;
pub mod schema;
pub mod services;

pub trait CoreConfig: scuffle_bootstrap::Global + scuffle_signal::SignalConfig + Sync + Send + 'static {
    fn bind(&self) -> SocketAddr;
    fn db(
        &self,
    ) -> impl Future<Output = anyhow::Result<diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncPgConnection>>> + Send;
    fn http_client(&self) -> &reqwest::Client;
    fn turnstile_secret_key(&self) -> &str;
}
