//! A pure Rust implementation of the H.264 encoder and decoder.
//!
//! This crate is designed to provide a simple and safe interface to encode and decode H.264 headers.
//!
//! ## Why do we need this?
//!
//! This crate aims to provides a simple and safe interface for h264.
//!
//! ## How is this different from other h264 crates?
//!
//! The other main h264 crate is TODO.
//!
//! ## Notable features
//!
//! This crate is a completely safe implementation of H264 encoding and decoding, which means there is no unsafe code!
//!
//! ## Examples
//!
//! TODO
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::missing_const_for_fn)]

/// Config functionality.
pub mod config;
/// Sequence Parameter Set (SPS) functionality.
pub mod sps;

pub use self::config::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
pub use self::sps::{ColorConfig, Sps, SpsExtended};
