use std::io;

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

use super::define::{Chunk, ChunkMessageHeader, ChunkType, INIT_CHUNK_SIZE};
use super::errors::ChunkWriteError;

/// A chunk writer.
///
/// This is used to write chunks into a stream.
pub struct ChunkWriter {
    chunk_size: usize,
}

impl Default for ChunkWriter {
    fn default() -> Self {
        Self {
            chunk_size: INIT_CHUNK_SIZE,
        }
    }
}

impl ChunkWriter {
    /// Set the chunk size.
    pub fn set_chunk_size(&mut self, chunk_size: usize) {
        self.chunk_size = chunk_size;
    }

    /// Internal function to write the basic header.
    #[inline]
    fn write_basic_header(io: &mut impl io::Write, fmt: ChunkType, csid: u32) -> Result<(), ChunkWriteError> {
        let fmt = fmt as u8;

        if csid >= 64 + 255 {
            io.write_u8((fmt << 6) | 1)?;
            let csid = csid - 64;

            let div = csid / 256;
            let rem = csid % 256;

            io.write_u8(rem as u8)?;
            io.write_u8(div as u8)?;
        } else if csid >= 64 {
            io.write_u8(fmt << 6)?;
            io.write_u8((csid - 64) as u8)?;
        } else {
            io.write_u8((fmt << 6) | csid as u8)?;
        }

        Ok(())
    }

    /// Internal function to write the message header.
    #[inline]
    fn write_message_header(io: &mut impl io::Write, message_header: &ChunkMessageHeader) -> Result<(), ChunkWriteError> {
        let timestamp = if message_header.timestamp >= 0xFFFFFF {
            0xFFFFFF
        } else {
            message_header.timestamp
        };

        io.write_u24::<BigEndian>(timestamp)?;
        io.write_u24::<BigEndian>(message_header.msg_length)?;
        io.write_u8(message_header.msg_type_id as u8)?;
        io.write_u32::<LittleEndian>(message_header.msg_stream_id)?;

        if message_header.is_extended_timestamp() {
            Self::write_extened_timestamp(io, message_header.timestamp)?;
        }

        Ok(())
    }

    /// Internal function to write the extended timestamp.
    #[inline]
    fn write_extened_timestamp(io: &mut impl io::Write, timestamp: u32) -> Result<(), ChunkWriteError> {
        io.write_u32::<BigEndian>(timestamp)?;

        Ok(())
    }

