use std::io;

use body::AudioTagBody;
use bytes::Bytes;
use header::AudioTagHeader;

use crate::error::Error;

pub mod aac;
pub mod body;
pub mod header;

/// FLV `AUDIODATA` tag
///
/// This is the container for the audio data.
///
/// Defined by:
/// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Audio tags)
/// - video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA)
#[derive(Debug, Clone, PartialEq)]
pub struct AudioData {
    /// The header of the audio data.
    pub header: AudioTagHeader,
    /// The body of the audio data.
    pub body: AudioTagBody,
}

impl AudioData {
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let header = AudioTagHeader::demux(reader)?;
        let body = AudioTagBody::demux(&header, reader)?;

        Ok(AudioData { header, body })
    }
}
