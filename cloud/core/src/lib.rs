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
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
// tonic::Status emits this warning
#![allow(clippy::result_large_err)]

use std::net::SocketAddr;

use diesel_async::AsyncPgConnection;

mod captcha;
pub mod cedar;
mod chrono_ext;
mod common;
mod emails;
mod google_api;
mod http_ext;
pub mod id;
mod middleware;
mod models;
mod operations;
mod schema;
pub mod services;
mod std_ext;
mod totp;

pub trait CoreConfig:
    scuffle_bootstrap::Global
    + scuffle_signal::SignalConfig
    + scuffle_bootstrap_telemetry::TelemetryConfig
    + Sync
    + Send
    + 'static
{
    fn service_name(&self) -> &str;
    fn bind(&self) -> SocketAddr;
    fn db(
        &self,
    ) -> impl Future<Output = anyhow::Result<diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncPgConnection>>> + Send;
    fn authorizer(&self) -> &cedar_policy::Authorizer;
    fn http_client(&self) -> &reqwest::Client;
    fn webauthn(&self) -> &webauthn_rs::Webauthn;
    fn redis(&self) -> &fred::clients::Pool;
    fn swagger_ui_enabled(&self) -> bool {
        false
    }
    fn dashboard_origin(&self) -> &url::Url;
    fn turnstile_secret_key(&self) -> &str {
        "1x0000000000000000000000000000000AA"
    }
    fn max_request_lifetime(&self) -> chrono::Duration {
        chrono::Duration::minutes(2)
    }
    fn user_session_timeout(&self) -> chrono::Duration {
        chrono::Duration::days(30)
    }
    fn mfa_timeout(&self) -> chrono::Duration {
        chrono::Duration::minutes(5)
    }
    fn user_session_token_timeout(&self) -> chrono::Duration {
        chrono::Duration::hours(4)
    }
    fn email_registration_request_timeout(&self) -> chrono::Duration {
        chrono::Duration::hours(1)
    }
    fn user_session_request_timeout(&self) -> chrono::Duration {
        chrono::Duration::minutes(5)
    }
    fn magic_link_user_session_request_timeout(&self) -> chrono::Duration {
        chrono::Duration::minutes(15)
    }
    fn google_client_id(&self) -> &str;
    fn google_client_secret(&self) -> &str;
    fn email_from_address(&self) -> &str;
}
