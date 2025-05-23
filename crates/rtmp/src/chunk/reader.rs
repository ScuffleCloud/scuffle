//! Types and functions for reading RTMP chunks.

use std::cmp::min;
use std::collections::HashMap;
use std::io::{self, Cursor, Seek, SeekFrom};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use bytes::BytesMut;
use num_traits::FromPrimitive;

use super::error::ChunkReadError;
use super::{Chunk, ChunkBasicHeader, ChunkMessageHeader, ChunkType, INIT_CHUNK_SIZE, MAX_CHUNK_SIZE};
use crate::messages::MessageType;

// These constants are used to limit the amount of memory we use for partial
// chunks on normal operations we should never hit these limits
// This is for when someone is trying to send us a malicious chunk streams
const MAX_PARTIAL_CHUNK_SIZE: usize = 10 * 1024 * 1024; // 10MB (should be more than enough)
const MAX_PREVIOUS_CHUNK_HEADERS: usize = 100; // 100 chunks
const MAX_PARTIAL_CHUNK_COUNT: usize = 4; // 4 chunks

/// A chunk reader.
///
/// This is used to read chunks from a stream.
pub struct ChunkReader {
    /// According to the spec chunk streams are identified by the chunk stream
    /// ID. In this case that is our key.
    /// We then have a chunk header (since some chunks refer to the previous
    /// chunk header)
    previous_chunk_headers: HashMap<u32, ChunkMessageHeader>,

    /// Technically according to the spec, we can have multiple message streams
    /// in a single chunk stream. Because of this the key of this map is a tuple
    /// (chunk stream id, message stream id).
    partial_chunks: HashMap<(u32, u32), BytesMut>,

    /// This is the max chunk size that the client has specified.
    /// By default this is 128 bytes.
    max_chunk_size: usize,
}

impl Default for ChunkReader {
    fn default() -> Self {
        Self {
            previous_chunk_headers: HashMap::with_capacity(MAX_PREVIOUS_CHUNK_HEADERS),
            partial_chunks: HashMap::with_capacity(MAX_PARTIAL_CHUNK_COUNT),
            max_chunk_size: INIT_CHUNK_SIZE,
        }
    }
}

impl ChunkReader {
    /// Call when a client requests a chunk size change.
    ///
    /// Returns `false` if the chunk size is out of bounds.
    /// The connection should be closed in this case.
    pub fn update_max_chunk_size(&mut self, chunk_size: usize) -> bool {
        // We need to make sure that the chunk size is within the allowed range.
        // Returning false here should close the connection.
        if !(INIT_CHUNK_SIZE..=MAX_CHUNK_SIZE).contains(&chunk_size) {
            false
        } else {
            self.max_chunk_size = chunk_size;
            true
        }
    }

