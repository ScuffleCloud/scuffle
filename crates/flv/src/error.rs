#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid signature in header: 0x{0:x}")]
    InvalidSignature(u32),
    #[error("invalid data offset: {0}")]
    InvalidDataOffset(u32),
    #[error("tag encryption is not supported")]
    UnsupportedTagEncryption,
    #[error("nested audio multitracks are not allowed")]
    AudioNestedMultitracks,
    #[error("invalid modExData, expected at least {expected_bytes} bytes")]
    AudioInvalidModExData { expected_bytes: usize },
}
