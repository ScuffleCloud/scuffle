use std::io;

use fixed::types::extra::{U8, U16};
use fixed::{FixedI16, FixedU32};
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};
use scuffle_bytes_util::{BitWriter, BytesCow};

use super::SampleEntry;
use crate::utils::pad_to_u64;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Sound media header
///
/// ISO/IEC 14496-12 - 12.2.2
#[derive(IsoBox, Debug, PartialEq, Eq, Default)]
#[iso_box(box_type = b"smhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SoundMediaHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A number that places mono audio tracks in a stereo space; 0 is centre (the
    /// normal value); full left is -1.0 and full right is 1.0.
    pub balance: FixedI16<U8>,
    /// Reserved 16 bits, must be set to 0.
    pub reserved: u16,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SoundMediaHeaderBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let balance = FixedI16::from_bits(i16::deserialize(&mut reader)?);
        let reserved = u16::deserialize(&mut reader)?;

        Ok(Self {
            full_header,
            balance,
            reserved,
        })
    }
}

impl Serialize for SoundMediaHeaderBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.balance.to_bits().serialize(&mut writer)?;
        self.reserved.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for SoundMediaHeaderBox {
    fn size(&self) -> usize {
        Self::add_header_size(self.full_header.size() + 2 + 2)
    }
}

/// Audio sample entry
///
/// ISO/IEC 14496-12 - 12.2.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - [`chnl`](ChannelLayout)
/// - [`dmix`](DownMixInstructions)
/// - `udc1` (`DRCCoefficientsBasic`, defined in ISO/IEC 23003-4)
/// - `udi1` (`DRCInstructionsBasic`, defined in ISO/IEC 23003-4)
/// - `udc2` (`DRCCoefficientsUniDRC`, defined in ISO/IEC 23003-4)
/// - `udi2` (`DRCInstructionsUniDRC`, defined in ISO/IEC 23003-4)
/// - `udex` (`UniDrcConfigExtension`, defined in ISO/IEC 23003-4)
/// - [`srat`](SamplingRateBox)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct AudioSampleEntry {
    /// The sample entry that this entry inherits from.
    pub sample_entry: SampleEntry,
    /// Reserved 64 bits, must be set to 0.
    pub reserved1: u64,
    /// The number of channels.
    ///
    /// - 0: inapplicable/unknown
    /// - 1: mono
    /// - 2: stereo (left/right)
    /// - all other values: the codec configuration should identify the channel assignment.
    pub channelcount: u16,
    /// In bits, and takes the default value of 16.
    pub samplesize: u16,
    /// Pre-defined 16 bit value, must be set to 0.
    pub pre_defined: u16,
    /// Reserved 16 bits, must be set to 0.
    pub reserved2: u16,
    /// When a [`SamplingRateBox`] is absent is the sampling rate; when a [`SamplingRateBox`] is present,
    /// is a suitable integer multiple or division of the actual sampling rate.
    pub samplerate: FixedU32<U16>,
}

impl AudioSampleEntry {
    /// Creates a new [`AudioSampleEntry`] with the given parameters.
    pub fn new(sample_entry: SampleEntry, channelcount: u16, samplesize: u16, samplerate: FixedU32<U16>) -> Self {
        Self {
            sample_entry,
            reserved1: 0,
            channelcount,
            samplesize,
            pre_defined: 0,
            reserved2: 0,
            samplerate,
        }
    }
}

impl<'a> Deserialize<'a> for AudioSampleEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let sample_entry = SampleEntry::deserialize(&mut reader)?;
        let reserved1 = u64::deserialize(&mut reader)?;
        let channelcount = u16::deserialize(&mut reader)?;
        let samplesize = u16::deserialize(&mut reader)?;
        let pre_defined = u16::deserialize(&mut reader)?;
        let reserved2 = u16::deserialize(&mut reader)?;
        let samplerate = FixedU32::from_bits(u32::deserialize(&mut reader)?);

        Ok(Self {
            sample_entry,
            reserved1,
            channelcount,
            samplesize,
            pre_defined,
            reserved2,
            samplerate,
        })
    }
}

