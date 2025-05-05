//! General RTMP error type.

use crate::chunk::error::ChunkReadError;
use crate::command_messages::error::CommandError;
use crate::handshake::complex::error::ComplexHandshakeError;
use crate::session::server::ServerSessionError;

/// RTMP error.
#[derive(Debug, thiserror::Error)]
pub enum RtmpError {
    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Chunk read error.
    #[error("chunk read error: {0}")]
    ChunkRead(#[from] ChunkReadError),
    /// Command error.
    #[error("command error: {0}")]
    Command(#[from] CommandError),
    /// Complex handshake error.
    #[error("complex handshake error: {0}")]
    ComplexHandshake(#[from] ComplexHandshakeError),
    /// Session error.
    #[error("session error: {0}")]
    Session(#[from] ServerSessionError),
}

impl RtmpError {
    /// Returns true if the error indicates that the client has closed the connection.
    pub fn is_client_closed(&self) -> bool {
        match self {
            Self::Io(err) => matches!(
                err.kind(),
                std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::UnexpectedEof
            ),
            Self::Session(ServerSessionError::Timeout(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::future;
    use std::io::ErrorKind;
    use std::time::Duration;

    use crate::error::RtmpError;
    use crate::session::server::ServerSessionError;

    #[tokio::test]
    async fn test_is_client_closed() {
        assert!(RtmpError::Io(std::io::Error::new(ErrorKind::ConnectionAborted, "test")).is_client_closed());
        assert!(RtmpError::Io(std::io::Error::new(ErrorKind::ConnectionReset, "test")).is_client_closed());
        assert!(RtmpError::Io(std::io::Error::new(ErrorKind::UnexpectedEof, "test")).is_client_closed());

        let elapsed = tokio::time::timeout(Duration::ZERO, future::pending::<()>())
            .await
            .unwrap_err();

        assert!(RtmpError::Session(ServerSessionError::Timeout(elapsed)).is_client_closed());

        assert!(!RtmpError::Io(std::io::Error::other("test")).is_client_closed());
    }
}
