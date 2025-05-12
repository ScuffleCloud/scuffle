use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::SampleEntry;
use crate::{BoxHeader, BoxType, FullBoxHeader, IsoBox};

/// Sound media header
///
/// ISO/IEC 14496-12 - 12.2.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"smhd", crate_path = crate)]
pub struct SoundMediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub balance: i16,
    pub reserved: u16,
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
#[derive(Debug)]
pub struct AudioSampleEntry {
    pub sample_entry: SampleEntry,
    pub channelcount: u16,
    pub samplesize: u16,
    pub pre_defined: u16,
    pub samplerate: u32,
}

impl<'a> Deserialize<'a> for AudioSampleEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let sample_entry = SampleEntry::deserialize(&mut reader)?;
        u64::deserialize(&mut reader)?; // reserved
        let channelcount = u16::deserialize(&mut reader)?;
        let samplesize = u16::deserialize(&mut reader)?;
        let pre_defined = u16::deserialize(&mut reader)?;
        u16::deserialize(&mut reader)?; // reserved
        let samplerate = u32::deserialize(&mut reader)?;

        Ok(Self {
            sample_entry,
            channelcount,
            samplesize,
            pre_defined,
            samplerate,
        })
    }
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"srat", crate_path = crate)]
pub struct SamplingRateBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
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
#[derive(Debug)]
pub struct AudioSampleEntryV1 {
    pub sample_entry: SampleEntry,
    pub entry_version: u16,
    pub channelcount: u16,
    pub samplesize: u16,
    pub pre_defined: u16,
    pub samplerate: u32,
}

/// Channel layout
///
/// ISO/IEC 14496-12 - 12.2.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"chnl", crate_path = crate)]
pub struct ChannelLayout<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub data: BytesCow<'a>,
}

/// Down mix instructions
///
/// ISO/IEC 14496-12 - 12.2.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"dmix", crate_path = crate)]
pub struct DownMixInstructions<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub data: BytesCow<'a>,
}

/// Audio stream loudness base box
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(Debug)]
pub struct LoudnessBaseBox {
    pub loudness_info_type: Option<u8>,
    pub loudness_base_count: u8,
    /// `mae_group_ID` or `mae_group_preset_ID` depending on the value of `loudness_info_type`.
    pub mae_group_id: Option<u8>,
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

#[derive(Debug)]
pub struct LoudnessBase {
    pub eq_set_id: Option<u8>,
    pub downmix_id: u8,
    pub drc_set_id: u8,
    pub bs_sample_peak_level: i16,
    pub bs_true_peak_level: i16,
    pub measurement_system_for_tp: u8,
    pub reliability_for_tp: u8,
    pub measurement_count: u8,
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

#[derive(Debug)]
pub struct LoudnessBaseMeasurement {
    pub method_definition: u8,
    pub method_value: u8,
    pub measurement_system: u8,
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

/// Track loudness info
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(Debug)]
pub struct TrackLoudnessInfo {
    pub header: FullBoxHeader,
    pub base_box: LoudnessBaseBox,
}

impl IsoBox for TrackLoudnessInfo {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"tlou");
}

impl<'a> Deserialize<'a> for TrackLoudnessInfo {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackLoudnessInfo {
    fn deserialize_seed<R>(reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            base_box: LoudnessBaseBox::deserialize_seed(reader, &seed)?,
            header: seed,
        })
    }
}

/// Album loudness info
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(Debug)]
pub struct AlbumLoudnessInfo {
    pub header: FullBoxHeader,
    pub base_box: LoudnessBaseBox,
}

impl IsoBox for AlbumLoudnessInfo {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"alou");
}

impl<'a> Deserialize<'a> for AlbumLoudnessInfo {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(&mut reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for AlbumLoudnessInfo {
    fn deserialize_seed<R>(reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            base_box: LoudnessBaseBox::deserialize_seed(reader, &seed)?,
            header: seed,
        })
    }
}

/// Loudness box
///
/// ISO/IEC 14496-12 - 12.2.7
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ludt", crate_path = crate)]
pub struct LoudnessBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub loudness: Vec<TrackLoudnessInfo>,
    #[iso_box(nested_box(collect))]
    pub album_loudness: Vec<AlbumLoudnessInfo>,
}