    /// Write a chunk into some writer.
    pub fn write_chunk(&self, io: &mut impl io::Write, mut chunk_info: Chunk) -> Result<(), ChunkWriteError> {
        Self::write_basic_header(io, ChunkType::Type0, chunk_info.basic_header.chunk_stream_id)?;

        Self::write_message_header(io, &chunk_info.message_header)?;

        while !chunk_info.payload.is_empty() {
            let cur_payload_size = if chunk_info.payload.len() > self.chunk_size {
                self.chunk_size
            } else {
                chunk_info.payload.len()
            };

            let payload_bytes = chunk_info.payload.split_to(cur_payload_size);
            io.write_all(&payload_bytes[..])?;

            if !chunk_info.payload.is_empty() {
                Self::write_basic_header(io, ChunkType::Type3, chunk_info.basic_header.chunk_stream_id)?;

                if chunk_info.message_header.is_extended_timestamp() {
                    Self::write_extened_timestamp(io, chunk_info.message_header.timestamp)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use bytes::Bytes;

    use super::*;
    use crate::messages::MessageType;

    #[test]
    fn test_writer_error_display() {
        let error = ChunkWriteError::UnknownReadState;
        assert_eq!(format!("{}", error), "unknown read state");

        let error = ChunkWriteError::Io(io::Error::from(io::ErrorKind::Other));
        assert_eq!(format!("{}", error), "io error: other error");
    }

    #[test]
    fn test_writer_write_small_chunk() {
        let writer = ChunkWriter::default();
        let mut buf = Vec::new();

        let chunk = Chunk::new(
            0,
            0,
            MessageType::Abort,
            0,
            Bytes::from(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        );

        writer.write_chunk(&mut buf, chunk).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf,
            vec![
                (0x00 << 6), // chunk basic header - fmt: 0, csid: 0
                0x00, 0x00, 0x00, // timestamp (0)
                0x00, 0x00, 0x08, // message length (8 bytes)
                0x02, // message type id (abort)
                0x00, 0x00, 0x00, 0x00, // message stream id (0)
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // message payload
            ]
        );
    }

    #[test]
    fn test_writer_write_large_chunk() {
        let writer = ChunkWriter::default();
        let mut buf = Vec::new();

        let mut payload = Vec::new();
        for i in 0..129 {
            payload.push(i);
        }

        let chunk = Chunk::new(10, 100, MessageType::Audio, 13, Bytes::from(payload));

        writer.write_chunk(&mut buf, chunk).unwrap();

        #[rustfmt::skip]
        let mut expected = vec![
            0x0A, // chunk basic header - fmt: 0, csid: 10 (the format should have been fixed to 0)
            0x00, 0x00, 0x64, // timestamp (100)
            0x00, 0x00, 0x81, // message length (129 bytes)
            0x08, // message type id (audio)
            0x0D, 0x00, 0x00, 0x00, // message stream id (13)
        ];

        for i in 0..128 {
            expected.push(i);
        }

        expected.push((0x03 << 6) | 0x0A); // chunk basic header - fmt: 3, csid: 10
        expected.push(128); // The rest of the payload should have been written

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_writer_extended_timestamp() {
        let writer = ChunkWriter::default();
        let mut buf = Vec::new();

        let chunk = Chunk::new(
            0,
            0xFFFFFFFF,
            MessageType::Abort,
            0,
            Bytes::from(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        );

        writer.write_chunk(&mut buf, chunk).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf,
            vec![
                (0x00 << 6), // chunk basic header - fmt: 0, csid: 0
                0xFF, 0xFF, 0xFF, // timestamp (0xFFFFFF)
                0x00, 0x00, 0x08, // message length (8 bytes)
                0x02, // message type id (abort)
                0x00, 0x00, 0x00,
                0x00, // message stream id (0)
                0xFF, 0xFF, 0xFF,
                0xFF, // extended timestamp (1)
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // message payload
            ]
        );
    }

    #[test]
    fn test_writer_extended_timestamp_ext() {
        let writer = ChunkWriter::default();
        let mut buf = Vec::new();

        let mut payload = Vec::new();
        for i in 0..129 {
            payload.push(i);
        }

        let chunk = Chunk::new(0, 0xFFFFFFFF, MessageType::Abort, 0, Bytes::from(payload));

        writer.write_chunk(&mut buf, chunk).unwrap();

        #[rustfmt::skip]
        let mut expected = vec![
            (0x00 << 6), // chunk basic header - fmt: 0, csid: 0
            0xFF, 0xFF, 0xFF, // timestamp (0xFFFFFF)
            0x00, 0x00, 0x81, // message length (8 bytes)
            0x02, // message type id (abort)
            0x00, 0x00, 0x00, 0x00, // message stream id (0)
            0xFF, 0xFF, 0xFF, 0xFF, // extended timestamp (1)
        ];

        for i in 0..128 {
            expected.push(i);
        }

        expected.push(0x03 << 6); // chunk basic header - fmt: 3, csid: 0
        expected.extend(vec![0xFF, 0xFF, 0xFF, 0xFF]); // extended timestamp
        expected.push(128); // The rest of the payload should have been written

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_writer_extended_csid() {
        let writer = ChunkWriter::default();
        let mut buf = Vec::new();

        let chunk = Chunk::new(
            64,
            0,
            MessageType::Abort,
            0,
            Bytes::from(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        );

        writer.write_chunk(&mut buf, chunk).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf,
            vec![
                (0x00 << 6), // chunk basic header - fmt: 0, csid: 0
                0x00, // extended csid (64 + 0) = 64
                0x00, 0x00, 0x00, // timestamp (0)
                0x00, 0x00, 0x08, // message length (8 bytes)
                0x02, // message type id (abort)
                0x00, 0x00, 0x00, 0x00, // message stream id (0)
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // message payload
            ]
        );
    }

    #[test]
    fn test_writer_extended_csid_ext() {
        let writer = ChunkWriter::default();
        let mut buf = Vec::new();

        let chunk = Chunk::new(
            320,
            0,
            MessageType::Abort,
            0,
            Bytes::from(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        );

        writer.write_chunk(&mut buf, chunk).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf,
            vec![
                0x01, // chunk basic header - fmt: 0, csid: 1
                0x00, // extended csid (64 + 0) = 64
                0x01, // extended csid (256 * 1) = 256 + 64 + 0 = 320
                0x00, 0x00, 0x00, // timestamp (0)
                0x00, 0x00, 0x08, // message length (8 bytes)
                0x02, // message type id (abort)
                0x00, 0x00, 0x00, 0x00, // message stream id (0)
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // message payload
            ]
        );
    }
}
