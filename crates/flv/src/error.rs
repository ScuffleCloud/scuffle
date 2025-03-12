use crate::video::body::enhanced::metadata::MetadataColorInfoError;

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
    #[error("nested multitracks are not allowed")]
    NestedMultitracks,
    #[error("invalid modExData, expected at least {expected_bytes} bytes")]
    InvalidModExData { expected_bytes: usize },
    #[error("amf0 read: {0}")]
    Amf0Read(#[from] scuffle_amf0::Amf0ReadError),
    #[error("color info metadata: {0}")]
    ColorInfoMetadata(#[from] MetadataColorInfoError),
}
