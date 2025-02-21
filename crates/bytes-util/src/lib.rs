//! Adds some helpful utilities for working with bits and bytes.
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

mod bit_read;
mod bit_write;
mod bytes_cursor;

pub use bit_read::BitReader;
pub use bit_write::BitWriter;
pub use bytes_cursor::{BytesCursor, BytesCursorExt};
