//! A pure-rust implementation of AMF0 encoder and decoder.
//!
//! This crate provides a simple interface for encoding and decoding AMF0 data.
//!
//! # Examples
//!
//! ```rust
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use scuffle_amf0::Amf0Decoder;
//! use scuffle_amf0::Amf0Encoder;
//! # let bytes = &[0x01, 0x01];
//! # let mut writer = Vec::new();
//!
//! // Create a new decoder
//! let mut reader = Amf0Decoder::new(bytes);
//! let value = reader.decode()?;
//!
//! // .. do something with the value
//!
//! // Encode a value into a writer
//! Amf0Encoder::encode(&mut writer, &value)?;
//!
//! # assert_eq!(writer, bytes);
//! # Ok(())
//! # }
//! # test().expect("test failed");
//! ```
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod decode;
mod define;
mod encode;
mod errors;

pub use crate::decode::Amf0Decoder;
pub use crate::define::{Amf0Marker, Amf0Value};
pub use crate::encode::Amf0Encoder;
pub use crate::errors::{Amf0ReadError, Amf0WriteError};
