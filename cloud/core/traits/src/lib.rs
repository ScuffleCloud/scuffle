#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

pub use crate::config::*;
pub use crate::database::*;
pub use crate::dataloader::*;
pub use crate::email::*;
pub use crate::http::*;
pub use crate::mtls::*;
pub use crate::redis::*;
pub use crate::webauthn::*;

mod config;
mod database;
mod dataloader;
mod email;
mod http;
mod mtls;
mod redis;
mod webauthn;

pub trait Global:
    ConfigInterface
    + DatabaseInterface
    + DataloaderInterface
    + HttpClientInterface
    + geo_ip::GeoIpInterface
    + EmailInterface
    + RedisInterface
    + WebAuthnInterface
    + MtlsInterface
    + Send
    + Sync
    + 'static
{
}
