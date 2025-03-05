use crate::command_messages::CommandError;
use crate::protocol_control_messages::ProtocolControlMessageError;

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("protocol control message error: {0}")]
    ProtocolControlMessage(#[from] ProtocolControlMessageError),
    #[error("command message: {0}")]
    CommandMessage(#[from] CommandError),
}
