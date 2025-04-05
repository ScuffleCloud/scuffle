//! Command error type.

/// Errors that can occur when processing command messages.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Amf0 error.
    #[error("amf0: {0}")]
    Amf0(#[from] scuffle_amf0::Amf0Error),
    /// Received an invalid onStatus info object.
    #[error("invalid onStatus info object")]
    InvalidOnStatusInfoObject,
    /// The RTMP client is not implemented yet.
    #[error("the rtmp client is not implemented yet")]
    NoClientImplementation,
}
