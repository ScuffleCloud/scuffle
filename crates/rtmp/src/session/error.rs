#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("unknown stream id: {0}")]
    UnknownStreamID(u32),
    #[error("publisher disconnected")]
    PublisherDisconnected,
    #[error("timeout: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("received publish command before connect command")]
    PublishBeforeConnect,
    #[error("no stream name")]
    NoStreamName,
    #[error("publish request denied")]
    PublishRequestDenied,
    #[error("connect request denied")]
    ConnectRequestDenied,
    #[error("play not supported")]
    PlayNotSupported,
    #[error("publisher dropped")]
    PublisherDropped,
    #[error("invalid chunk size: {0}")]
    InvalidChunkSize(usize),
}
