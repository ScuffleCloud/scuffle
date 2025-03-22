use std::io;

use byteorder::{BigEndian, WriteBytesExt};

use super::define::{EventMessageStreamBegin, EventType};
use crate::chunk::{Chunk, ChunkWriter};
use crate::messages::MessageType;

impl EventMessageStreamBegin {
    pub fn write(&self, writer: &ChunkWriter, io: &mut impl io::Write) -> io::Result<()> {
        let mut data = Vec::new();

        data.write_u16::<BigEndian>(EventType::StreamBegin.0).expect("write u16");
        data.write_u32::<BigEndian>(self.stream_id).expect("write u32");

        writer.write_chunk(io, Chunk::new(0x02, 0, MessageType::UserControlEvent, 0, data.into()))?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, Bytes, BytesMut};

    use super::*;
    use crate::chunk::ChunkReader;
    use crate::user_control_messages::define::EventMessageStreamBegin;

    #[test]
    fn test_write_stream_begin() {
        let mut buf = BytesMut::new();
        let writer = ChunkWriter::default();

        EventMessageStreamBegin { stream_id: 1 }
            .write(&writer, &mut (&mut buf).writer())
            .unwrap();

        let mut reader = ChunkReader::default();

        let chunk = reader.read_chunk(&mut buf).expect("read chunk").expect("chunk");
        assert_eq!(chunk.basic_header.chunk_stream_id, 0x02);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x04);
        assert_eq!(chunk.message_header.msg_stream_id, 0);
        assert_eq!(chunk.payload, Bytes::from(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x01]));
    }
}
