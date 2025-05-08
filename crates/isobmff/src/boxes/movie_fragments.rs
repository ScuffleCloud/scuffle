use std::io;

use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, ZeroCopyReader};

use super::{
    CompositionToDecodeBox, MetaBox, SampleAuxiliaryInformationOffsetsBox, SampleAuxiliaryInformationSizesBox,
    SampleGroupDescriptionBox, SampleToGroupBox, SubSampleInformationBox, UserDataBox,
};
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox};

/// Movie extends box
///
/// ISO/IEC 14496-12 - 8.8.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mvex", crate_path = crate)]
pub struct MovieExtendsBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub mehd: Option<MovieExtendsHeaderBox>,
    #[iso_box(nested_box(collect))]
    pub trex: Vec<TrackExtendsBox>,
    #[iso_box(nested_box(collect))]
    pub leva: Option<LevelAssignmentBox>,
}

/// Movie extends header box
///
/// ISO/IEC 14496-12 - 8.8.2
#[derive(Debug)]
pub struct MovieExtendsHeaderBox {
    pub header: FullBoxHeader,
    pub fragment_duration: u64,
}

impl IsoBox for MovieExtendsHeaderBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"mehd";
}

impl<'a> Deserialize<'a> for MovieExtendsHeaderBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for MovieExtendsHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let fragment_duration = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        Ok(Self {
            header: seed,
            fragment_duration,
        })
    }
}

/// Track extends box
///
/// ISO/IEC 14496-12 - 8.8.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trex", crate_path = crate)]
pub struct TrackExtendsBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: u32,
}

/// Movie fragment box
///
/// ISO/IEC 14496-12 - 8.8.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"moof", crate_path = crate)]
pub struct MovieFragmentBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub mfhd: MovieFragmentHeaderBox,
    #[iso_box(nested_box(collect))]
    pub traf: Vec<TrackFragmentBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub trun: Vec<TrackRunBox>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
}

/// Movie fragment header box
///
/// ISO/IEC 14496-12 - 8.8.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mfhd", crate_path = crate)]
pub struct MovieFragmentHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub sequence_number: u32,
}

/// Track fragment box
///
/// ISO/IEC 14496-12 - 8.8.6
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"traf", crate_path = crate)]
pub struct TrackFragmentBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub tfhd: TrackFragmentHeaderBox,
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
    #[iso_box(nested_box(collect))]
    pub tfdt: Option<TrackFragmentBaseMediaDecodeTimeBox>,
    #[iso_box(nested_box(collect))]
    pub sbgp: Vec<SampleToGroupBox>,
    #[iso_box(nested_box(collect))]
    pub sgpd: Vec<SampleGroupDescriptionBox>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
}

/// Track fragment header box
///
/// ISO/IEC 14496-12 - 8.8.7
#[derive(Debug)]
pub struct TrackFragmentHeaderBox {
    pub header: FullBoxHeader,
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<u32>,
}

impl IsoBox for TrackFragmentHeaderBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"tfhd";
}

impl<'a> Deserialize<'a> for TrackFragmentHeaderBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TfFlags: u32 {
        const BaseDataOffsetPresent = 0x000001;
        const SampleDescriptionIndexPresent = 0x000002;
        const DefaultSampleDurationPresent = 0x000008;
        const DefaultSampleSizePresent = 0x000010;
        const DefaultSampleFlagsPresent = 0x000020;
        const DurationIsEmpty = 0x010000;
        const DefaultBaseIsMoof = 0x020000;
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackFragmentHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let flags = TfFlags::from_bits_truncate(seed.flags);

