use std::io;

use bytes::Bytes;

use super::define::CommandResultLevel;
use super::{Command, CommandError, CommandType};
use crate::chunk::{COMMAND_CHUNK_STREAM_ID, Chunk, ChunkEncodeError, ChunkEncoder};
use crate::messages::MessageType;

impl CommandResultLevel {
    pub fn to_str(&self) -> &'static str {
        match self {
            CommandResultLevel::Warning => "warning",
            CommandResultLevel::Status => "status",
            CommandResultLevel::Error => "error",
        }
    }
}

impl Command {
    fn write_amf0_chunk(
        writer: &mut impl io::Write,
        encoder: &ChunkEncoder,
        payload: Bytes,
    ) -> Result<(), ChunkEncodeError> {
        encoder.write_chunk(
            writer,
            Chunk::new(COMMAND_CHUNK_STREAM_ID, 0, MessageType::CommandAMF0, 0, payload),
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
    pub fn write(&self, writer: &mut impl io::Write, encoder: &ChunkEncoder) -> Result<(), CommandError> {
        let mut buf = Vec::new();

        match &self.net_command {
            CommandType::NetConnection(command) => {
                command.write(&mut buf, self.transaction_id)?;
            }
            CommandType::NetStream(command) => {
                command.write(&mut buf, self.transaction_id)?;
            }
            // don't write unknown commands
            CommandType::Unknown { .. } => {}
        }

        Self::write_amf0_chunk(writer, encoder, Bytes::from(buf))?;

        Ok(())
    }
}
