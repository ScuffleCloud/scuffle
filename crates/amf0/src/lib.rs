//! A pure-rust implementation of AMF0 encoder and decoder.
//!
//! This crate provides a simple interface for encoding and decoding AMF0 data.
//!
//! # Limitations
//!
//! - Does not support deserializing of AMF0 references.
//! - Does not support the AVM+ Type Marker. (see AMF 0 spec, 3.1)
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
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

pub mod de;
pub mod error;
pub mod ser;
pub mod value;

pub use de::{Deserializer, from_bytes, from_reader};
pub use error::{Amf0Error, Result};
pub use ser::{Serializer, to_bytes, to_writer};
pub use value::{Amf0Object, Amf0Value};

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