impl Serialize for AudioSampleEntry {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(&mut writer)?;
        self.reserved1.serialize(&mut writer)?;
        self.channelcount.serialize(&mut writer)?;
        self.samplesize.serialize(&mut writer)?;
        self.pre_defined.serialize(&mut writer)?;
        self.reserved2.serialize(&mut writer)?;
        self.samplerate.to_bits().serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for AudioSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
            + 8 // reserved1
            + 2 // channelcount
            + 2 // samplesize
            + 2 // pre_defined
            + 2 // reserved2
            + 4 // samplerate
    }
}

/// Sampling rate box
///
/// ISO/IEC 14496-12 - 12.2.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"srat", crate_path = crate)]
pub struct SamplingRateBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The actual sampling rate of the audio media in samples/second, expressed as a 32-bit integer.
    pub sampling_rate: u32,
}

/// Audio sample entry version 1
///
/// ISO/IEC 14496-12 - 12.2.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - [`srat`](SamplingRateBox)
/// - [`chnl`](ChannelLayout)
/// - [`dmix`](DownMixInstructions)
/// - `udc1` (`DRCCoefficientsBasic`, defined in ISO/IEC 23003-4)
/// - `udi1` (`DRCInstructionsBasic`, defined in ISO/IEC 23003-4)
/// - `udc2` (`DRCCoefficientsUniDRC`, defined in ISO/IEC 23003-4)
/// - `udi2` (`DRCInstructionsUniDRC`, defined in ISO/IEC 23003-4)
/// - `udex` (`UniDrcConfigExtension`, defined in ISO/IEC 23003-4)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct AudioSampleEntryV1 {
    /// The sample entry that this entry inherits from.
    pub sample_entry: SampleEntry,
    /// Shall be 1.
    pub entry_version: u16,
    /// Reserved 48 bits, must be set to 0.
    pub reserved1: [u16; 3],
    /// The number of channels.
    ///
    /// - 0: inapplicable/unknown
    /// - 1: mono
    /// - 2: stereo (left/right)
    /// - all other values: the codec configuration should identify the channel assignment.
    pub channelcount: u16,
    /// In bits, and takes the default value of 16.
    pub samplesize: u16,
    /// Pre-defined 16 bit value, must be set to 0.
    pub pre_defined: u16,
    /// Reserved 16 bits, must be set to 0.
    pub reserved2: u16,
    /// When a [`SamplingRateBox`] is absent is the sampling rate; when a [`SamplingRateBox`] is present,
    /// is a suitable integer multiple or division of the actual sampling rate.
    pub samplerate: FixedU32<U16>,
}

impl<'a> Deserialize<'a> for AudioSampleEntryV1 {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let sample_entry = SampleEntry::deserialize(&mut reader)?;
        let entry_version = u16::deserialize(&mut reader)?;
        let reserved1 = <[u16; 3]>::deserialize(&mut reader)?;
        let channelcount = u16::deserialize(&mut reader)?;
        let samplesize = u16::deserialize(&mut reader)?;
        let pre_defined = u16::deserialize(&mut reader)?;
        let reserved2 = u16::deserialize(&mut reader)?;
        let samplerate = FixedU32::from_bits(u32::deserialize(&mut reader)?);

        Ok(Self {
            sample_entry,
            entry_version,
            reserved1,
            channelcount,
            samplesize,
            pre_defined,
            reserved2,
            samplerate,
        })
    }
}

impl Serialize for AudioSampleEntryV1 {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(&mut writer)?;
        self.entry_version.serialize(&mut writer)?;
        self.reserved1.serialize(&mut writer)?;
        self.channelcount.serialize(&mut writer)?;
        self.samplesize.serialize(&mut writer)?;
        self.pre_defined.serialize(&mut writer)?;
        self.reserved2.serialize(&mut writer)?;
        self.samplerate.to_bits().serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for AudioSampleEntryV1 {
    fn size(&self) -> usize {
        self.sample_entry.size()
            + 2 // entry_version
            + 2 * 3 // reserved1
            + 2 // channelcount
            + 2 // samplesize
            + 2 // pre_defined
            + 2 // reserved2
            + 4 // samplerate
    }
}

/// Channel layout
///
/// ISO/IEC 14496-12 - 12.2.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"chnl", crate_path = crate)]
pub struct ChannelLayout<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The channel layout data. (not further implemented)
    pub data: BytesCow<'a>,
}

