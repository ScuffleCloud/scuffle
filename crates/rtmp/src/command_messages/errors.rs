#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("amf0 read: {0}")]
    Amf0Read(#[from] scuffle_amf0::Amf0ReadError),
    #[error("no app name of type string in connect command")]
    NoAppName,
    #[error("invalid publish command publishing type: {0}")]
    InvalidPublishingType(String),
    #[error("invalid command result level: {0}")]
    InvalidCommandResultLevel(String),
    #[error("invalid onStatus info object")]
    InvalidOnStatusInfoObject,
    #[error("chunk encode: {0}")]
    ChunkEncode(#[from] crate::chunk::ChunkEncodeError),
    #[error("amf0 write: {0}")]
    Amf0Write(#[from] scuffle_amf0::Amf0WriteError),
}
