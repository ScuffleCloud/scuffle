//! Protobuf Compiled Definitions for Tinc
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(clippy::all)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

include!(concat!(env!("OUT_DIR"), "/tinc.rs"));

/// The raw protobuf file
pub const TINC_ANNOTATIONS: &str = include_str!("../annotations.proto");
/// Field descriptor binary
pub const TINC_ANNOTATIONS_PB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tinc.annotations.pb"));
