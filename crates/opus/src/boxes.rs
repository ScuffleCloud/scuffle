//! ISO base media file format boxes for OPUS.
//!
//! [Encapsulation of Opus in ISO Base Media File Format Version 0.6.8](https://www.opus-codec.org/docs/opus_in_isobmff.html)

use isobmff::boxes::AudioSampleEntry;
use isobmff::{BoxHeader, IsoBox, UnknownBox};
use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

/// Opus Sample Entry Format
///
/// Encapsulation of Opus in ISO Base Media File Format - 4.3.1
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"Opus")]
pub struct OpusSampleEntry<'a> {
    /// The header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The audio sample entry fields that this box inherits.
    pub sample_entry: AudioSampleEntry,
    /// Contains initializing information for the decoder.
    #[iso_box(nested_box)]
    pub dops: OpusSpecificBox<'a>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// Opus Specific Box
///
/// Encapsulation of Opus in ISO Base Media File Format - 4.3.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"dOps", skip_impl(deserialize_seed, serialize))]
pub struct OpusSpecificBox<'a> {
    /// The header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// Shall be set to 0.
    pub version: u8,
    /// Shall be set to the same value as the *Output Channel Count* field in the
    /// identification header defined in Ogg Opus.
    pub output_channel_count: u8,
    /// Indicates the number of the priming samples, that is, the number of samples at 48000 Hz
    /// to discard from the decoder output when starting playback.
    pub pre_skip: u16,
    /// Shall be set to the same value as the *Input Sample Rate* field in the
    /// identification header defined in Ogg Opus.
    pub input_sample_rate: u32,
    /// Shall be set to the same value as the *Output Gain* field in the identification
    /// header define in Ogg Opus.
    pub output_gain: i16,
    /// Shall be set to the same value as the *Channel Mapping Family* field in
    /// the identification header defined in Ogg Opus.
    pub channel_mapping_family: u8,
    /// The optional channel mapping table contained in this box.
    ///
    /// Only present if [`channel_mapping_family`](Self::channel_mapping_family) is not 0.
    pub channel_mapping_table: Option<ChannelMappingTable<'a>>,
}

// The IsoBox derive macro doesn't support conditional fields.
// That's why we have to implement the traits manually here.

impl<'a> DeserializeSeed<'a, BoxHeader> for OpusSpecificBox<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let version = u8::deserialize(&mut reader)?;
        if version != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "OpusSpecificBox version must be 0",
            ));
        }

        let output_channel_count = u8::deserialize(&mut reader)?;
        let pre_skip = u16::deserialize(&mut reader)?;
        let input_sample_rate = u32::deserialize(&mut reader)?;
        let output_gain = i16::deserialize(&mut reader)?;
        let channel_mapping_family = u8::deserialize(&mut reader)?;
        let channel_mapping_table = if channel_mapping_family != 0 {
            Some(ChannelMappingTable::deserialize_seed(&mut reader, output_channel_count)?)
        } else {
            None
        };

        Ok(Self {
            header: seed,
            version,
            output_channel_count,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping_family,
            channel_mapping_table,
        })
    }
}

impl Serialize for OpusSpecificBox<'_> {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.header.serialize(&mut writer)?;
        self.version.serialize(&mut writer)?;
        self.output_channel_count.serialize(&mut writer)?;
        self.pre_skip.serialize(&mut writer)?;
        self.input_sample_rate.serialize(&mut writer)?;
        self.output_gain.serialize(&mut writer)?;
        self.channel_mapping_family.serialize(&mut writer)?;
        if let Some(channel_mapping_table) = &self.channel_mapping_table {
            channel_mapping_table.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Channel Mapping Table
///
/// Encapsulation of Opus in ISO Base Media File Format - 4.3.2
#[derive(Debug)]
pub struct ChannelMappingTable<'a> {
    /// Shall be set to the same value as the *Stream Count* field in the identification
    /// header defined in Ogg Opus.
    pub stream_count: u8,
    /// Shall be set to the same value as the *Coupled Count* field in the identification
    /// header defined in Ogg Opus.
    pub coupled_count: u8,
    /// Shall be set to the same octet string as *Channel Mapping* field in the identification
    /// header defined in Ogg Opus.
    pub channel_mapping: BytesCow<'a>,
}

impl<'a> DeserializeSeed<'a, u8> for ChannelMappingTable<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        // seed is OutputChannelCount
        let stream_count = u8::deserialize(&mut reader)?;
        let coupled_count = u8::deserialize(&mut reader)?;
        let channel_mapping = reader.try_read(seed as usize)?;

        Ok(Self {
            stream_count,
            coupled_count,
            channel_mapping,
        })
    }
}

impl Serialize for ChannelMappingTable<'_> {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.stream_count.serialize(&mut writer)?;
        self.coupled_count.serialize(&mut writer)?;
        self.channel_mapping.serialize(&mut writer)?;

        Ok(())
    }
}