        let track_id = u32::deserialize(&mut reader)?;
        let base_data_offset = if flags.contains(TfFlags::BaseDataOffsetPresent) {
            Some(u64::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_description_index = if flags.contains(TfFlags::SampleDescriptionIndexPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_sample_duration = if flags.contains(TfFlags::DefaultSampleDurationPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_sample_size = if flags.contains(TfFlags::DefaultSampleSizePresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_sample_flags = if flags.contains(TfFlags::DefaultSampleFlagsPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            header: seed,
            track_id,
            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        })
    }
}

/// Track fragment run box
///
/// ISO/IEC 14496-12 - 8.8.8
#[derive(Debug)]
pub struct TrackRunBox {
    pub header: FullBoxHeader,
    pub sample_count: u32,
    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<u32>,
    pub samples: Vec<TrackRunBoxSample>,
}

#[derive(Debug)]
pub struct TrackRunBoxSample {
    pub sample_duration: Option<u32>,
    pub sample_size: Option<u32>,
    pub sample_flags: Option<u32>,
    /// Should be interpreted as signed when version is 1
    pub sample_composition_time_offset: Option<u32>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TrFlags: u32 {
        const DataOffsetPresent = 0x000001;
        const FirstSampleFlagsPresent = 0x000004;
        const SampleDurationPresent = 0x000100;
        const SampleSizePresent = 0x000200;
        const SampleFlagsPresent = 0x000400;
        const SampleCompositionTimeOffsetsPresent = 0x000800;
    }
}

impl<'a> DeserializeSeed<'a, TrFlags> for TrackRunBoxSample {
    fn deserialize_seed<R>(mut reader: R, seed: TrFlags) -> std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let sample_duration = if seed.contains(TrFlags::SampleDurationPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_size = if seed.contains(TrFlags::SampleSizePresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_flags = if seed.contains(TrFlags::SampleFlagsPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_composition_time_offset = if seed.contains(TrFlags::SampleCompositionTimeOffsetsPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            sample_duration,
            sample_size,
            sample_flags,
            sample_composition_time_offset,
        })
    }
}

impl IsoBox for TrackRunBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"trun";
}

impl<'a> Deserialize<'a> for TrackRunBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackRunBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let flags = TrFlags::from_bits_truncate(seed.flags);

        let sample_count = u32::deserialize(&mut reader)?;
        let data_offset = if flags.contains(TrFlags::DataOffsetPresent) {
            Some(i32::deserialize(&mut reader)?)
        } else {
            None
        };
        let first_sample_flags = if flags.contains(TrFlags::FirstSampleFlagsPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let mut samples = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count {
            let sample = TrackRunBoxSample::deserialize_seed(&mut reader, flags)?;
            samples.push(sample);
        }

        Ok(Self {
            header: seed,
            sample_count,
            data_offset,
            first_sample_flags,
            samples,
        })
    }
}

/// Movie fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.9
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mfra", crate_path = crate)]
pub struct MovieFragmentRandomAccessBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub tfra: Vec<TrackFragmentRandomAccessBox>,
    #[iso_box(nested_box)]
    pub mfro: MovieFragmentRandomAccessOffsetBox,
}

/// Track fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.10
#[derive(Debug)]
pub struct TrackFragmentRandomAccessBox {
    pub header: FullBoxHeader,
    pub track_id: u32,
    pub length_size_of_traf_num: u8,
    pub length_size_of_trun_num: u8,
    pub length_size_of_sample_num: u8,
    pub number_of_entry: u32,
    pub entries: Vec<TrackFragmentRandomAccessBoxEntry>,
}

impl IsoBox for TrackFragmentRandomAccessBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"tfra";
}

impl<'a> Deserialize<'a> for TrackFragmentRandomAccessBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = <crate::BoxHeader as Deserialize>::deserialize(&mut reader)?;
        let seed = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, seed)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackFragmentRandomAccessBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let track_id = u32::deserialize(&mut reader)?;
        // 00000000 00000000 00000000 00xxxxxx
        let bytes = u32::deserialize(&mut reader)?;
        let length_size_of_traf_num = ((bytes >> 4) & 0b11) as u8;
        let length_size_of_trun_num = ((bytes >> 2) & 0b11) as u8;
        let length_size_of_sample_num = (bytes & 0b11) as u8;
        let number_of_entry = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(number_of_entry as usize);
        for _ in 0..number_of_entry {
            let time = if seed.version == 1 {
                u64::deserialize(&mut reader)?
            } else {
                u32::deserialize(&mut reader)? as u64
            };
            let moof_offset = if seed.version == 1 {
                u64::deserialize(&mut reader)?
            } else {
                u32::deserialize(&mut reader)? as u64
            };

            // The length of the following fields is bound to 3 bytes because the length fields are all 2 bits
            let traf_number = pad_to_u32(reader.try_read(length_size_of_traf_num as usize)?);
            let trun_number = pad_to_u32(reader.try_read(length_size_of_trun_num as usize)?);
            let sample_number = pad_to_u32(reader.try_read(length_size_of_sample_num as usize)?);

            entries.push(TrackFragmentRandomAccessBoxEntry {
                time,
                moof_offset,
                traf_number,
                trun_number,
                sample_number,
            });
        }

        Ok(Self {
            header: seed,
            track_id,
            length_size_of_traf_num,
            length_size_of_trun_num,
            length_size_of_sample_num,
            number_of_entry,
            entries,
        })
    }
}

fn pad_to_u32(bytes: BytesCow<'_>) -> u32 {
    // We copy the bytes into a 4 byte array and convert it to a u32
    assert!(bytes.len() <= 4);
    let mut buf = [0u8; 4];
    buf[4 - bytes.len()..].copy_from_slice(bytes.as_bytes());
    u32::from_be_bytes(buf)
}

#[derive(Debug)]
pub struct TrackFragmentRandomAccessBoxEntry {
    pub time: u64,
    pub moof_offset: u64,
    pub traf_number: u32,
    pub trun_number: u32,
    pub sample_number: u32,
}

/// Movie fragment random access offset box
///
/// ISO/IEC 14496-12 - 8.8.11
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mfro", crate_path = crate)]
pub struct MovieFragmentRandomAccessOffsetBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub parent_size: u32,
}

/// Track fragment decode time box
///
/// ISO/IEC 14496-12 - 8.8.12
#[derive(Debug)]
pub struct TrackFragmentBaseMediaDecodeTimeBox {
    pub header: FullBoxHeader,
    pub base_media_decode_time: u64,
}