    /// This function is used to read a chunk from the buffer.
    ///
    /// Returns:
    /// - `Ok(None)` if the buffer does not contain enough data to read a full chunk.
    /// - `Ok(Some(Chunk))` if a full chunk is read.
    /// - `Err(ChunkReadError)` if there is an error decoding a chunk. The connection should be closed.
    ///
    /// # See also
    ///
    /// - [`Chunk`]
    /// - [`ChunkReadError`]
    pub fn read_chunk(&mut self, buffer: &mut BytesMut) -> Result<Option<Chunk>, crate::error::RtmpError> {
        // We do this in a loop because we may have multiple chunks in the buffer,
        // And those chunks may be partial chunks thus we need to keep reading until we
        // have a full chunk or we run out of data.
        loop {
            // The cursor is an advanced cursor that is a reference to the buffer.
            // This means the cursor does not advance the reader's position.
            // Thus allowing us to backtrack if we need to read more data.
            let mut cursor = std::io::Cursor::new(buffer.as_ref());

            let header = match self.read_header(&mut cursor) {
                Ok(header) => header,
                Err(None) => {
                    // Returning none here means that the buffer is empty and we need to wait for
                    // more data.
                    return Ok(None);
                }
                Err(Some(err)) => {
                    // This is an error that we can't recover from, so we return it.
                    // The connection will be closed.
                    return Err(crate::error::RtmpError::Io(err));
                }
            };

            let message_header = match self.read_message_header(&header, &mut cursor) {
                Ok(message_header) => message_header,
                Err(None) => {
                    // Returning none here means that the buffer is empty and we need to wait for
                    // more data.
                    return Ok(None);
                }
                Err(Some(err)) => {
                    // This is an error that we can't recover from, so we return it.
                    // The connection will be closed.
                    return Err(err);
                }
            };

            let (payload_range_start, payload_range_end) =
                match self.get_payload_range(&header, &message_header, &mut cursor) {
                    Ok(data) => data,
                    Err(None) => {
                        // Returning none here means that the buffer is empty and we need to wait
                        // for more data.
                        return Ok(None);
                    }
                    Err(Some(err)) => {
                        // This is an error that we can't recover from, so we return it.
                        // The connection will be closed.
                        return Err(err);
                    }
                };

            // Since we were reading from an advanced cursor, our reads did not actually
            // advance the reader's position. We need to manually advance the reader's
            // position to the cursor's position.
            let position = cursor.position() as usize;
            if position > buffer.len() {
                // In some cases we dont have enough data yet to read the chunk.
                // We return Ok(None) here and the loop will continue.
                return Ok(None);
            }

            let data = buffer.split_to(position);

            // We freeze the chunk data and slice it to get the payload.
            // Data before the slice is the header data, and data after the slice is the
            // next chunk We don't need to keep the header data, because we already decoded
            // it into struct form. The payload_range_end should be the same as the cursor's
            // position.
            let payload = data.freeze().slice(payload_range_start..payload_range_end);

            // We need to check here if the chunk header is already stored in our map.
            // This isnt a spec check but it is a check to make sure that we dont have too
            // many previous chunk headers stored in memory.
            let count = if self.previous_chunk_headers.contains_key(&header.chunk_stream_id) {
                self.previous_chunk_headers.len()
            } else {
                self.previous_chunk_headers.len() + 1
            };

            // If this is hit, then we have too many previous chunk headers stored in
            // memory. And the client is probably trying to DoS us.
            // We return an error and the connection will be closed.
            if count > MAX_PREVIOUS_CHUNK_HEADERS {
                return Err(crate::error::RtmpError::ChunkRead(
                    ChunkReadError::TooManyPreviousChunkHeaders,
                ));
            }

            // We insert the chunk header into our map.
            self.previous_chunk_headers
                .insert(header.chunk_stream_id, message_header.clone());

            // It is possible in theory to get a chunk message that requires us to change
            // the max chunk size. However the size of that message is smaller than the
            // default max chunk size. Therefore we can ignore this case.
            // Since if we get such a message we will read it and the payload.len() will be
            // equal to the message length. and thus we will return the chunk.

            // Check if the payload is the same as the message length.
            // If this is true we have a full chunk and we can return it.
            if payload.len() == message_header.msg_length as usize {
                return Ok(Some(Chunk {
                    basic_header: header,
                    message_header,
                    payload,
                }));
            } else {
                // Otherwise we generate a key using the chunk stream id and the message stream
                // id. We then get the partial chunk from the map using the key.
                let key = (header.chunk_stream_id, message_header.msg_stream_id);
                let partial_chunk = match self.partial_chunks.get_mut(&key) {
                    Some(partial_chunk) => partial_chunk,
                    None => {
                        // If it does not exists we create a new one.
                        // If we have too many partial chunks we return an error.
                        // Since the client is probably trying to DoS us.
                        // The connection will be closed.
                        if self.partial_chunks.len() >= MAX_PARTIAL_CHUNK_COUNT {
                            return Err(crate::error::RtmpError::ChunkRead(ChunkReadError::TooManyPartialChunks));
                        }

                        // Insert a new empty BytesMut into the map.
                        self.partial_chunks.insert(key, BytesMut::new());
                        // Get the partial chunk we just inserted.
                        self.partial_chunks.get_mut(&key).expect("we just inserted it")
                    }
                };

                // We extend the partial chunk with the payload.
                // And get the new length of the partial chunk.
                let length = {
                    // If the length of a single chunk is larger than the max partial chunk size
                    // we return an error. The client is probably trying to DoS us.
                    if partial_chunk.len() + payload.len() > MAX_PARTIAL_CHUNK_SIZE {
                        return Err(crate::error::RtmpError::ChunkRead(ChunkReadError::PartialChunkTooLarge(
                            partial_chunk.len() + payload.len(),
                        )));
                    }

                    // Extend the partial chunk with the payload.
                    partial_chunk.extend_from_slice(&payload[..]);

                    // Return the new length of the partial chunk.
                    partial_chunk.len()
                };

                // If we have a full chunk we return it.
                if length == message_header.msg_length as usize {
                    return Ok(Some(Chunk {
                        basic_header: header,
                        message_header,
                        payload: self.partial_chunks.remove(&key).unwrap().freeze(),
                    }));
                }

                // If we don't have a full chunk we just let the loop continue.
                // Usually this will result in returning Ok(None) from one of
                // the above checks. However there is a edge case that we have
                // enough data in our buffer to read the next chunk and the
                // client is waiting for us to send a response. Meaning if we
                // just return Ok(None) here We would deadlock the connection,
                // and it will eventually timeout. So we need to loop again here
                // to check if we have enough data to read the next chunk.
            }
        }
    }

