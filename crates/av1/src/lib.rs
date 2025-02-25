//! A crate for decoding and encoding AV1 video headers.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod config;
mod obu;

pub use config::{AV1CodecConfigurationRecord, AV1VideoDescriptor};
pub use obu::{ObuHeader, ObuType, seq};
