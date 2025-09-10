//! FLV audio processing
//!
//! Use [`AudioData`] to demux audio data contained in an RTMP audio message.

use std::io;

use body::AudioTagBody;
use bytes::Bytes;
use header::AudioTagHeader;

use crate::error::FlvError;

pub mod body;
pub mod header;

/// FLV `AUDIODATA` tag
///
/// This is a container for legacy as well as enhanced audio data.
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.2.1
/// - Enhanced RTMP spec, page 19, Enhanced Audio
#[derive(Debug, Clone, PartialEq)]
pub struct AudioData {
    /// The header of the audio data.
    pub header: AudioTagHeader,
    /// The body of the audio data.
    pub body: AudioTagBody,
}

impl AudioData {
    /// Demux audio data from a given reader.
    ///
    /// This function will automatically determine whether the given data represents a legacy or enhanced audio data
    /// and demux it accordingly.
    ///
    /// Returns a new instance of [`AudioData`] if successful.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
        let header = AudioTagHeader::demux(reader)?;
        let body = AudioTagBody::demux(&header, reader)?;

        Ok(AudioData { header, body })
    }
}