    /// Internal function used to read the basic chunk header.
    fn read_header(&self, cursor: &mut Cursor<&[u8]>) -> Result<ChunkBasicHeader, Option<io::Error>> {
        // The first byte of the basic header is the format of the chunk and the stream
        // id. Mapping the error to none means that this isn't a real error but we dont
        // have enough data.
        let byte = cursor.read_u8().eof_to_none()?;
        // The format is the first 2 bits of the byte. We shift the byte 6 bits to the
        // right to get the format.
        let format = (byte >> 6) & 0b00000011;

        // We do not check that the format is valid.
        // It should not be possible to get an invalid chunk type
        // because, we bitshift the byte 6 bits to the right. Leaving 2 bits which can
        // only be 0, 1 or 2 or 3 which is the only valid chunk types.
        let format = ChunkType::from_u8(format).expect("unreachable");

        // We then parse the chunk stream id.
        let chunk_stream_id = match (byte & 0b00111111) as u32 {
            // If the chunk stream id is 0 we read the next byte and add 64 to it.
            0 => {
                let first_byte = cursor.read_u8().eof_to_none()?;

                64 + first_byte as u32
            }
            // If it is 1 we read the next 2 bytes and add 64 to it and multiply the 2nd byte by
            // 256.
            1 => {
                let first_byte = cursor.read_u8().eof_to_none()?;
                let second_byte = cursor.read_u8().eof_to_none()?;

                64 + first_byte as u32 + second_byte as u32 * 256
            }
            // Any other value means that the chunk stream id is the value of the byte.
            csid => csid,
        };

        // We then read the message header.
        let header = ChunkBasicHeader { chunk_stream_id, format };

        Ok(header)
    }

