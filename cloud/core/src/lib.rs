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

mod captcha;
pub mod cedar;
mod chrono_ext;
mod common;
mod google_api;
mod http_ext;
mod middleware;
mod operations;
pub mod services;
mod totp;
