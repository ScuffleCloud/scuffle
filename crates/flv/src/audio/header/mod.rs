use std::io::{self, Seek};

use byteorder::ReadBytesExt;
use bytes::Bytes;

use crate::error::Error;

mod enhanced;
mod legacy;

pub use enhanced::*;
pub use legacy::*;

/// This is a helper enum to represent the different types of audio tag headers.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioTagHeader {
    Legacy(LegacyAudioTagHeader),
    Enhanced(ExAudioTagHeader),
}

impl AudioTagHeader {
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let byte = reader.read_u8()?;
        let sound_format = SoundFormat::from((byte & 0b1111_00_0_0) >> 4);
        // Seek back one byte because we need the other half of the byte again
        reader.seek_relative(-1)?;

        if sound_format == SoundFormat::ExHeader {
            ExAudioTagHeader::demux(reader).map(AudioTagHeader::Enhanced)
        } else {
            LegacyAudioTagHeader::demux(reader).map(AudioTagHeader::Legacy)
        }
    }
}
