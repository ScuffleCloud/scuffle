//! A pure Rust implementation of the H.264 (header only) builder and parser.
//!
//! This crate is designed to provide a simple and safe interface to build and parse H.264 headers.
//!
//! ## Why do we need this?
//!
//! This crate aims to provides a simple and safe interface for h264.
//!
//! ## How is this different from other h264 crates?
//!
//! This crate is only for encoding and decoding H.264 headers.
//!
//! ## Notable features
//!
//! This crate is a completely safe implementation of encoding and decoding H.264 headers.
//!
//! We mainly use this with scuffle-mp4 and scuffle-flv to work with mp4 and flv container formats respectively.
//!
//! ## Examples
//!
//! ### Parsing
//!
//! ```rust
//! use std::io;
//!
//! use bytes::Bytes;
//!
//! use scuffle_h264::{AVCDecoderConfigurationRecord, Sps};
//!
//! // A sample h264 bytestream to parse
//! # let bytes = Bytes::from(b"\x01d\0\x1f\xff\xe1\0\x17\x67\x64\x00\x1F\xAC\xD9\x41\xE0\x6D\xF9\xE6\xA0\x20\x20\x28\x00\x00\x00\x08\x00\x00\x01\xE0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0".to_vec());
//!
//! // Parsing
//! let result = AVCDecoderConfigurationRecord::parse(&mut io::Cursor::new(bytes)).unwrap();
//!
//! // Do something with it!
//!
//! // You can also parse an Sps from the Sps struct:
//! let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(&result.sps[0]));
//! ```
//!
//! For more examples, check out the tests in the source code for the parse function.
//!
//! ### Building
//!
//! ```rust
//! use bytes::Bytes;
//!
//! use scuffle_h264::{AVCDecoderConfigurationRecord, AvccExtendedConfig, Sps, SpsExtended};
//!
//! let extended_config = AvccExtendedConfig {
//!     chroma_format_idc: 1,
//!     bit_depth_luma_minus8: 0,
//!     bit_depth_chroma_minus8: 0,
//!     sequence_parameter_set_ext: vec![SpsExtended {
//!         chroma_format_idc: 1,
//!         separate_color_plane_flag: false,
//!         bit_depth_luma_minus8: 2,
//!         bit_depth_chroma_minus8: 3,
//!         qpprime_y_zero_transform_bypass_flag: false,
//!         scaling_matrix: vec![],
//!     }],
//! };
//! let config = AVCDecoderConfigurationRecord {
//!     configuration_version: 1,
//!     profile_indication: 100,
//!     profile_compatibility: 0,
//!     level_indication: 31,
//!     length_size_minus_one: 3,
//!     sps: vec![
//!         Bytes::from_static(b"spsdata"),
//!     ],
//!     pps: vec![Bytes::from_static(b"ppsdata")],
//!     extended_config: Some(extended_config),
//! };
//!
//! // Creating a buffer to store the built bytestream
//! let mut built = Vec::new();
//!
//! // Building
//! config.build(&mut built).unwrap();
//!
//! // Do something with it!
//! ```
//!
//! For more examples, check out the tests in the source code for the build function.
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
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

mod config;
mod enums;
mod sps;

pub use enums::*;
pub use sps::*;

pub use self::config::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