impl IsoBox for TrackFragmentBaseMediaDecodeTimeBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"tfdt";
}

impl<'a> Deserialize<'a> for TrackFragmentBaseMediaDecodeTimeBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackFragmentBaseMediaDecodeTimeBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let base_media_decode_time = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        Ok(Self {
            header: seed,
            base_media_decode_time,
        })
    }
}

/// Level assignment box
///
/// ISO/IEC 14496-12 - 8.8.13
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"leva", crate_path = crate)]
pub struct LevelAssignmentBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub level_count: u8,
    #[iso_box(repeated)]
    pub levels: Vec<LevelAssignmentBoxLevel>,
}

#[derive(Debug)]
pub struct LevelAssignmentBoxLevel {
    pub track_id: u32,
    pub padding_flag: bool,
    pub assignment_type: LevelAssignmentBoxLevelAssignmentType,
}

impl<'a> Deserialize<'a> for LevelAssignmentBoxLevel {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let track_id = u32::deserialize(&mut reader)?;
        let byte = u8::deserialize(&mut reader)?;
        let padding_flag = (byte >> 7) == 1;
        let assignment_type = byte & 0b01111111;
        let assignment_type = match assignment_type {
            0 => LevelAssignmentBoxLevelAssignmentType::Type0 {
                grouping_type: u32::deserialize(&mut reader)?,
            },
            1 => LevelAssignmentBoxLevelAssignmentType::Type1 {
                grouping_type: u32::deserialize(&mut reader)?,
                grouping_type_parameter: u32::deserialize(&mut reader)?,
            },
            2 => LevelAssignmentBoxLevelAssignmentType::Type2,
            3 => LevelAssignmentBoxLevelAssignmentType::Type3,
            4 => LevelAssignmentBoxLevelAssignmentType::Type4 {
                sub_track_id: u32::deserialize(&mut reader)?,
            },
            _ => LevelAssignmentBoxLevelAssignmentType::Other(assignment_type),
        };

        Ok(Self {
            track_id,
            padding_flag,
            assignment_type,
        })
    }
}

#[derive(Debug)]
pub enum LevelAssignmentBoxLevelAssignmentType {
    Type0 {
        grouping_type: u32,
    },
    Type1 {
        grouping_type: u32,
        grouping_type_parameter: u32,
    },
    Type2,
    Type3,
    Type4 {
        sub_track_id: u32,
    },
    Other(u8),
}

/// Track Extension Properties box
///
/// ISO/IEC 14496-12 - 8.8.15
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trep", crate_path = crate)]
pub struct TrackExtensionPropertiesBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub track_id: u32,
    #[iso_box(nested_box(collect))]
    pub cslg: Option<CompositionToDecodeBox>,
    #[iso_box(nested_box(collect))]
    pub assp: Option<AlternativeStartupSequencePropertiesBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Alternative startup sequence properties box
///
/// ISO/IEC 14496-12 - 8.8.16
#[derive(Debug)]
pub struct AlternativeStartupSequencePropertiesBox {
    pub header: FullBoxHeader,
    pub version: AlternativeStartupSequencePropertiesBoxVersion,
}

impl IsoBox for AlternativeStartupSequencePropertiesBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"assp";
}

impl<'a> Deserialize<'a> for AlternativeStartupSequencePropertiesBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for AlternativeStartupSequencePropertiesBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let version = match seed.version {
            0 => AlternativeStartupSequencePropertiesBoxVersion::Version0 {
                min_initial_alt_startup_offset: i32::deserialize(&mut reader)?,
            },
            1 => {
                let num_entries = u32::deserialize(&mut reader)?;
                AlternativeStartupSequencePropertiesBoxVersion::Version1 {
                    num_entries,
                    entries: {
                        let mut entries = Vec::with_capacity(num_entries as usize);

                        for _ in 0..num_entries {
                            let entry = AlternativeStartupSequencePropertiesBoxVersion1Entry::deserialize(&mut reader)?;
                            entries.push(entry);
                        }

                        entries
                    },
                }
            }
            v => AlternativeStartupSequencePropertiesBoxVersion::Other(v),
        };

        Ok(Self { header: seed, version })
    }
}

#[derive(Debug)]
pub enum AlternativeStartupSequencePropertiesBoxVersion {
    Version0 {
        min_initial_alt_startup_offset: i32,
    },
    Version1 {
        num_entries: u32,
        entries: Vec<AlternativeStartupSequencePropertiesBoxVersion1Entry>,
    },
    Other(u8),
}

#[derive(Debug)]
pub struct AlternativeStartupSequencePropertiesBoxVersion1Entry {
    pub grouping_type_parameter: u32,
    pub min_initial_alt_startup_offset: i32,
}

impl<'a> Deserialize<'a> for AlternativeStartupSequencePropertiesBoxVersion1Entry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let grouping_type_parameter = u32::deserialize(&mut reader)?;
        let min_initial_alt_startup_offset = i32::deserialize(&mut reader)?;

        Ok(Self {
            grouping_type_parameter,
            min_initial_alt_startup_offset,
        })
    }
}
