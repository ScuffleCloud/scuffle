//! FLV audio tag bodies.

use std::io;

use bytes::Bytes;
use enhanced::ExAudioTagBody;
use legacy::LegacyAudioTagBody;

use super::header::AudioTagHeader;

pub mod enhanced;
pub mod legacy;

/// FLV `AudioTagBody`
///
/// This only describes the audio tag body, see [`AudioData`](super::AudioData) for the full audio data container.
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.2.1
/// - Enhanced RTMP spec, page 19, Enhanced Audio
#[derive(Debug, Clone, PartialEq)]
pub enum AudioTagBody {
    /// Legacy audio tag body.
    Legacy(LegacyAudioTagBody),
    /// Enhanced audio tag body.
    Enhanced(ExAudioTagBody),
}

impl AudioTagBody {
    /// Demux the audio tag body from the given reader.
    ///
    /// If you want to demux the full audio data tag, use [`AudioData::demux`](super::AudioData::demux) instead.
    /// This function will automatically determine whether the given data represents a legacy or an enhanced audio tag body
    /// and demux it accordingly.
    ///
    /// The reader will be entirely consumed.
    pub fn demux(header: &AudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        match header {
            AudioTagHeader::Legacy(header) => LegacyAudioTagBody::demux(header, reader).map(Self::Legacy),
            AudioTagHeader::Enhanced(header) => ExAudioTagBody::demux(header, reader).map(Self::Enhanced),
        }
    }
}
