#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

pub use aws::*;
pub use config::*;
pub use http::*;

mod aws;
mod config;
mod http;

pub trait Global: ConfigInterface + HttpClientInterface + AwsInterface + Send + Sync + 'static {}
