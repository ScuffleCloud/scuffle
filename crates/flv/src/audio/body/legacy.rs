use std::io;

use byteorder::ReadBytesExt;
use bytes::Bytes;
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::aac;
use crate::audio::header::{LegacyAudioTagHeader, SoundFormat};

/// The legacy FLV `AudioTagBody`.
///
/// This is the container for the audio data body.
///
/// Defined by:
/// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Audio tags)
/// - video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA)
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyAudioTagBody {
    /// AAC Audio Packet
    Aac(aac::AacAudioData),
    /// Some other audio format we don't know how to parse
    Other { sound_data: Bytes },
}

impl LegacyAudioTagBody {
    /// Demux the audio tag body from the given reader.
    ///
    /// The reader will be entirely consumed.
    pub fn demux(header: &LegacyAudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        match header.sound_format {
            SoundFormat::Aac => {
                // For some reason the spec adds a specific byte before the AAC data.
                // This byte is the AAC packet type.
                let aac_packet_type = aac::AacPacketType::from(reader.read_u8()?);
                Ok(Self::Aac(aac::AacAudioData::new(aac_packet_type, reader.extract_remaining())))
            }
            _ => Ok(Self::Other {
                sound_data: reader.extract_remaining(),
            }),
        }
    }
}
