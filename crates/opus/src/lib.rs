//! A pure Rust implementation of Opus ISOBMFF boxes.
//!
//! This crates implements the Opus ISO Base Media File Format (ISOBMFF) boxes defined by [Encapsulation of Opus in ISO Base Media File Format Version 0.6.8](https://www.opus-codec.org/docs/opus_in_isobmff.html).
//!
//! It serves only that purpose and does not implement any other Opus-related functionality.
//!
//! Future work may change this.
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
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

pub mod boxes;
