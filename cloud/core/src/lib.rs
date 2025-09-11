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
#![deny(clippy::mod_module_files)]
// tonic::Status emits this warning
#![allow(clippy::result_large_err)]

use std::net::SocketAddr;

use diesel_async::AsyncPgConnection;
use scuffle_batching::DataLoader;

use crate::config::{GoogleOAuth2Config, ReverseProxyConfig, TimeoutConfig};

mod captcha;
pub mod cedar;
mod chrono_ext;
mod common;
pub mod config;
pub mod dataloaders;
mod emails;
pub mod geoip;
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
    ) -> impl Future<
        Output = anyhow::Result<diesel_async::pooled_connection::bb8::PooledConnection<'static, AsyncPgConnection>>,
    > + Send;
    fn authorizer(&self) -> &cedar_policy::Authorizer;
    fn http_client(&self) -> &reqwest::Client;
    fn webauthn(&self) -> &webauthn_rs::Webauthn;
    fn redis(&self) -> &fred::clients::Pool;
    fn email_service(
        &self,
    ) -> pb::scufflecloud::email::v1::email_service_client::EmailServiceClient<tonic::transport::Channel>;
    fn geoip_resolver(&self) -> &geoip::GeoIpResolver;
    fn user_loader(&self) -> &DataLoader<dataloaders::UserLoader>;
    fn organization_loader(&self) -> &DataLoader<dataloaders::OrganizationLoader>;
    fn organization_member_by_user_id_loader(&self) -> &DataLoader<dataloaders::OrganizationMemberByUserIdLoader>;
    fn swagger_ui_enabled(&self) -> bool;
    fn dashboard_origin(&self) -> &url::Url;
    fn turnstile_secret_key(&self) -> &str;
    fn timeout_config(&self) -> &TimeoutConfig;
    fn google_oauth2_config(&self) -> &GoogleOAuth2Config;
    fn email_from_address(&self) -> &str;
    fn reverse_proxy_config(&self) -> Option<&ReverseProxyConfig>;
}
