//! Protobuf definitions for various services used by <https://scuffle.cloud/>.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]

pub const ANNOTATIONS_PB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tinc.fd.bin"));

tinc::include_proto!();
