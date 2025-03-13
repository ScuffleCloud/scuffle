use std::io;

use bytes::Bytes;

use super::define::CommandResultLevel;
use super::{Command, CommandType};
use crate::chunk::{Chunk, ChunkStreamId, ChunkWriter};
use crate::messages::MessageType;

impl CommandResultLevel {
    pub fn to_str(&self) -> &str {
        match self {
            CommandResultLevel::Warning => "warning",
            CommandResultLevel::Status => "status",
            CommandResultLevel::Error => "error",
            CommandResultLevel::Unknown(s) => s,
        }
    }
}

impl Command<'_> {
    fn write_amf0_chunk(io: &mut impl io::Write, writer: &ChunkWriter, payload: Bytes) -> io::Result<()> {
        writer.write_chunk(
            io,
            Chunk::new(ChunkStreamId::Command.0, 0, MessageType::CommandAMF0, 0, payload),
        )
    }

    // The only AMF encoding supported by this server is AMF0
    // So we ignore the objectEncoding value sent by the client
    // and always use AMF0
    // - OBS does not support AMF3 (https://github.com/obsproject/obs-studio/blob/1be1f51635ac85b3ad768a88b3265b192bd0bf18/plugins/obs-outputs/librtmp/rtmp.c#L1737)
    // - Ffmpeg does not support AMF3 either (https://github.com/FFmpeg/FFmpeg/blob/c125860892e931d9b10f88ace73c91484815c3a8/libavformat/rtmpproto.c#L569)
    // - NginxRTMP does not support AMF3 (https://github.com/arut/nginx-rtmp-module/issues/313)
    // - SRS does not support AMF3 (https://github.com/ossrs/srs/blob/dcd02fe69cdbd7f401a7b8d139d95b522deb55b1/trunk/src/protocol/srs_protocol_rtmp_stack.cpp#L599)
    // However, the new enhanced-rtmp-v1 spec from YouTube does encourage the use of AMF3 over AMF0 (https://github.com/veovera/enhanced-rtmp)
    // We will eventually support this spec but for now we will stick to AMF0
    pub fn write(self, io: &mut impl io::Write, writer: &ChunkWriter) -> Result<(), crate::error::Error> {
        let mut buf = Vec::new();

        match self.net_command {
            CommandType::NetConnection(command) => {
                command.write(&mut buf, self.transaction_id)?;
            }
            CommandType::NetStream(command) => {
                command.write(&mut buf, self.transaction_id)?;
            }
            // don't write unknown commands
            CommandType::Unknown { .. } => {}
        }

        Self::write_amf0_chunk(io, writer, Bytes::from(buf))?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::super::CommandResultLevel;

    #[test]
    fn test_command_result_level_to_str() {
        assert_eq!(CommandResultLevel::Warning.to_str(), "warning");
        assert_eq!(CommandResultLevel::Status.to_str(), "status");
        assert_eq!(CommandResultLevel::Error.to_str(), "error");
        assert_eq!(CommandResultLevel::Unknown("custom".to_string()).to_str(), "custom");
    }
}
