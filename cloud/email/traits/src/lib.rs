#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

pub use aws_ses::*;
pub use config::*;

mod aws_ses;
mod config;

pub trait Global: ConfigInterface + AwsSesInterface + Send + Sync + 'static {}
