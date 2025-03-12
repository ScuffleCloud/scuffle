use std::io;

use bytes::Bytes;

mod enhanced;
mod legacy;

pub use enhanced::*;
pub use legacy::*;

use super::header::AudioTagHeader;

#[derive(Debug, Clone, PartialEq)]
pub enum AudioTagBody {
    Legacy(LegacyAudioTagBody),
    Enhanced(ExAudioTagBody),
}

impl AudioTagBody {
    /// Demux the audio tag body from the given reader.
    ///
    /// The reader will be entirely consumed.
    pub fn demux(header: &AudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        match header {
            AudioTagHeader::Legacy(header) => LegacyAudioTagBody::demux(header, reader).map(Self::Legacy),
            AudioTagHeader::Enhanced(header) => ExAudioTagBody::demux(header, reader).map(Self::Enhanced),
        }
    }
}
