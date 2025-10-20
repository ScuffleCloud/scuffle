//! Protobuf definitions for the email services used by <https://scuffle.cloud/>.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(unsafe_code)]

/// The raw file descriptor binary for the compiled protobuf definitions.
pub const ANNOTATIONS_PB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tinc.fd.bin"));

/// Include the core v1 protobuf definitions.
pub mod v1 {
    tinc::include_proto!("scufflecloud.email.v1");
}
