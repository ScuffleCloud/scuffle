#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

mod config;
mod rtmps;

pub use config::*;
pub use rtmps::*;

pub trait Global: ConfigInterface + RtmpsInterface + Send + Sync + 'static {}
