//! A pure-rust implementation of AMF0 encoder and decoder.
//!
//! This crate provides serde support for serialization and deserialization of AMF0 data.
//!
//! ## Specification
//!
//! | Name | Version | Link | Comments |
//! | --- | --- | --- | --- |
//! | Action Message Format -- AMF 0 | - | <https://rtmp.veriskope.com/pdf/amf0-file-format-specification.pdf> | Refered to as 'AMF0 spec' in this documentation |
//!
//! ## Limitations
//!
//! - Does not support AMF0 references.
//! - Does not support the AVM+ Type Marker. (see AMF 0 spec, 3.1)
//!
//! ## Example
//!
//! ```rust
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! # let bytes = &[0x02, 0, 1, b'a'];
//! # let mut writer = Vec::new();
//! // Decode a string value from bytes
//! let value: String = scuffle_amf0::from_slice(bytes)?;
//!
//! // .. do something with the value
//!
//! // Encode a value into a writer
//! scuffle_amf0::to_writer(&mut writer, &value)?;
//! # assert_eq!(writer, bytes);
//! # Ok(())
//! # }
//! # test().expect("test failed");
//! ```
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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod de;
pub mod decoder;
pub mod encoder;
pub mod error;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod ser;
pub mod value;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use de::{from_buf, from_reader, from_slice};
pub use decoder::Amf0Decoder;
pub use encoder::Amf0Encoder;
pub use error::{Amf0Error, Result};
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use ser::{to_bytes, to_writer};
pub use value::{Amf0Array, Amf0Object, Amf0Value};

/// AMF0 marker types.
///
/// Defined by:
/// - AMF 0 spec, 2.1.
#[derive(Debug, PartialEq, Eq, Clone, Copy, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum Amf0Marker {
    /// number-marker
    Number = 0x00,
    /// boolean-marker
    Boolean = 0x01,
    /// string-marker
    String = 0x02,
    /// object-marker
    Object = 0x03,
    /// movieclip-marker
    ///
    /// reserved, not supported
    MovieClipMarker = 0x04,
    /// null-marker
    Null = 0x05,
    /// undefined-marker
    Undefined = 0x06,
    /// reference-marker
    Reference = 0x07,
    /// ecma-array-marker
    EcmaArray = 0x08,
    /// object-end-marker
    ObjectEnd = 0x09,
    /// strict-array-marker
    StrictArray = 0x0a,
    /// date-marker
    Date = 0x0b,
    /// long-string-marker
    LongString = 0x0c,
    /// unsupported-marker
    Unsupported = 0x0d,
    /// recordset-marker
    ///
    /// reserved, not supported
    Recordset = 0x0e,
    /// xml-document-marker
    XmlDocument = 0x0f,
    /// typed-object-marker
    TypedObject = 0x10,
    /// avmplus-object-marker
    ///
    /// AMF3 marker
    AVMPlusObject = 0x11,
}
