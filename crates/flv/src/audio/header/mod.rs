//! FLV audio tag headers.

use std::io::{self, Seek};

use byteorder::ReadBytesExt;
use bytes::Bytes;
use enhanced::ExAudioTagHeader;
use legacy::{LegacyAudioTagHeader, SoundFormat};

use crate::error::FlvError;

pub mod enhanced;
pub mod legacy;

/// FLV `AudioTagHeader`
///
/// This only describes the audio tag header, see [`AudioData`](super::AudioData) for the full audio data container.
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.2.1
/// - Enhanced RTMP spec, page 19, Enhanced Audio
#[derive(Debug, Clone, PartialEq)]
pub enum AudioTagHeader {
    /// Legacy audio tag header.
    Legacy(LegacyAudioTagHeader),
    /// Enhanced audio tag header.
    Enhanced(ExAudioTagHeader),
}

impl AudioTagHeader {
    /// Demux the audio tag header from the given reader.
    ///
    /// If you want to demux the full audio data tag, use [`AudioData::demux`](super::AudioData::demux) instead.
    /// This function will automatically determine whether the given data represents a legacy or an enhanced audio tag header
    /// and demux it accordingly.
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
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
