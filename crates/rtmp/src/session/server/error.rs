//! Error type for server sessions.

/// Errors that can occur during a server session.
#[derive(Debug, thiserror::Error)]
pub enum ServerSessionError {
    /// Timeout.
    #[error("timeout: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    /// Received publish command before connect command.
    #[error("received publish command before connect command")]
    PublishBeforeConnect,
    /// Play not supported.
    #[error("play not supported")]
    PlayNotSupported,
    /// Invalid chunk size.
    #[error("invalid chunk size: {0}")]
    InvalidChunkSize(usize),
}
