//! A crate for decoding AAC audio headers.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

use std::io;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scuffle_bytes_util::BitReader;

/// A Partial Audio Specific Config
/// ISO/IEC 14496-3:2019(E) - 1.6
///
/// This struct does not represent the full AudioSpecificConfig, it only
/// represents the top few fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct PartialAudioSpecificConfig {
    /// Audio Object Type
    pub audio_object_type: AudioObjectType,
    /// Sampling Frequency
    pub sampling_frequency: u32,
    /// Channel Configuration
    pub channel_configuration: u8,
}

/// SBR Audio Object Type
/// ISO/IEC 14496-3:2019(E) - 1.5.1.2.6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum AudioObjectType {
    /// AAC main
    AacMain,
    /// AAC LC
    AacLowComplexity,
    /// Any other object type
    Unknown(u16),
}

impl AudioObjectType {
    /// Converts an AudioObjectType to a u16
    pub const fn as_u16(&self) -> u16 {
        match self {
            AudioObjectType::AacMain => 1,
            AudioObjectType::AacLowComplexity => 2,
            AudioObjectType::Unknown(value) => *value,
        }
    }

    /// Converts a u16 to an AudioObjectType
    pub const fn from_u16(value: u16) -> Self {
        match value {
            1 => AudioObjectType::AacMain,
            2 => AudioObjectType::AacLowComplexity,
            _ => AudioObjectType::Unknown(value),
        }
    }
}

impl From<u16> for AudioObjectType {
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl From<AudioObjectType> for u16 {
    fn from(value: AudioObjectType) -> Self {
        value.as_u16()
    }
}

/// Sampling Frequency Index
///
/// The purpose of the FrequencyIndex is to encode commonly used frequencies in
/// 4 bits to save space. These are the set of commonly used frequencies defined
/// in the specification.
///
/// ISO/IEC 14496-3:2019(E) - 1.6.2.4 (Table 1.22)
#[derive(FromPrimitive, Debug, Clone, PartialEq, Copy, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[must_use]
pub enum SampleFrequencyIndex {
    /// 96000 Hz
    Freq96000 = 0x0,
    /// 88200 Hz
    Freq88200 = 0x1,
    /// 64000 Hz
    Freq64000 = 0x2,
    /// 48000 Hz
    Freq48000 = 0x3,
    /// 44100 Hz
    Freq44100 = 0x4,
    /// 32000 Hz
    Freq32000 = 0x5,
    /// 24000 Hz
    Freq24000 = 0x6,
    /// 22050 Hz
    Freq22050 = 0x7,
    /// 16000 Hz
    Freq16000 = 0x8,
    /// 12000 Hz
    Freq12000 = 0x9,
    /// 11025 Hz
    Freq11025 = 0xA,
    /// 8000 Hz
    Freq8000 = 0xB,
    /// 7350 Hz
    Freq7350 = 0xC,
    /// Reserved
    FreqReserved = 0xD,
    /// Reserved
    FreqReserved2 = 0xE,
    /// Escape (Meaning the frequency is not in the table, and we need to read
    /// an additional 24 bits to get the frequency)
    FreqEscape = 0xF,
}

impl SampleFrequencyIndex {
    /// Convert the SampleFrequencyIndex to the actual frequency in Hz
    pub const fn to_freq(&self) -> Option<u32> {
        match self {
            SampleFrequencyIndex::Freq96000 => Some(96000),
            SampleFrequencyIndex::Freq88200 => Some(88200),
            SampleFrequencyIndex::Freq64000 => Some(64000),
            SampleFrequencyIndex::Freq48000 => Some(48000),
            SampleFrequencyIndex::Freq44100 => Some(44100),
            SampleFrequencyIndex::Freq32000 => Some(32000),
            SampleFrequencyIndex::Freq24000 => Some(24000),
            SampleFrequencyIndex::Freq22050 => Some(22050),
            SampleFrequencyIndex::Freq16000 => Some(16000),
            SampleFrequencyIndex::Freq12000 => Some(12000),
            SampleFrequencyIndex::Freq11025 => Some(11025),
            SampleFrequencyIndex::Freq8000 => Some(8000),
            SampleFrequencyIndex::Freq7350 => Some(7350),
            SampleFrequencyIndex::FreqReserved => None,
            SampleFrequencyIndex::FreqReserved2 => None,
            SampleFrequencyIndex::FreqEscape => None,
        }
    }
}

impl PartialAudioSpecificConfig {
    /// Parse the Audio Specific Config from given bytes
    /// The implementation is based on ISO/IEC 14496-3:2019(E) - 1.6.2.1 (Table
    /// 1.19) This does not parse the entire AAC Data, it only parses the
    /// top few fields.
    /// - Audio Object Type
    /// - Sampling Frequency
    /// - Channel Configuration
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut bitreader = BitReader::new_from_slice(data);

