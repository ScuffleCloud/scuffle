//! Writing [`Command`].

use std::fmt::Display;
use std::io;

use bytes::{BufMut, Bytes, BytesMut};

use super::error::CommandError;
use super::{Command, CommandResultLevel, CommandType};
use crate::chunk::writer::ChunkWriter;
use crate::chunk::{CHUNK_STREAM_ID_COMMAND, Chunk};
use crate::error::RtmpError;
use crate::messages::MessageType;

impl AsRef<str> for CommandResultLevel {
    fn as_ref(&self) -> &str {
        match self {
            CommandResultLevel::Warning => "warning",
            CommandResultLevel::Status => "status",
            CommandResultLevel::Error => "error",
            CommandResultLevel::Unknown(s) => s,
        }
    }
}

impl Display for CommandResultLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResultLevel::Warning => write!(f, "warning"),
            CommandResultLevel::Status => write!(f, "status"),
            CommandResultLevel::Error => write!(f, "error"),
            CommandResultLevel::Unknown(s) => write!(f, "{s}"),
        }
    }
}

impl Command<'_> {
    fn write_amf0_chunk(io: &mut impl io::Write, writer: &ChunkWriter, payload: Bytes) -> io::Result<()> {
        writer.write_chunk(
            io,
            Chunk::new(CHUNK_STREAM_ID_COMMAND, 0, MessageType::CommandAMF0, 0, payload),
        )
    }

    /// Writes a [`Command`] to the given writer.
    ///
    /// Skips unknown commands.
    pub fn write(self, io: &mut impl io::Write, writer: &ChunkWriter) -> Result<(), RtmpError> {
        let mut buf = BytesMut::new();
        let mut buf_writer = (&mut buf).writer();

        match self.command_type {
            CommandType::NetConnection(command) => {
                command.write(&mut buf_writer, self.transaction_id)?;
            }
            CommandType::NetStream(_) => {
                return Err(RtmpError::from(CommandError::NoClientImplementation));
            }
            CommandType::OnStatus(command) => {
                command.write(&mut buf_writer, self.transaction_id)?;
            }
            // don't write unknown commands
            CommandType::Unknown { .. } => {}
        }

        Self::write_amf0_chunk(io, writer, buf.freeze())?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::super::{Command, CommandResultLevel};
    use crate::chunk::writer::ChunkWriter;
    use crate::command_messages::CommandType;
    use crate::command_messages::error::CommandError;
    use crate::command_messages::netstream::NetStreamCommand;
    use crate::error::RtmpError;

    #[test]
    fn command_result_level_to_str() {
        assert_eq!(CommandResultLevel::Warning.as_ref(), "warning");
        assert_eq!(CommandResultLevel::Status.as_ref(), "status");
        assert_eq!(CommandResultLevel::Error.as_ref(), "error");
        assert_eq!(CommandResultLevel::Unknown("custom".to_string()).as_ref(), "custom");
    }

    #[test]
    fn command_result_level_into_string() {
        assert_eq!(CommandResultLevel::Warning.to_string(), "warning");
        assert_eq!(CommandResultLevel::Status.to_string(), "status");
        assert_eq!(CommandResultLevel::Error.to_string(), "error");
        assert_eq!(CommandResultLevel::Unknown("custom".to_string()).to_string(), "custom");
    }

    #[test]
    fn netstream_command_write() {
        let mut buf = Vec::new();
        let writer = ChunkWriter::default();

        let err = Command {
            command_type: CommandType::NetStream(NetStreamCommand::CloseStream),
            transaction_id: 1.0,
        }
        .write(&mut buf, &writer)
        .unwrap_err();

        assert!(matches!(err, RtmpError::Command(CommandError::NoClientImplementation)));
    }
}
