use std::io;

use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;

use super::define::{
    ProtocolControlMessageSetChunkSize, ProtocolControlMessageSetPeerBandwidth,
    ProtocolControlMessageWindowAcknowledgementSize,
};
use super::errors::ProtocolControlMessageError;
use crate::chunk::{Chunk, ChunkWriter};
use crate::messages::MessageType;

impl ProtocolControlMessageSetChunkSize {
    pub fn write(&self, io: &mut impl io::Write, writer: &ChunkWriter) -> Result<(), ProtocolControlMessageError> {
        // According to spec the first bit must be 0.
        let chunk_size = self.chunk_size & 0x7FFFFFFF; // 31 bits only

        writer.write_chunk(
            io,
            Chunk::new(
                2, // chunk stream must be 2
                0, // timestamps are ignored
                MessageType::SetChunkSize,
                0, // message stream id is ignored
                Bytes::from(chunk_size.to_be_bytes().to_vec()),
            ),
        )?;

        Ok(())
    }
}

impl ProtocolControlMessageWindowAcknowledgementSize {
    pub fn write(&self, io: &mut impl io::Write, writer: &ChunkWriter) -> Result<(), ProtocolControlMessageError> {
        writer.write_chunk(
            io,
            Chunk::new(
                2, // chunk stream must be 2
                0, // timestamps are ignored
                MessageType::WindowAcknowledgementSize,
                0, // message stream id is ignored
                Bytes::from(self.acknowledgement_window_size.to_be_bytes().to_vec()),
            ),
        )?;

        Ok(())
    }
}

impl ProtocolControlMessageSetPeerBandwidth {
    pub fn write(&self, io: &mut impl io::Write, writer: &ChunkWriter) -> Result<(), ProtocolControlMessageError> {
        let mut data = Vec::new();
        data.write_u32::<BigEndian>(self.acknowledgement_window_size)
            .expect("Failed to write window size");
        data.write_u8(self.limit_type as u8).expect("Failed to write limit type");

        writer.write_chunk(
            io,
            Chunk::new(
                2, // chunk stream must be 2
                0, // timestamps are ignored
                MessageType::SetPeerBandwidth,
                0, // message stream id is ignored
                Bytes::from(data),
            ),
        )?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;
    use crate::chunk::ChunkReader;
    use crate::protocol_control_messages::ProtocolControlMessageSetPeerBandwidthLimitType;

    #[test]
    fn test_writer_write_set_chunk_size() {
        let writer = ChunkWriter::default();
        let mut buf = BytesMut::new();

        ProtocolControlMessageSetChunkSize { chunk_size: 1 }
            .write(&mut (&mut buf).writer(), &writer)
            .unwrap();

        let mut reader = ChunkReader::default();

        let chunk = reader.read_chunk(&mut buf).expect("read chunk").expect("chunk");
        assert_eq!(chunk.basic_header.chunk_stream_id, 0x02);
        assert_eq!(chunk.message_header.msg_type_id as u8, 0x01);
        assert_eq!(chunk.message_header.msg_stream_id, 0);
        assert_eq!(chunk.payload, vec![0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn test_writer_window_acknowledgement_size() {
        let writer = ChunkWriter::default();
        let mut buf = BytesMut::new();

        ProtocolControlMessageWindowAcknowledgementSize {
            acknowledgement_window_size: 1,
        }
        .write(&mut (&mut buf).writer(), &writer)
        .unwrap();

        let mut reader = ChunkReader::default();

        let chunk = reader.read_chunk(&mut buf).expect("read chunk").expect("chunk");
        assert_eq!(chunk.basic_header.chunk_stream_id, 0x02);
        assert_eq!(chunk.message_header.msg_type_id as u8, 0x05);
        assert_eq!(chunk.message_header.msg_stream_id, 0);
        assert_eq!(chunk.payload, vec![0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn test_writer_set_peer_bandwidth() {
        let writer = ChunkWriter::default();
        let mut buf = BytesMut::new();

        ProtocolControlMessageSetPeerBandwidth {
            acknowledgement_window_size: 1,
            limit_type: ProtocolControlMessageSetPeerBandwidthLimitType::Dynamic,
        }
        .write(&mut (&mut buf).writer(), &writer)
        .unwrap();

        let mut reader = ChunkReader::default();

        let chunk = reader.read_chunk(&mut buf).expect("read chunk").expect("chunk");
        assert_eq!(chunk.basic_header.chunk_stream_id, 0x02);
        assert_eq!(chunk.message_header.msg_type_id as u8, 0x06);
        assert_eq!(chunk.message_header.msg_stream_id, 0);
        assert_eq!(chunk.payload, vec![0x00, 0x00, 0x00, 0x01, 0x02]);
    }
}
