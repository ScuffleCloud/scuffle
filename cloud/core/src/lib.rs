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
mod chrono_ext;
mod google_api;
mod http_ext;
mod id;
mod middleware;
mod models;
mod schema;
pub mod services;
mod std_ext;
mod totp;
mod utils;
mod webauthn;

pub trait CoreConfig: scuffle_bootstrap::Global + scuffle_signal::SignalConfig + Sync + Send + 'static {
    fn bind(&self) -> SocketAddr;
    fn db(
        &self,
    ) -> impl Future<Output = anyhow::Result<diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncPgConnection>>> + Send;
    fn http_client(&self) -> &reqwest::Client;
    fn dashboard_url(&self) -> &str;
    fn turnstile_secret_key(&self) -> &str {
        "1x0000000000000000000000000000000AA"
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
}