/// Down mix instructions
///
/// ISO/IEC 14496-12 - 12.2.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"dmix", crate_path = crate)]
pub struct DownMixInstructions<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The down mix instructions data. (not further implemented)
    pub data: BytesCow<'a>,
}

/// Audio stream loudness base box
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(Debug, PartialEq, Eq)]
pub struct LoudnessBaseBox {
    /// The type of audio scene described by the loudness information. It shall take a
    /// value of zero unless other types are supported by the loudness processing. For defined values refer
    /// to the corresponding `loudnessInfoType` specification in ISO/IEC 23008-3.
    pub loudness_info_type: Option<u8>,
    /// Count of loudness bases in the box.
    pub loudness_base_count: u8,
    /// `mae_group_ID` or `mae_group_preset_ID` depending on the value of `loudness_info_type`.
    ///
    /// - `mae_group_ID` is a unique identifier for a group of metadata elements as specified in ISO/IEC 23008-3.
    /// - `mae_group_preset_ID` is a unique identifier for a group preset as specified in ISO/IEC 23008-3.
    pub mae_group_id: Option<u8>,
    /// The loudness bases contained in this box.
    pub loudness_bases: Vec<LoudnessBase>,
}

impl<'a> DeserializeSeed<'a, &FullBoxHeader> for LoudnessBaseBox {
    fn deserialize_seed<R>(mut reader: R, seed: &FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let (loudness_info_type, loudness_base_count, mae_group_id) = if seed.version >= 2 {
            let byte = u8::deserialize(&mut reader)?;
            let loudness_info_type = (byte >> 6) & 0b11;
            let loudness_base_count = byte & 0b0011_1111;
            let mae_group_id = if loudness_info_type == 1 || loudness_info_type == 2 {
                Some(u8::deserialize(&mut reader)? & 0b0111_1111)
            } else if loudness_info_type == 3 {
                Some(u8::deserialize(&mut reader)? & 0b0001_1111)
            } else {
                None
            };
            (Some(loudness_info_type), loudness_base_count, mae_group_id)
        } else if seed.version == 1 {
            let byte = u8::deserialize(&mut reader)?;
            (None, byte & 0b0011_1111, None)
        } else {
            (None, 1, None)
        };

        let mut loudness_bases = Vec::with_capacity(loudness_base_count as usize);
        for _ in 0..loudness_base_count {
            loudness_bases.push(LoudnessBase::deserialize_seed(&mut reader, seed)?);
        }

        Ok(Self {
            loudness_info_type,
            loudness_base_count,
            mae_group_id,
            loudness_bases,
        })
    }
}

impl LoudnessBaseBox {
    fn serialize<W>(&self, writer: W, version: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if version >= 2 {
            let loudness_info_type = self
                .loudness_info_type
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "loudness_info_type is required"))?;
            bit_writer.write_bits(loudness_info_type as u64, 2)?;
            bit_writer.write_bits(self.loudness_base_count as u64, 6)?;

            if loudness_info_type == 1 || loudness_info_type == 2 {
                bit_writer.write_bit(false)?;
                bit_writer.write_bits(
                    self.mae_group_id
                        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "mae_group_ID is required"))?
                        as u64,
                    7,
                )?;
            } else if loudness_info_type == 3 {
                bit_writer.write_bits(0, 3)?;
                bit_writer.write_bits(
                    self.mae_group_id
                        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "mae_group_preset_ID is required"))?
                        as u64,
                    5,
                )?;
            }
        } else if version == 1 {
            bit_writer.write_bits(0, 2)?;
            bit_writer.write_bits(self.loudness_base_count as u64, 6)?;
        }

        for loudness_base in &self.loudness_bases {
            loudness_base.serialize(&mut bit_writer, version)?;
        }

        Ok(())
    }
}

impl LoudnessBaseBox {
    /// Returns the size of the box in bytes, depending on the version.
    pub fn size(&self, version: u8) -> usize {
        let mut size = 0;

        if version >= 2 {
            size += 1; // loudness_base_count + reserved
            if self.loudness_info_type.is_some_and(|t| t == 1 || t == 2 || t == 3) {
                size += 1; // mae_group_ID or mae_group_preset_ID + reserved
            }
        } else if version == 1 {
            size += 1; // loudness_base_count + reserved
        }

        size += self.loudness_bases.size();

        size
    }
}