    /// Internal function used to read the message header.
    fn read_message_header(
        &self,
        header: &ChunkBasicHeader,
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<ChunkMessageHeader, Option<crate::error::RtmpError>> {
        // Each format has a different message header length.
        match header.format {
            // Type0 headers have the most information and can be compared to keyframes in video.
            // They do not reference any previous chunks. They contain the full message header.
            ChunkType::Type0 => {
                // The first 3 bytes are the timestamp.
                let timestamp = cursor
                    .read_u24::<BigEndian>()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                // Followed by a 3 byte message length. (this is the length of the entire
                // payload not just this chunk)
                let msg_length = cursor
                    .read_u24::<BigEndian>()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                if msg_length as usize > MAX_PARTIAL_CHUNK_SIZE {
                    return Err(Some(crate::error::RtmpError::ChunkRead(
                        ChunkReadError::PartialChunkTooLarge(msg_length as usize),
                    )));
                }

                // We then have a 1 byte message type id.
                let msg_type_id = cursor
                    .read_u8()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                let msg_type_id = MessageType::from(msg_type_id);

                // We then read the message stream id. (According to spec this is stored in
                // LittleEndian, no idea why.)
                let msg_stream_id = cursor
                    .read_u32::<LittleEndian>()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;

                // Sometimes the timestamp is larger than 3 bytes.
                // If the timestamp is 0xFFFFFF we read the next 4 bytes as the timestamp.
                // I am not exactly sure why they did it this way.
                // Why not just use 3 bytes for the timestamp, and if the 3 bytes are set to
                // 0xFFFFFF just read 1 additional byte and then shift it 24 bits.
                // Like if timestamp == 0xFFFFFF { timestamp |= cursor.read_u8() << 24; }
                // This would save 3 bytes in the header and would be more
                // efficient but I guess the Spec writers are smarter than me.
                let (timestamp, was_extended_timestamp) = if timestamp == 0xFFFFFF {
                    (
                        cursor
                            .read_u32::<BigEndian>()
                            .eof_to_none()
                            .map_err(|e| e.map(crate::error::RtmpError::Io))?,
                        true,
                    )
                } else {
                    (timestamp, false)
                };

                Ok(ChunkMessageHeader {
                    timestamp,
                    msg_length,
                    msg_type_id,
                    msg_stream_id,
                    was_extended_timestamp,
                })
            }
            // For ChunkType 1 we have a delta timestamp, message length and message type id.
            // The message stream id is the same as the previous chunk.
            ChunkType::Type1 => {
                // The first 3 bytes are the delta timestamp.
                let timestamp_delta = cursor
                    .read_u24::<BigEndian>()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                // Followed by a 3 byte message length. (this is the length of the entire
                // payload not just this chunk)
                let msg_length = cursor
                    .read_u24::<BigEndian>()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                if msg_length as usize > MAX_PARTIAL_CHUNK_SIZE {
                    return Err(Some(crate::error::RtmpError::ChunkRead(
                        ChunkReadError::PartialChunkTooLarge(msg_length as usize),
                    )));
                }

                // We then have a 1 byte message type id.
                let msg_type_id = cursor
                    .read_u8()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                let msg_type_id = MessageType::from(msg_type_id);

                // Again as mentioned above we sometimes have a delta timestamp larger than 3
                // bytes.
                let (timestamp_delta, was_extended_timestamp) = if timestamp_delta == 0xFFFFFF {
                    (
                        cursor
                            .read_u32::<BigEndian>()
                            .eof_to_none()
                            .map_err(|e| e.map(crate::error::RtmpError::Io))?,
                        true,
                    )
                } else {
                    (timestamp_delta, false)
                };

                // We get the previous chunk header.
                // If the previous chunk header is not found we return an error. (this is a real
                // error)
                let previous_header =
                    self.previous_chunk_headers
                        .get(&header.chunk_stream_id)
                        .ok_or(crate::error::RtmpError::ChunkRead(
                            ChunkReadError::MissingPreviousChunkHeader(header.chunk_stream_id),
                        ))?;

                // We calculate the timestamp by adding the delta timestamp to the previous
                // timestamp. We need to make sure this does not overflow.
                let timestamp = previous_header.timestamp.checked_add(timestamp_delta).unwrap_or_else(|| {
                    tracing::warn!(
						"Timestamp overflow detected. Previous timestamp: {}, delta timestamp: {}, using previous timestamp.",
						previous_header.timestamp,
						timestamp_delta
					);

                    previous_header.timestamp
                });

                Ok(ChunkMessageHeader {
                    timestamp,
                    msg_length,
                    msg_type_id,
                    was_extended_timestamp,
                    // The message stream id is the same as the previous chunk.
                    msg_stream_id: previous_header.msg_stream_id,
                })
            }
            // ChunkType2 headers only have a delta timestamp.
            // The message length, message type id and message stream id are the same as the
            // previous chunk.
            ChunkType::Type2 => {
                // We read the delta timestamp.
                let timestamp_delta = cursor
                    .read_u24::<BigEndian>()
                    .eof_to_none()
                    .map_err(|e| e.map(crate::error::RtmpError::Io))?;

                // Again if the delta timestamp is larger than 3 bytes we read the next 4 bytes
                // as the timestamp.
                let (timestamp_delta, was_extended_timestamp) = if timestamp_delta == 0xFFFFFF {
                    (
                        cursor
                            .read_u32::<BigEndian>()
                            .eof_to_none()
                            .map_err(|e| e.map(crate::error::RtmpError::Io))?,
                        true,
                    )
                } else {
                    (timestamp_delta, false)
                };

                // We get the previous chunk header.
                // If the previous chunk header is not found we return an error. (this is a real
                // error)
                let previous_header =
                    self.previous_chunk_headers
                        .get(&header.chunk_stream_id)
                        .ok_or(crate::error::RtmpError::ChunkRead(
                            ChunkReadError::MissingPreviousChunkHeader(header.chunk_stream_id),
                        ))?;

                // We calculate the timestamp by adding the delta timestamp to the previous
                // timestamp.
                let timestamp = previous_header.timestamp + timestamp_delta;

                Ok(ChunkMessageHeader {
                    timestamp,
                    msg_length: previous_header.msg_length,
                    msg_type_id: previous_header.msg_type_id,
                    msg_stream_id: previous_header.msg_stream_id,
                    was_extended_timestamp,
                })
            }
            // ChunkType3 headers are the same as the previous chunk header.
            ChunkType::Type3 => {
                // We get the previous chunk header.
                // If the previous chunk header is not found we return an error. (this is a real
                // error)
                let previous_header = self
                    .previous_chunk_headers
                    .get(&header.chunk_stream_id)
                    .ok_or(crate::error::RtmpError::ChunkRead(
                        ChunkReadError::MissingPreviousChunkHeader(header.chunk_stream_id),
                    ))?
                    .clone();

                // Now this is truely stupid.
                // If the PREVIOUS HEADER is extended then we now waste an additional 4 bytes to
                // read the timestamp. Why not just read the timestamp in the previous header if
                // it is extended? I guess the spec writers had some reason and its obviously
                // way above my knowledge.
                if previous_header.was_extended_timestamp {
                    // Not a real error, we just dont have enough data.
                    // We dont have to store this value since it is the same as the previous header.
                    cursor
                        .read_u32::<BigEndian>()
                        .eof_to_none()
                        .map_err(|e| e.map(crate::error::RtmpError::Io))?;
                }

                Ok(previous_header)
            }
        }
    }

    /// Internal function to get the payload range of a chunk.
    fn get_payload_range(
        &self,
        header: &ChunkBasicHeader,
        message_header: &ChunkMessageHeader,
        cursor: &mut Cursor<&'_ [u8]>,
    ) -> Result<(usize, usize), Option<crate::error::RtmpError>> {
        // We find out if the chunk is a partial chunk (and if we have already read some
        // of it).
        let key = (header.chunk_stream_id, message_header.msg_stream_id);

        // Check how much we still need to read (if we have already read some of the
        // chunk)
        let remaining_read_length =
            message_header.msg_length as usize - self.partial_chunks.get(&key).map(|data| data.len()).unwrap_or(0);

        // We get the min between our max chunk size and the remaining read length.
        // This is the amount of bytes we need to read.
        let need_read_length = min(remaining_read_length, self.max_chunk_size);

        // We get the current position in the cursor.
        let pos = cursor.position() as usize;

        // We seek forward to where the payload starts.
        cursor
            .seek(SeekFrom::Current(need_read_length as i64))
            .eof_to_none()
            .map_err(|e| e.map(crate::error::RtmpError::Io))?;

        // We then return the range of the payload.
        // Which would be the pos to the pos + need_read_length.
        Ok((pos, pos + need_read_length))
    }
}

trait IoResultExt<T> {
    fn eof_to_none(self) -> Result<T, Option<io::Error>>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn eof_to_none(self) -> Result<T, Option<io::Error>> {
        self.map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                None
            } else {
                Some(e)
            }
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use byteorder::WriteBytesExt;
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_reader_error_display() {
        let error = ChunkReadError::MissingPreviousChunkHeader(123);
        assert_eq!(format!("{error}"), "missing previous chunk header: 123");

        let error = ChunkReadError::TooManyPartialChunks;
        assert_eq!(format!("{error}"), "too many partial chunks");

        let error = ChunkReadError::TooManyPreviousChunkHeaders;
        assert_eq!(format!("{error}"), "too many previous chunk headers");

        let error = ChunkReadError::PartialChunkTooLarge(100);
        assert_eq!(format!("{error}"), "partial chunk too large: 100");
    }

