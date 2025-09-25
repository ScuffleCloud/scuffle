#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

pub use aws::*;
pub use config::*;
pub use http::*;
pub use mtls::*;

mod aws;
mod config;
mod http;
mod mtls;

pub trait Global: ConfigInterface + HttpClientInterface + AwsInterface + MtlsInterface + Send + Sync + 'static {}