/// Loudness base in [`LoudnessBaseBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct LoudnessBase {
    /// When zero, declares the characteristics without applying EQ. If non-zero, this box declares
    /// the loudness after applying the EQ with the matching `EQ_set_ID` and shall match a value in exactly
    /// one box in the `UniDrcConfigExtension` of this track.
    pub eq_set_id: Option<u8>,
    /// When zero, declares the loudness characteristics of the layout without downmix. If non-zero,
    /// this box declares the loudness after applying the downmix with the matching `downmix_ID` and
    /// shall match a value in exactly one box in the sample entry of this track.
    pub downmix_id: u8,
    /// When zero, declares the characteristics without applying a DRC. If non-zero, this box
    /// declares the loudness after applying the DRC with the matching DRC_set_ID and shall match a
    /// value in exactly one box in the sample entry of this track.
    pub drc_set_id: u8,
    /// A value for the sample peak level as defined in ISO/IEC 23003-4; all other values are reserved.
    pub bs_sample_peak_level: i16,
    /// A value for the true peak level as defined in ISO/IEC 23003-4; all other values are reserved.
    pub bs_true_peak_level: i16,
    /// An index for the measurement system as defined in ISO/IEC 23003-4; all other values are reserved.
    pub measurement_system_for_tp: u8,
    /// - 0: Reliability is unknown.
    /// - 1: Value is reported/imported but unverified.
    /// - 2: Value is a 'not to exceed' ceiling.
    /// - 3: Value is measured and accurate.
    /// - 4: All other values are reserved.
    pub reliability_for_tp: u8,
    /// The number of measurements in the [`measurements`](Self::measurements) vec.
    pub measurement_count: u8,
    /// The measurements.
    pub measurements: Vec<LoudnessBaseMeasurement>,
}

impl<'a> DeserializeSeed<'a, &FullBoxHeader> for LoudnessBase {
    fn deserialize_seed<R>(mut reader: R, seed: &FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let eq_set_id = if seed.version >= 1 {
            Some(u8::deserialize(&mut reader)? & 0b0011_1111)
        } else {
            None
        };

        let bytes = u16::deserialize(&mut reader)?;
        let downmix_id = ((bytes >> 6) & 0b00_0111_1111) as u8;
        let drc_set_id = (bytes & 0b11_1111) as u8;

        let bytes = u32::deserialize(&mut reader)?;
        let bs_sample_peak_level = (bytes >> 20) as i16;
        let bs_true_peak_level = ((bytes >> 8) & 0b1111_1111_1111) as i16;
        let measurement_system_for_tp = ((bytes >> 4) & 0b1111) as u8;
        let reliability_for_tp = (bytes & 0b1111) as u8;

        let measurement_count = u8::deserialize(&mut reader)?;
        let mut measurements = Vec::with_capacity(measurement_count as usize);
        for _ in 0..measurement_count {
            measurements.push(LoudnessBaseMeasurement::deserialize(&mut reader)?);
        }

        Ok(Self {
            eq_set_id,
            downmix_id,
            drc_set_id,
            bs_sample_peak_level,
            bs_true_peak_level,
            measurement_system_for_tp,
            reliability_for_tp,
            measurement_count,
            measurements,
        })
    }
}

impl LoudnessBase {
    fn serialize<W>(&self, writer: W, version: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if version >= 1 {
            bit_writer.write_bits(0, 2)?;
            bit_writer.write_bits(
                self.eq_set_id
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "eq_set_ID is required"))? as u64,
                6,
            )?;
        }

        bit_writer.write_bits(0, 3)?;
        bit_writer.write_bits(self.downmix_id as u64, 7)?;
        bit_writer.write_bits(self.drc_set_id as u64, 6)?;

        bit_writer.write_bits(pad_to_u64(&self.bs_sample_peak_level.to_be_bytes()), 12)?;
        bit_writer.write_bits(pad_to_u64(&self.bs_true_peak_level.to_be_bytes()), 12)?;
        bit_writer.write_bits(self.measurement_system_for_tp as u64, 4)?;
        bit_writer.write_bits(self.reliability_for_tp as u64, 4)?;

        self.measurement_count.serialize(&mut bit_writer)?;
        for measurement in &self.measurements {
            measurement.serialize(&mut bit_writer)?;
        }

        Ok(())
    }
}

