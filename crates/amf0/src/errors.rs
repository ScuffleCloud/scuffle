use std::io;

use super::define::Amf0Marker;

/// Errors that can occur when decoding AMF0 data.
#[derive(Debug, thiserror::Error)]
pub enum Amf0ReadError {
    /// An unknown marker was encountered.
    #[error("unknown marker: {0}")]
    UnknownMarker(u8),
    /// An unsupported type was encountered.
    #[error("unsupported type: {0:?}")]
    UnsupportedType(Amf0Marker),
    /// A string parse error occurred.
    #[error("string parse error: {0}")]
    StringParseError(#[from] std::str::Utf8Error),
    /// An IO error occurred.
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    /// A wrong type was encountered. Created when using
    /// `Amf0Decoder::next_with_type` and the next value is not the expected
    /// type.
    #[error("wrong type: expected {expected:?}, got {got:?}")]
    WrongType {
        /// The expected type.
        expected: Amf0Marker,
        /// The actual type.
        got: Amf0Marker,
    },
}

/// Errors that can occur when encoding AMF0 data.
#[derive(Debug, thiserror::Error)]
pub enum Amf0WriteError {
    /// A normal string was too long.
    #[error("normal string too long")]
    NormalStringTooLong,
    /// An IO error occurred.
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    /// An unsupported type was encountered.
    #[error("unsupported type: {0:?}")]
    UnsupportedType(Amf0Marker),
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use byteorder::ReadBytesExt;
    use io::Cursor;

    use super::*;

    #[test]
    fn test_read_error_display() {
        let cases = [
            (Amf0ReadError::UnknownMarker(100), "unknown marker: 100"),
            (
                Amf0ReadError::UnsupportedType(Amf0Marker::Reference),
                "unsupported type: Reference",
            ),
            (
                Amf0ReadError::WrongType {
                    expected: Amf0Marker::Reference,
                    got: Amf0Marker::Boolean,
                },
                "wrong type: expected Reference, got Boolean",
            ),
            (
                Amf0ReadError::StringParseError(
                    #[allow(unknown_lints, invalid_from_utf8)]
                    std::str::from_utf8(b"\xFF\xFF").unwrap_err(),
                ),
                "string parse error: invalid utf-8 sequence of 1 bytes from index 0",
            ),
            (
                Amf0ReadError::Io(Cursor::new(Vec::<u8>::new()).read_u8().unwrap_err()),
                "io error: failed to fill whole buffer",
            ),
        ];

        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_write_error_display() {
        let cases = [
            (
                Amf0WriteError::UnsupportedType(Amf0Marker::ObjectEnd),
                "unsupported type: ObjectEnd",
            ),
            (
                Amf0WriteError::Io(Cursor::new(Vec::<u8>::new()).read_u8().unwrap_err()),
                "io error: failed to fill whole buffer",
            ),
            (Amf0WriteError::NormalStringTooLong, "normal string too long"),
        ];

        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }
}
