use crate::SessionError;
use crate::chunk::ChunkReadError;
use crate::command_messages::CommandError;
use crate::handshake::ComplexHandshakeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("chunk read error: {0}")]
    ChunkRead(#[from] ChunkReadError),
    #[error("command error: {0}")]
    Command(#[from] CommandError),
    #[error("complex handshake error: {0}")]
    ComplexHandshake(#[from] ComplexHandshakeError),
    #[error("session error: {0}")]
    Session(#[from] SessionError),
}

impl Error {
    pub fn is_client_closed(&self) -> bool {
        match self {
            Self::Io(err) => matches!(
                err.kind(),
                std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::UnexpectedEof
            ),
            Self::Session(SessionError::Timeout(_)) => true,
            _ => false,
        }
    }
}
