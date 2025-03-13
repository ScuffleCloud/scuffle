use std::io;

use byteorder::ReadBytesExt;
use bytes::Bytes;
use nutype_enum::nutype_enum;

use crate::error::Error;

nutype_enum! {
    /// FLV Sound Format
    ///
    /// Denotes the type of the underlying data packet
    ///
    /// Defined by:
    /// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Audio tags)
    /// - video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA)
    pub enum SoundFormat(u8) {
        /// Linear PCM, platform endian
        LinearPcmPlatformEndian = 0,
        /// ADPCM
        Adpcm = 1,
        /// MP3
        Mp3 = 2,
        /// Linear PCM, little endian
        LinearPcmLittleEndian = 3,
        /// Nellymoser 16Khz Mono
        Nellymoser16KhzMono = 4,
        /// Nellymoser 8Khz Mono
        Nellymoser8KhzMono = 5,
        /// Nellymoser
        Nellymoser = 6,
        /// G.711 A-Law logarithmic PCM
        G711ALaw = 7,
        /// G.711 Mu-Law logarithmic PCM
        G711MuLaw = 8,
        /// The `ExAudioTagHeader` is present
        ///
        /// Defined by: Enhanced RTMP v2 (Enhanced Audio section)
        ExHeader = 9,
        /// AAC
        Aac = 10,
        /// Speex
        Speex = 11,
        /// Mp3 8Khz
        Mp38Khz = 14,
        /// Device specific sound
        DeviceSpecificSound = 15,
    }
}

nutype_enum! {
    /// FLV Sound Rate
    ///
    /// Denotes the sampling rate of the audio data.
    ///
    /// Defined by:
    /// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Audio tags)
    /// - video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA)
    pub enum SoundRate(u8) {
        /// 5.5 KHz
        Hz5500 = 0,
        /// 11 KHz
        Hz11000 = 1,
        /// 22 KHz
        Hz22000 = 2,
        /// 44 KHz
        Hz44000 = 3,
    }
}

nutype_enum! {
    /// FLV Sound Size
    ///
    /// Denotes the size of each sample in the audio data.
    ///
    /// Defined by:
    /// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Audio tags)
    /// - video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA)
    pub enum SoundSize(u8) {
        /// 8 bit
        Bit8 = 0,
        /// 16 bit
        Bit16 = 1,
    }
}

nutype_enum! {
    /// FLV Sound Type
    ///
    /// Denotes the number of channels in the audio data.
    ///
    /// Defined by:
    /// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Audio tags)
    /// - video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA)
    pub enum SoundType(u8) {
        /// Mono
        Mono = 0,
        /// Stereo
        Stereo = 1,
    }
}

/// The legacy FLV `AudioTagHeader` as defined by the original spec.
///
/// Defined by video_file_format_spec_v10_1.pdf (Annex E.4.2.1 - AUDIODATA).
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAudioTagHeader {
    /// The sound format of the audio data. (4 bits)
    pub sound_format: SoundFormat,
    /// The sound rate of the audio data. (2 bits)
    pub sound_rate: SoundRate,
    /// The sound size of the audio data. (1 bit)
    pub sound_size: SoundSize,
    /// The sound type of the audio data. (1 bit)
    pub sound_type: SoundType,
}

impl LegacyAudioTagHeader {
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let byte = reader.read_u8()?;

        // SoundFormat is the first 4 bits of the byte
        let sound_format = SoundFormat::from(byte >> 4); // 0b1111_00_0_0
        // SoundRate is the next 2 bits of the byte
        let sound_rate = SoundRate::from((byte & 0b0000_11_0_0) >> 2);
        // SoundSize is the next bit of the byte
        let sound_size = SoundSize::from((byte & 0b0000_00_1_0) >> 1);
        // SoundType is the last bit of the byte
        let sound_type = SoundType::from(byte & 0b0000_00_0_1);

        Ok(Self {
            sound_format,
            sound_rate,
            sound_size,
            sound_type,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_sound_format() {
        let cases = [
            (
                0x00,
                SoundFormat::LinearPcmPlatformEndian,
                "SoundFormat::LinearPcmPlatformEndian",
            ),
            (0x01, SoundFormat::Adpcm, "SoundFormat::Adpcm"),
            (0x02, SoundFormat::Mp3, "SoundFormat::Mp3"),
            (0x03, SoundFormat::LinearPcmLittleEndian, "SoundFormat::LinearPcmLittleEndian"),
            (0x04, SoundFormat::Nellymoser16KhzMono, "SoundFormat::Nellymoser16KhzMono"),
            (0x05, SoundFormat::Nellymoser8KhzMono, "SoundFormat::Nellymoser8KhzMono"),
            (0x06, SoundFormat::Nellymoser, "SoundFormat::Nellymoser"),
            (0x07, SoundFormat::G711ALaw, "SoundFormat::G711ALaw"),
            (0x08, SoundFormat::G711MuLaw, "SoundFormat::G711MuLaw"),
            (0x0A, SoundFormat::Aac, "SoundFormat::Aac"),
            (0x0B, SoundFormat::Speex, "SoundFormat::Speex"),
            (0x0E, SoundFormat::Mp38Khz, "SoundFormat::Mp38Khz"),
            (0x0F, SoundFormat::DeviceSpecificSound, "SoundFormat::DeviceSpecificSound"),
        ];

        for (value, expected, name) in cases {
            let sound_format = SoundFormat::from(value);
            assert_eq!(sound_format, expected);
            assert_eq!(format!("{:?}", sound_format), name);
        }
    }

    #[test]
    fn test_sound_rate() {
        let cases = [
            (0x00, SoundRate::Hz5500, "SoundRate::Hz5500"),
            (0x01, SoundRate::Hz11000, "SoundRate::Hz11000"),
            (0x02, SoundRate::Hz22000, "SoundRate::Hz22000"),
            (0x03, SoundRate::Hz44000, "SoundRate::Hz44000"),
        ];

        for (value, expected, name) in cases {
            let sound_rate = SoundRate::from(value);
            assert_eq!(sound_rate, expected);
            assert_eq!(format!("{:?}", sound_rate), name);
        }
    }

    #[test]
    fn test_sound_size() {
        let cases = [
            (0x00, SoundSize::Bit8, "SoundSize::Bit8"),
            (0x01, SoundSize::Bit16, "SoundSize::Bit16"),
        ];

        for (value, expected, name) in cases {
            let sound_size = SoundSize::from(value);
            assert_eq!(sound_size, expected);
            assert_eq!(format!("{:?}", sound_size), name);
        }
    }

    #[test]
    fn test_sound_type() {
        let cases = [
            (0x00, SoundType::Mono, "SoundType::Mono"),
            (0x01, SoundType::Stereo, "SoundType::Stereo"),
        ];

        for (value, expected, name) in cases {
            let sound_type = SoundType::from(value);
            assert_eq!(sound_type, expected);
            assert_eq!(format!("{:?}", sound_type), name);
        }
    }
}