        // GetAudioObjectType() # ISO/IEC 14496-3:2019(E) - 1.6.2.1 (Table 1.20)
        let mut audio_object_type = bitreader.read_bits(5)? as u16;
        if audio_object_type == 31 {
            audio_object_type = 32 + bitreader.read_bits(6)? as u16;
        }

        // The table calls for us to read a 4-bit value. If the value is type FreqEscape
        // (0xF), we need to read 24 bits to get the sampling frequency.
        let sampling_frequency_index = SampleFrequencyIndex::from_u8(bitreader.read_bits(4)? as u8)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid sampling frequency index"))?;

        let sampling_frequency = match sampling_frequency_index {
            // Uses the extended sampling frequency to represent the freq as a non-common value
            SampleFrequencyIndex::FreqEscape => bitreader.read_bits(24)? as u32,
            _ => sampling_frequency_index
                .to_freq()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid sampling frequency index"))?,
        };

        // 4 Bits to get the channel configuration
        let channel_configuration = bitreader.read_bits(4)? as u8;

        Ok(Self {
            audio_object_type: audio_object_type.into(),
            sampling_frequency,
            channel_configuration,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_aac_config_parse() {
        let data = [
            0x12, 0x10, 0x56, 0xe5, 0x00, 0x2d, 0x96, 0x01, 0x80, 0x80, 0x05, 0x00, 0x00, 0x00, 0x00,
        ];

        let config = PartialAudioSpecificConfig::parse(&data).unwrap();
        assert_eq!(config.audio_object_type, AudioObjectType::AacLowComplexity);
        assert_eq!(config.sampling_frequency, 44100);
        assert_eq!(config.channel_configuration, 2);
    }

    #[test]
    fn test_idx_to_freq() {
        let cases = [
            (SampleFrequencyIndex::FreqEscape, None),
            (SampleFrequencyIndex::FreqReserved2, None),
            (SampleFrequencyIndex::FreqReserved, None),
            (SampleFrequencyIndex::Freq7350, Some(7350)),
            (SampleFrequencyIndex::Freq8000, Some(8000)),
            (SampleFrequencyIndex::Freq11025, Some(11025)),
            (SampleFrequencyIndex::Freq12000, Some(12000)),
            (SampleFrequencyIndex::Freq16000, Some(16000)),
            (SampleFrequencyIndex::Freq22050, Some(22050)),
            (SampleFrequencyIndex::Freq24000, Some(24000)),
            (SampleFrequencyIndex::Freq32000, Some(32000)),
            (SampleFrequencyIndex::Freq44100, Some(44100)),
            (SampleFrequencyIndex::Freq48000, Some(48000)),
            (SampleFrequencyIndex::Freq64000, Some(64000)),
            (SampleFrequencyIndex::Freq88200, Some(88200)),
            (SampleFrequencyIndex::Freq96000, Some(96000)),
        ];

        for (idx, freq) in cases {
            assert_eq!(freq, idx.to_freq(), "Expected frequency for {:?}", idx);
        }
    }
}
