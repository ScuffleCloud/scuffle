//! Legacy audio tag body
//!
//! Types and functions defined by the legacy FLV spec, Annex E.4.2.1.

use std::io;

use byteorder::ReadBytesExt;
use bytes::Bytes;
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::header::legacy::{LegacyAudioTagHeader, SoundFormat};

pub mod aac;

/// The legacy FLV `AudioTagBody`.
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.2.1
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyAudioTagBody {
    /// AAC Audio Packet
    Aac(aac::AacAudioData),
    /// Any other audio format
    Other {
        /// The sound data
        sound_data: Bytes,
    },
}

impl LegacyAudioTagBody {
    /// Demux the audio tag body from the given reader.
    ///
    /// The reader will be consumed entirely.
    pub fn demux(header: &LegacyAudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        match header.sound_format {
            SoundFormat::Aac => {
                let aac_packet_type = aac::AacPacketType::from(reader.read_u8()?);
                Ok(Self::Aac(aac::AacAudioData::new(aac_packet_type, reader.extract_remaining())))
            }
            _ => Ok(Self::Other {
                sound_data: reader.extract_remaining(),
            }),
        }
    }
}