impl IsoSized for LoudnessBase {
    fn size(&self) -> usize {
        let mut size = 0;
        if self.eq_set_id.is_some() {
            size += 1;
        }

        size += 2; // downmix_id + drc_set_id
        size += 4; // bs_sample_peak_level + bs_true_peak_level + measurement_system_for_tp + reliability_for_tp
        size += 1; // measurement_count
        size += self.measurements.size();
        size
    }
}

/// Measurement in [`LoudnessBase`].
#[derive(Debug, PartialEq, Eq)]
pub struct LoudnessBaseMeasurement {
    /// An index for the measurement method as defined in ISO/IEC 23003-4; all others are reserved.
    pub method_definition: u8,
    /// The method value.
    ///
    /// See [`method_definition`](Self::method_definition).
    pub method_value: u8,
    /// An index for the measurement system as defined in ISO/IEC 23003-4; all others are reserved.
    pub measurement_system: u8,
    /// - 0: Reliability is unknown.
    /// - 1: Value is reported/imported but unverified.
    /// - 2: Value is a 'not to exceed' ceiling.
    /// - 3: Value is measured and accurate.
    /// - 4: All other values are reserved.
    pub reliability: u8,
}

impl<'a> Deserialize<'a> for LoudnessBaseMeasurement {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let method_definition = u8::deserialize(&mut reader)?;
        let method_value = u8::deserialize(&mut reader)?;
        let byte = u8::deserialize(&mut reader)?;
        let measurement_system = (byte >> 4) & 0x0F;
        let reliability = byte & 0x0F;

        Ok(Self {
            method_definition,
            method_value,
            measurement_system,
            reliability,
        })
    }
}

impl Serialize for LoudnessBaseMeasurement {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.method_definition.serialize(&mut writer)?;
        self.method_value.serialize(&mut writer)?;

        let mut byte = 0u8;
        byte |= (self.measurement_system & 0b1111) << 4;
        byte |= self.reliability & 0b1111;
        byte.serialize(&mut writer)?;

        Ok(())
    }
}

impl IsoSized for LoudnessBaseMeasurement {
    fn size(&self) -> usize {
        1 // method_definition
            + 1 // method_value
            + 1 // measurement_system + reliability
    }
}

/// Track loudness info
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tlou", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackLoudnessInfo {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The loudness base box that this box inherits from.
    pub base_box: LoudnessBaseBox,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for TrackLoudnessInfo {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let base_box = LoudnessBaseBox::deserialize_seed(&mut reader, &full_header)?;

        Ok(Self { full_header, base_box })
    }
}

impl Serialize for TrackLoudnessInfo {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.base_box.serialize(&mut writer, self.full_header.version)?;
        Ok(())
    }
}

impl IsoSized for TrackLoudnessInfo {
    fn size(&self) -> usize {
        Self::add_header_size(self.full_header.size() + self.base_box.size(self.full_header.version))
    }
}

/// Album loudness info
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"alou", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct AlbumLoudnessInfo {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The loudness base box that this box inherits from.
    pub base_box: LoudnessBaseBox,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for AlbumLoudnessInfo {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let base_box = LoudnessBaseBox::deserialize_seed(&mut reader, &full_header)?;

        Ok(Self { full_header, base_box })
    }
}

impl Serialize for AlbumLoudnessInfo {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.base_box.serialize(&mut writer, self.full_header.version)?;
        Ok(())
    }
}

impl IsoSized for AlbumLoudnessInfo {
    fn size(&self) -> usize {
        Self::add_header_size(self.full_header.size() + self.base_box.size(self.full_header.version))
    }
}

/// Loudness box
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ludt", crate_path = crate)]
pub struct LoudnessBox {
    /// The contained [`TrackLoudnessInfo`] boxes. (any quantity)
    #[iso_box(nested_box(collect))]
    pub loudness: Vec<TrackLoudnessInfo>,
    /// The contained [`AlbumLoudnessInfo`] boxes. (any quantity)
    #[iso_box(nested_box(collect))]
    pub album_loudness: Vec<AlbumLoudnessInfo>,
}
