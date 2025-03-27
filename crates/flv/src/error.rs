//! Error types.

/// Error type for FLV processing.
#[derive(Debug, thiserror::Error)]
pub enum FlvError {
    /// IO error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// The FLV signature (magic bytes) is invalid.
    #[error("invalid signature in header: 0x{0:x}")]
    InvalidSignature(u32),
    /// The data offset in the FLV header is invalid.
    #[error("invalid data offset: {0}")]
    InvalidDataOffset(u32),
    /// Multitracks cannot be nested.
    #[error("nested multitracks are not allowed")]
    NestedMultitracks,
    /// Invalid modExData.
    #[error("invalid modExData, expected at least {expected_bytes} bytes")]
    InvalidModExData {
        /// The expected number of bytes.
        expected_bytes: usize,
    },
    /// AMF0 error.
    #[error("amf0: {0}")]
    Amf0(#[from] scuffle_amf0::Amf0Error),
}