    #[test]
    fn test_reader_chunk_size_out_of_bounds() {
        let mut reader = ChunkReader::default();
        assert!(!reader.update_max_chunk_size(MAX_CHUNK_SIZE + 1));
    }

    #[test]
    fn test_incomplete_header() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0b00_000000]);

        let reader = ChunkReader::default();
        let err = reader.read_header(&mut Cursor::new(&buf));
        assert!(matches!(err, Err(None)));
    }

    #[test]
    fn test_reader_chunk_type0_single_sized() {
        let mut buf = BytesMut::new();

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x80, // message length (128) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let mut unpacker = ChunkReader::default();
        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");
        assert_eq!(chunk.basic_header.chunk_stream_id, 3);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x09);
        assert_eq!(chunk.message_header.timestamp, 0);
        assert_eq!(chunk.message_header.msg_length, 128);
        assert_eq!(chunk.message_header.msg_stream_id, 0x0100); // since it's little endian, it's 0x0100
        assert_eq!(chunk.payload.len(), 128);
    }

    #[test]
    fn test_reader_chunk_type0_double_sized() {
        let mut buf = BytesMut::new();
        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0x00, 0x00, 0x00, // timestamp
            0x00, 0x01, 0x00, // message length (256) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let mut unpacker = ChunkReader::default();

        let chunk = buf.as_ref().to_vec();

        // We should not have enough data to read the chunk
        // But the chunk is valid, so we should not get an error
        assert!(unpacker.read_chunk(&mut buf).expect("read chunk").is_none());

        // We just feed the same data again in this test to see if the Unpacker merges
        // the chunks Which it should do
        buf.extend_from_slice(&chunk);

        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 3);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x09);
        assert_eq!(chunk.message_header.timestamp, 0);
        assert_eq!(chunk.message_header.msg_length, 256);
        assert_eq!(chunk.message_header.msg_stream_id, 0x0100); // since it's little endian, it's 0x0100
        assert_eq!(chunk.payload.len(), 256);
    }

    #[test]
    fn test_reader_chunk_mutli_streams() {
        let mut buf = BytesMut::new();

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0x00, 0x00, 0x00, // timestamp
            0x00, 0x01, 0x00, // message length (256) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
        ]);

        for _ in 0..128 {
            (&mut buf).writer().write_u8(3).unwrap();
        }

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            4, // chunk type 0, chunk stream id 4 (different stream)
            0x00, 0x00, 0x00, // timestamp
            0x00, 0x01, 0x00, // message length (256) (max chunk size is set to 128)
            0x08, // message type id (audio)
            0x00, 0x03, 0x00, 0x00, // message stream id
        ]);

        for _ in 0..128 {
            (&mut buf).writer().write_u8(4).unwrap();
        }

        let mut unpacker = ChunkReader::default();

        // We wrote 2 chunks but neither of them are complete
        assert!(unpacker.read_chunk(&mut buf).expect("read chunk").is_none());

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (3 << 6) | 4, // chunk type 3, chunk stream id 4
        ]);

        for _ in 0..128 {
            (&mut buf).writer().write_u8(3).unwrap();
        }

        // Even though we wrote chunk 3 first, chunk 4 should be read first since it's a
        // different stream
        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 4);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x08);
        assert_eq!(chunk.message_header.timestamp, 0);
        assert_eq!(chunk.message_header.msg_length, 256);
        assert_eq!(chunk.message_header.msg_stream_id, 0x0300); // since it's little endian, it's 0x0100
        assert_eq!(chunk.payload.len(), 256);
        for i in 0..128 {
            assert_eq!(chunk.payload[i], 4);
        }

        // No chunk is ready yet
        assert!(unpacker.read_chunk(&mut buf).expect("read chunk").is_none());

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (3 << 6) | 3, // chunk type 3, chunk stream id 3
        ]);

        for _ in 0..128 {
            (&mut buf).writer().write_u8(3).unwrap();
        }

        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 3);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x09);
        assert_eq!(chunk.message_header.timestamp, 0);
        assert_eq!(chunk.message_header.msg_length, 256);
        assert_eq!(chunk.message_header.msg_stream_id, 0x0100); // since it's little endian, it's 0x0100
        assert_eq!(chunk.payload.len(), 256);
        for i in 0..128 {
            assert_eq!(chunk.payload[i], 3);
        }
    }

    #[test]
    fn test_reader_extended_timestamp() {
        let mut buf = BytesMut::new();

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0xFF, 0xFF, 0xFF, // timestamp
            0x00, 0x02, 0x00, // message length (384) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
            0x01, 0x00, 0x00, 0x00, // extended timestamp
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let mut unpacker = ChunkReader::default();

        // We should not have enough data to read the chunk
        // But the chunk is valid, so we should not get an error
        assert!(unpacker.read_chunk(&mut buf).expect("read chunk").is_none());

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (1 << 6) | 3, // chunk type 1, chunk stream id 3
            0xFF, 0xFF, 0xFF, // extended timestamp (again)
            0x00, 0x02, 0x00, // message length (384) (max chunk size is set to 128)
            0x09, // message type id (video)
            // message stream id is not present since it's the same as the previous chunk
            0x01, 0x00, 0x00, 0x00, // extended timestamp (again)
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (2 << 6) | 3, // chunk type 3, chunk stream id 3
            0x00, 0x00, 0x01, // not extended timestamp
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (3 << 6) | 3, // chunk type 3, chunk stream id 3
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 3);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x09);
        assert_eq!(chunk.message_header.timestamp, 0x02000001);
        assert_eq!(chunk.message_header.msg_length, 512);
        assert_eq!(chunk.message_header.msg_stream_id, 0x0100); // since it's little endian, it's 0x0100
        assert_eq!(chunk.payload.len(), 512);
    }

    #[test]
    fn test_reader_extended_timestamp_ext() {
        let mut buf = BytesMut::new();

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0xFF, 0xFF, 0xFF, // timestamp
            0x00, 0x01, 0x00, // message length (256) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
            0x01, 0x00, 0x00, 0x00, // extended timestamp
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let mut unpacker = ChunkReader::default();

        // We should not have enough data to read the chunk
        // But the chunk is valid, so we should not get an error
        assert!(unpacker.read_chunk(&mut buf).expect("read chunk").is_none());

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (3 << 6) | 3, // chunk type 1, chunk stream id 3
            0x00, 0x00, 0x00, 0x00, // extended timestamp this value is ignored
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 3);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x09);
        assert_eq!(chunk.message_header.timestamp, 0x01000000);
        assert_eq!(chunk.message_header.msg_length, 256);
        assert_eq!(chunk.message_header.msg_stream_id, 0x0100); // since it's little endian, it's 0x0100
        assert_eq!(chunk.payload.len(), 256);
    }

    #[test]
    fn test_read_extended_csid() {
        let mut buf = BytesMut::new();

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (0 << 6), // chunk type 0, chunk stream id 0
            10,       // extended chunk stream id
            0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x00, // message length (256) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
        ]);

        let mut unpacker = ChunkReader::default();
        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 64 + 10);
    }

    #[test]
    fn test_read_extended_csid_ext2() {
        let mut buf = BytesMut::new();

        #[rustfmt::skip]
        buf.extend_from_slice(&[
            1,  // chunk type 0, chunk stream id 0
            10, // extended chunk stream id
            13, // extended chunk stream id 2
            0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x00, // message length (256) (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
        ]);

        let mut unpacker = ChunkReader::default();

        let chunk = unpacker.read_chunk(&mut buf).expect("read chunk").expect("chunk");

        assert_eq!(chunk.basic_header.chunk_stream_id, 64 + 10 + 256 * 13);
    }

    #[test]
    fn test_reader_error_no_previous_chunk() {
        let mut buf = BytesMut::new();

        // Write a chunk with type 3 but no previous chunk
        #[rustfmt::skip]
        buf.extend_from_slice(&[
            (3 << 6) | 3, // chunk type 0, chunk stream id 3
        ]);

        let mut unpacker = ChunkReader::default();
        let err = unpacker.read_chunk(&mut buf).unwrap_err();
        match err {
            crate::error::RtmpError::ChunkRead(ChunkReadError::MissingPreviousChunkHeader(3)) => {}
            _ => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn test_reader_error_partial_chunk_too_large() {
        let mut buf = BytesMut::new();

        // Write a chunk that has a message size that is too large
        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0xFF, 0xFF, 0xFF, // timestamp
            0xFF, 0xFF, 0xFF, // message length (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
            0x01, 0x00, 0x00, 0x00, // extended timestamp
        ]);

        let mut unpacker = ChunkReader::default();

        let err = unpacker.read_chunk(&mut buf).unwrap_err();
        match err {
            crate::error::RtmpError::ChunkRead(ChunkReadError::PartialChunkTooLarge(16777215)) => {}
            _ => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn test_reader_error_too_many_partial_chunks() {
        let mut buf = BytesMut::new();

        let mut unpacker = ChunkReader::default();

        for i in 0..4 {
            // Write another chunk with a different chunk stream id
            #[rustfmt::skip]
            buf.extend_from_slice(&[
                (i + 2), // chunk type 0 (partial), chunk stream id i
                0xFF, 0xFF, 0xFF, // timestamp
                0x00, 0x01, 0x00, // message length (max chunk size is set to 128)
                0x09, // message type id (video)
                0x00, 0x01, 0x00, 0x00, // message stream id
                0x01, 0x00, 0x00, 0x00, // extended timestamp
            ]);

            for i in 0..128 {
                (&mut buf).writer().write_u8(i as u8).unwrap();
            }

            // Read the chunk
            assert!(
                unpacker
                    .read_chunk(&mut buf)
                    .unwrap_or_else(|_| panic!("chunk failed {i}"))
                    .is_none()
            );
        }

        // Write another chunk with a different chunk stream id
        #[rustfmt::skip]
        buf.extend_from_slice(&[
            12, // chunk type 0, chunk stream id 6
            0xFF, 0xFF, 0xFF, // timestamp
            0x00, 0x01, 0x00, // message length (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
            0x01, 0x00, 0x00, 0x00, // extended timestamp
        ]);

        for i in 0..128 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let err = unpacker.read_chunk(&mut buf).unwrap_err();
        match err {
            crate::error::RtmpError::ChunkRead(ChunkReadError::TooManyPartialChunks) => {}
            _ => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn test_reader_error_too_many_chunk_headers() {
        let mut buf = BytesMut::new();

        let mut unpacker = ChunkReader::default();

        for i in 0..100 {
            // Write another chunk with a different chunk stream id
            #[rustfmt::skip]
            buf.extend_from_slice(&[
                (0 << 6), // chunk type 0 (partial), chunk stream id 0
                i,        // chunk id
                0xFF, 0xFF, 0xFF, // timestamp
                0x00, 0x00, 0x00, // message length (max chunk size is set to 128)
                0x09, // message type id (video)
                0x00, 0x01, 0x00, 0x00, // message stream id
                0x01, 0x00, 0x00, 0x00, // extended timestamp
            ]);

            // Read the chunk (should be a full chunk since the message length is 0)
            assert!(
                unpacker
                    .read_chunk(&mut buf)
                    .unwrap_or_else(|_| panic!("chunk failed {i}"))
                    .is_some()
            );
        }

        // Write another chunk with a different chunk stream id
        #[rustfmt::skip]
        buf.extend_from_slice(&[
            12, // chunk type 0, chunk stream id 6
            0xFF, 0xFF, 0xFF, // timestamp
            0x00, 0x00, 0x00, // message length (max chunk size is set to 128)
            0x09, // message type id (video)
            0x00, 0x01, 0x00, 0x00, // message stream id
            0x01, 0x00, 0x00, 0x00, // extended timestamp
        ]);

        let err = unpacker.read_chunk(&mut buf).unwrap_err();
        match err {
            crate::error::RtmpError::ChunkRead(ChunkReadError::TooManyPreviousChunkHeaders) => {}
            _ => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn test_reader_larger_chunk_size() {
        let mut buf = BytesMut::new();

        // Write a chunk that has a message size that is too large
        #[rustfmt::skip]
        buf.extend_from_slice(&[
            3, // chunk type 0, chunk stream id 3
            0x00, 0x00, 0xFF, // timestamp
            0x00, 0x0F, 0x00, // message length ()
            0x09, // message type id (video)
            0x01, 0x00, 0x00, 0x00, // message stream id
        ]);

        for i in 0..3840 {
            (&mut buf).writer().write_u8(i as u8).unwrap();
        }

        let mut unpacker = ChunkReader::default();
        unpacker.update_max_chunk_size(4096);

        let chunk = unpacker.read_chunk(&mut buf).expect("failed").expect("chunk");
        assert_eq!(chunk.basic_header.chunk_stream_id, 3);
        assert_eq!(chunk.message_header.timestamp, 255);
        assert_eq!(chunk.message_header.msg_length, 3840);
        assert_eq!(chunk.message_header.msg_type_id.0, 0x09);
        assert_eq!(chunk.message_header.msg_stream_id, 1); // little endian
        assert_eq!(chunk.payload.len(), 3840);

        for i in 0..3840 {
            assert_eq!(chunk.payload[i], i as u8);
        }
    }
}
