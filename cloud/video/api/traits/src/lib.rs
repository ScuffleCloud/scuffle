#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

mod config;
mod database;
mod dataloader;

pub use config::*;
pub use database::*;
pub use dataloader::*;

pub trait Global: ConfigInterface + DatabaseInterface + DataloaderInterface + Send + Sync + 'static {}
