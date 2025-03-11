use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use scuffle_bytes_util::BytesCursorExt;

use crate::error::Error;

/// The FLV header
/// Whenever a FLV file is read these are the first 9 bytes of the file.
///
/// Defined by:
/// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV Header - Page 8)
/// - video_file_format_spec_v10_1.pdf (Annex E.2 - The FLV Header)
#[derive(Debug, Clone, PartialEq)]
pub struct FlvHeader {
    /// The version of the FLV file.
    pub version: u8,
    /// Whether the FLV file contains audio tags.
    pub is_audio_present: bool,
    /// Whether the FLV file contains video tags.
    pub is_video_present: bool,
    /// The extra data in the FLV header.
    ///
    /// Since the header provides a data offset, this is the remaining bytes after the DataOffset field
    /// to the end of the header.
    pub extra: Bytes,
}

impl FlvHeader {
    /// Demux the FLV header from the given reader.
    /// The reader will be returned in the position of the start of the data
    /// offset.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let start = reader.position() as usize;

        let signature = reader.read_u24::<BigEndian>()?;

        // 0 byte at the beginning because we are only reading 3 bytes not 4.
        if signature != u32::from_be_bytes([0, b'F', b'L', b'V']) {
            return Err(Error::InvalidSignature(signature));
        }

        let version = reader.read_u8()?;
        let flags = reader.read_u8()?;
        let is_audio_present = (flags & 0b00000100) != 0;
        let is_video_present = (flags & 0b00000001) != 0;

        let data_offset = reader.read_u32::<BigEndian>()?;
        let end = reader.position() as usize;
        let size = end - start;

        let remaining = (data_offset as usize)
            .checked_sub(size)
            .ok_or(Error::InvalidDataOffset(data_offset))?;

        let extra = reader.extract_bytes(remaining)?;

        Ok(FlvHeader {
            version,
            is_audio_present,
            is_video_present,
            extra,
        })
    }
}
