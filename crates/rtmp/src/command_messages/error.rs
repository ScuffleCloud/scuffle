#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("amf0 read: {0}")]
    Amf0Read(#[from] scuffle_amf0::Amf0ReadError),
    #[error("amf0 write: {0}")]
    Amf0Write(#[from] scuffle_amf0::Amf0WriteError),
    #[error("no app name of type string in connect command")]
    NoAppName,
    #[error("invalid onStatus info object")]
    InvalidOnStatusInfoObject,
    #[error("the rtmp client is not implemented yet")]
    NoClientImplementation,
}
