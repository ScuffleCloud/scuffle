use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, SerializeSeed, U24Be, ZeroCopyReader};
use scuffle_bytes_util::{BitReader, BitWriter};

use super::{
    CompositionToDecodeBox, MetaBox, SampleAuxiliaryInformationOffsetsBox, SampleAuxiliaryInformationSizesBox,
    SampleGroupDescriptionBox, SampleToGroupBox, SubSampleInformationBox, UserDataBox,
};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Movie extends box
///
/// ISO/IEC 14496-12 - 8.8.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mvex", crate_path = crate)]
pub struct MovieExtendsBox {
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mehd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct MovieExtendsHeaderBox {
    pub full_header: FullBoxHeader,
    pub fragment_duration: u64,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for MovieExtendsHeaderBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let fragment_duration = if full_header.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        Ok(Self {
            full_header,
            fragment_duration,
        })
    }
}

impl Serialize for MovieExtendsHeaderBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        if self.full_header.version == 1 {
            self.fragment_duration.serialize(&mut writer)?;
        } else {
            (self.fragment_duration as u32).serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for MovieExtendsHeaderBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size(); // header
        if self.full_header.version == 1 {
            size += 8; // fragment_duration
        } else {
            size += 4; // fragment_duration
        }
        Self::add_header_size(size)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SampleFlags {
    pub reserved: u8,
    pub is_leading: u8,
    pub sample_depends_on: u8,
    pub sample_is_depended_on: u8,
    pub sample_has_redundancy: u8,
    pub sample_padding_value: u8,
    pub sample_is_non_sync_sample: bool,
    pub sample_degradation_priority: u16,
}

impl<'a> Deserialize<'a> for SampleFlags {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let mut reader = BitReader::new(reader.as_std());
        let reserved = reader.read_bits(4)? as u8;
        let is_leading = reader.read_bits(2)? as u8;
        let sample_depends_on = reader.read_bits(2)? as u8;
        let sample_is_depended_on = reader.read_bits(2)? as u8;
        let sample_has_redundancy = reader.read_bits(2)? as u8;
        let sample_padding_value = reader.read_bits(3)? as u8;
        let sample_is_non_sync_sample = reader.read_bit()?;
        let sample_degradation_priority = reader.read_bits(16)? as u16;

        Ok(Self {
            reserved,
            is_leading,
            sample_depends_on,
            sample_is_depended_on,
            sample_has_redundancy,
            sample_padding_value,
            sample_is_non_sync_sample,
            sample_degradation_priority,
        })
    }
}

impl Serialize for SampleFlags {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);
        bit_writer.write_bits(self.reserved as u64, 4)?;
        bit_writer.write_bits(self.is_leading as u64, 2)?;
        bit_writer.write_bits(self.sample_depends_on as u64, 2)?;
        bit_writer.write_bits(self.sample_is_depended_on as u64, 2)?;
        bit_writer.write_bits(self.sample_has_redundancy as u64, 2)?;
        bit_writer.write_bits(self.sample_padding_value as u64, 3)?;
        bit_writer.write_bit(self.sample_is_non_sync_sample)?;
        bit_writer.write_bits(self.sample_degradation_priority as u64, 16)?;

        Ok(())
    }
}

impl IsoSized for SampleFlags {
    fn size(&self) -> usize {
        4
    }
}

/// Track extends box
///
/// ISO/IEC 14496-12 - 8.8.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trex", crate_path = crate)]
pub struct TrackExtendsBox {
    pub full_header: FullBoxHeader,
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: SampleFlags,
}

impl TrackExtendsBox {
    pub fn new(track_id: u32) -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            track_id,
            default_sample_description_index: 1,
            default_sample_duration: 0,
            default_sample_size: 0,
            default_sample_flags: SampleFlags {
                reserved: 0,
                is_leading: 0,
                sample_depends_on: 0,
                sample_is_depended_on: 0,
                sample_has_redundancy: 0,
                sample_padding_value: 0,
                sample_is_non_sync_sample: false,
                sample_degradation_priority: 0,
            },
        }
    }
}

/// Movie fragment box
///
/// ISO/IEC 14496-12 - 8.8.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"moof", crate_path = crate)]
pub struct MovieFragmentBox<'a> {
    #[iso_box(nested_box)]
    pub mfhd: MovieFragmentHeaderBox,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub traf: Vec<TrackFragmentBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Movie fragment header box
///
/// ISO/IEC 14496-12 - 8.8.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mfhd", crate_path = crate)]
pub struct MovieFragmentHeaderBox {
    pub full_header: FullBoxHeader,
    pub sequence_number: u32,
}

impl MovieFragmentHeaderBox {
    pub fn new(sequence_number: u32) -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            sequence_number,
        }
    }
}

/// Track fragment box
///
/// ISO/IEC 14496-12 - 8.8.6
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"traf", crate_path = crate)]
pub struct TrackFragmentBox<'a> {
    #[iso_box(nested_box)]
    pub tfhd: TrackFragmentHeaderBox,
    #[iso_box(nested_box(collect))]
    pub trun: Vec<TrackRunBox>,
    #[iso_box(nested_box(collect))]
    pub sbgp: Vec<SampleToGroupBox>,
    #[iso_box(nested_box(collect))]
    pub sgpd: Vec<SampleGroupDescriptionBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
    #[iso_box(nested_box(collect))]
    pub tfdt: Option<TrackFragmentBaseMediaDecodeTimeBox>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Track fragment header box
///
/// ISO/IEC 14496-12 - 8.8.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"tfhd", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct TrackFragmentHeaderBox {
    // full header:
    pub version: u8,
    pub flags: TfFlags,
    // body:
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<SampleFlags>,
}

impl TrackFragmentHeaderBox {
    pub fn new(
        track_id: u32,
        base_data_offset: Option<u64>,
        sample_description_index: Option<u32>,
        default_sample_duration: Option<u32>,
        default_sample_size: Option<u32>,
        default_sample_flags: Option<SampleFlags>,
    ) -> Self {
        let mut flags = TfFlags::DefaultBaseIsMoof;

        if base_data_offset.is_some() {
            flags |= TfFlags::BaseDataOffsetPresent;
        }

        if sample_description_index.is_some() {
            flags |= TfFlags::SampleDescriptionIndexPresent;
        }

        if default_sample_duration.is_some() {
            flags |= TfFlags::DefaultSampleDurationPresent;
        }

        if default_sample_size.is_some() {
            flags |= TfFlags::DefaultSampleSizePresent;
        }

        if default_sample_flags.is_some() {
            flags |= TfFlags::DefaultSampleFlagsPresent;
        }

        Self {
            version: 0,
            flags,
            track_id,
            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        }
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

impl<'a> Deserialize<'a> for TfFlags {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let flags = U24Be::deserialize(reader)?;
        Ok(TfFlags::from_bits_truncate(*flags))
    }
}

impl Serialize for TfFlags {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        U24Be(self.bits()).serialize(writer)
    }
}

impl IsoSized for TfFlags {
    fn size(&self) -> usize {
        3
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for TrackFragmentHeaderBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let version = u8::deserialize(&mut reader)?;
        let flags = TfFlags::deserialize(&mut reader)?;

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
            Some(SampleFlags::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            version,
            flags,
            track_id,
            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        })
    }
}

impl Serialize for TrackFragmentHeaderBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.version.serialize(&mut writer)?;
        self.flags.serialize(&mut writer)?;

        self.track_id.serialize(&mut writer)?;
        if self.flags.contains(TfFlags::BaseDataOffsetPresent) {
            self.base_data_offset
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "base_data_offset is required"))?
                .serialize(&mut writer)?;
        }
        if self.flags.contains(TfFlags::SampleDescriptionIndexPresent) {
            self.sample_description_index
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "sample_description_index is required",
                ))?
                .serialize(&mut writer)?;
        }
        if self.flags.contains(TfFlags::DefaultSampleDurationPresent) {
            self.default_sample_duration
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "default_sample_duration is required",
                ))?
                .serialize(&mut writer)?;
        }
        if self.flags.contains(TfFlags::DefaultSampleSizePresent) {
            self.default_sample_size
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "default_sample_size is required"))?
                .serialize(&mut writer)?;
        }
        if self.flags.contains(TfFlags::DefaultSampleFlagsPresent) {
            self.default_sample_flags
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "default_sample_flags is required"))?
                .serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Track fragment run box
///
/// ISO/IEC 14496-12 - 8.8.8
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"trun", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackRunBox {
    // full header:
    pub version: u8,
    pub flags: TrFlags,
    // body:
    pub sample_count: u32,
    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<SampleFlags>,
    pub samples: Vec<TrackRunBoxSample>,
}

impl TrackRunBox {
    pub fn new(samples: Vec<TrackRunBoxSample>, first_sample_flags: Option<SampleFlags>) -> Self {
        let mut flags = TrFlags::DataOffsetPresent;

        if let Some(first_sample) = samples.first() {
            if first_sample.sample_duration.is_some() {
                flags |= TrFlags::SampleDurationPresent;
            }
            if first_sample.sample_size.is_some() {
                flags |= TrFlags::SampleSizePresent;
            }
            if first_sample.sample_flags.is_some() {
                flags |= TrFlags::SampleFlagsPresent;
            }
            if first_sample.sample_composition_time_offset.is_some() {
                flags |= TrFlags::SampleCompositionTimeOffsetsPresent;
            }
        }

        let version = if samples
            .iter()
            .any(|s| s.sample_composition_time_offset.is_some_and(|o| o < 0))
        {
            1
        } else {
            0
        };

        Self {
            version,
            flags,
            sample_count: samples.len() as u32,
            data_offset: Some(0),
            first_sample_flags,
            samples,
        }
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for TrackRunBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let version = u8::deserialize(&mut reader)?;
        let flags = TrFlags::deserialize(&mut reader)?;

        let sample_count = u32::deserialize(&mut reader)?;
        let data_offset = if flags.contains(TrFlags::DataOffsetPresent) {
            Some(i32::deserialize(&mut reader)?)
        } else {
            None
        };
        let first_sample_flags = if flags.contains(TrFlags::FirstSampleFlagsPresent) {
            Some(SampleFlags::deserialize(&mut reader)?)
        } else {
            None
        };

        let mut samples = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count {
            samples.push(TrackRunBoxSample::deserialize_seed(&mut reader, (version, flags))?);
        }

        Ok(Self {
            version,
            flags,
            sample_count,
            data_offset,
            first_sample_flags,
            samples,
        })
    }
}

impl IsoSized for TrackRunBox {
    fn size(&self) -> usize {
        let mut size = 0;
        size += self.version.size(); // version
        size += self.flags.size(); // flags
        size += 4; // sample_count
        if self.flags.contains(TrFlags::DataOffsetPresent) {
            size += 4; // data_offset
        }
        if self.flags.contains(TrFlags::FirstSampleFlagsPresent) {
            size += 4; // first_sample_flags
        }

        size += self.samples.iter().map(|s| s.size(self.flags)).sum::<usize>();

        Self::add_header_size(size)
    }
}

impl Serialize for TrackRunBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.version.serialize(&mut writer)?;
        self.flags.serialize(&mut writer)?;

        self.sample_count.serialize(&mut writer)?;
        if self.flags.contains(TrFlags::DataOffsetPresent) {
            self.data_offset
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "data_offset is required"))?
                .serialize(&mut writer)?;
        }
        if self.flags.contains(TrFlags::FirstSampleFlagsPresent) {
            self.first_sample_flags
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "first_sample_flags is required"))?
                .serialize(&mut writer)?;
        }

        for sample in &self.samples {
            sample.serialize_seed(&mut writer, (self.version, self.flags))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct TrackRunBoxSample {
    pub sample_duration: Option<u32>,
    pub sample_size: Option<u32>,
    pub sample_flags: Option<SampleFlags>,
    /// Either a u32 or a i32
    pub sample_composition_time_offset: Option<i64>,
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

impl<'a> Deserialize<'a> for TrFlags {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let flags = U24Be::deserialize(reader)?;
        Ok(TrFlags::from_bits_truncate(*flags))
    }
}

impl Serialize for TrFlags {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        U24Be(self.bits()).serialize(writer)
    }
}

impl IsoSized for TrFlags {
    fn size(&self) -> usize {
        3
    }
}

impl<'a> DeserializeSeed<'a, (u8, TrFlags)> for TrackRunBoxSample {
    fn deserialize_seed<R>(mut reader: R, seed: (u8, TrFlags)) -> std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let (version, flags) = seed;

        let sample_duration = if flags.contains(TrFlags::SampleDurationPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_size = if flags.contains(TrFlags::SampleSizePresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_flags = if flags.contains(TrFlags::SampleFlagsPresent) {
            Some(SampleFlags::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_composition_time_offset = if flags.contains(TrFlags::SampleCompositionTimeOffsetsPresent) {
            if version == 0 {
                Some(u32::deserialize(&mut reader)? as i64)
            } else {
                Some(i32::deserialize(&mut reader)? as i64)
            }
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

impl SerializeSeed<(u8, TrFlags)> for TrackRunBoxSample {
    fn serialize_seed<W>(&self, mut writer: W, seed: (u8, TrFlags)) -> io::Result<()>
    where
        W: io::Write,
    {
        let (version, flags) = seed;

        if flags.contains(TrFlags::SampleDurationPresent) {
            self.sample_duration
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "sample_duration is required"))?
                .serialize(&mut writer)?;
        }
        if flags.contains(TrFlags::SampleSizePresent) {
            self.sample_size
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "sample_size is required"))?
                .serialize(&mut writer)?;
        }
        if flags.contains(TrFlags::SampleFlagsPresent) {
            self.sample_flags
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "sample_flags is required"))?
                .serialize(&mut writer)?;
        }
        if flags.contains(TrFlags::SampleCompositionTimeOffsetsPresent) {
            let sample_composition_time_offset = self.sample_composition_time_offset.ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "sample_composition_time_offset is required",
            ))?;
            if version == 0 {
                (sample_composition_time_offset as u32).serialize(&mut writer)?;
            } else {
                (sample_composition_time_offset as i32).serialize(&mut writer)?;
            }
        }

        Ok(())
    }
}

impl TrackRunBoxSample {
    pub fn size(&self, flags: TrFlags) -> usize {
        let mut size = 0;
        if flags.contains(TrFlags::SampleDurationPresent) {
            size += 4; // sample_duration
        }
        if flags.contains(TrFlags::SampleSizePresent) {
            size += 4; // sample_size
        }
        if flags.contains(TrFlags::SampleFlagsPresent) {
            size += 4; // sample_flags
        }
        if flags.contains(TrFlags::SampleCompositionTimeOffsetsPresent) {
            size += 4; // sample_composition_time_offset
        }

        size
    }
}

/// Movie fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.9
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mfra", crate_path = crate)]
pub struct MovieFragmentRandomAccessBox {
    #[iso_box(nested_box(collect))]
    pub tfra: Vec<TrackFragmentRandomAccessBox>,
    #[iso_box(nested_box)]
    pub mfro: MovieFragmentRandomAccessOffsetBox,
}

/// Track fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.10
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"tfra", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackFragmentRandomAccessBox {
    pub full_header: FullBoxHeader,
    pub track_id: u32,
    pub length_size_of_traf_num: u8,
    pub length_size_of_trun_num: u8,
    pub length_size_of_sample_num: u8,
    pub number_of_entry: u32,
    pub entries: Vec<TrackFragmentRandomAccessBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for TrackFragmentRandomAccessBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let track_id = u32::deserialize(&mut reader)?;
        // 00000000 00000000 00000000 00xxxxxx
        let bytes = u32::deserialize(&mut reader)?;
        let length_size_of_traf_num = ((bytes >> 4) & 0b11) as u8;
        let length_size_of_trun_num = ((bytes >> 2) & 0b11) as u8;
        let length_size_of_sample_num = (bytes & 0b11) as u8;
        let number_of_entry = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(number_of_entry as usize);
        for _ in 0..number_of_entry {
            let time = if full_header.version == 1 {
                u64::deserialize(&mut reader)?
            } else {
                u32::deserialize(&mut reader)? as u64
            };
            let moof_offset = if full_header.version == 1 {
                u64::deserialize(&mut reader)?
            } else {
                u32::deserialize(&mut reader)? as u64
            };

            // The length of the following fields is bound to 3 bytes because the length fields are all 2 bits
            let traf_number = reader.try_read(length_size_of_traf_num as usize + 1)?.pad_to_u32_be();
            let trun_number = reader.try_read(length_size_of_trun_num as usize + 1)?.pad_to_u32_be();
            let sample_number = reader.try_read(length_size_of_sample_num as usize + 1)?.pad_to_u32_be();

            entries.push(TrackFragmentRandomAccessBoxEntry {
                time,
                moof_offset,
                traf_number,
                trun_number,
                sample_number,
            });
        }

        Ok(Self {
            full_header,
            track_id,
            length_size_of_traf_num,
            length_size_of_trun_num,
            length_size_of_sample_num,
            number_of_entry,
            entries,
        })
    }
}

impl Serialize for TrackFragmentRandomAccessBox {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        self.serialize_box_header(&mut bit_writer)?;
        self.full_header.serialize(&mut bit_writer)?;

        self.track_id.serialize(&mut bit_writer)?;
        bit_writer.write_bits(0, 26)?;
        bit_writer.write_bits(self.length_size_of_traf_num as u64, 2)?;
        bit_writer.write_bits(self.length_size_of_trun_num as u64, 2)?;
        bit_writer.write_bits(self.length_size_of_sample_num as u64, 2)?;
        self.number_of_entry.serialize(&mut bit_writer)?;

        for entry in &self.entries {
            entry.serialize_seed(&mut bit_writer, self)?;
        }

        Ok(())
    }
}

impl IsoSized for TrackFragmentRandomAccessBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size(); // header
        size += 4; // track_id
        size += 4; // length_size_of_traf_num + length_size_of_trun_num + length_size_of_sample_num
        size += 4; // number_of_entry
        size += self.entries.iter().map(|e| e.size(self)).sum::<usize>();

        Self::add_header_size(size)
    }
}

#[derive(Debug)]
pub struct TrackFragmentRandomAccessBoxEntry {
    pub time: u64,
    pub moof_offset: u64,
    pub traf_number: u32,
    pub trun_number: u32,
    pub sample_number: u32,
}

impl SerializeSeed<&TrackFragmentRandomAccessBox> for TrackFragmentRandomAccessBoxEntry {
    fn serialize_seed<W>(&self, writer: W, seed: &TrackFragmentRandomAccessBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if seed.full_header.version == 1 {
            self.time.serialize(&mut bit_writer)?;
            self.moof_offset.serialize(&mut bit_writer)?;
        } else {
            (self.time as u32).serialize(&mut bit_writer)?;
            (self.moof_offset as u32).serialize(&mut bit_writer)?;
        }

        bit_writer.write_bits(self.traf_number as u64, (seed.length_size_of_traf_num + 1) * 8)?;
        bit_writer.write_bits(self.trun_number as u64, (seed.length_size_of_trun_num + 1) * 8)?;
        bit_writer.write_bits(self.sample_number as u64, (seed.length_size_of_sample_num + 1) * 8)?;

        Ok(())
    }
}

impl TrackFragmentRandomAccessBoxEntry {
    pub fn size(&self, parent: &TrackFragmentRandomAccessBox) -> usize {
        let mut size = 0;
        if parent.full_header.version == 1 {
            size += 8;
            size += 8;
        } else {
            size += 4;
            size += 4;
        }
        size += parent.length_size_of_traf_num as usize + 1;
        size += parent.length_size_of_trun_num as usize + 1;
        size += parent.length_size_of_sample_num as usize + 1;
        size
    }
}

/// Movie fragment random access offset box
///
/// ISO/IEC 14496-12 - 8.8.11
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mfro", crate_path = crate)]
pub struct MovieFragmentRandomAccessOffsetBox {
    pub full_header: FullBoxHeader,
    pub parent_size: u32,
}

/// Track fragment decode time box
///
/// ISO/IEC 14496-12 - 8.8.12
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"tfdt", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackFragmentBaseMediaDecodeTimeBox {
    pub full_header: FullBoxHeader,
    pub base_media_decode_time: u64,
}

impl TrackFragmentBaseMediaDecodeTimeBox {
    pub fn new(base_media_decode_time: u64) -> Self {
        let version = if base_media_decode_time > u32::MAX as u64 { 1 } else { 0 };

        Self {
            full_header: FullBoxHeader {
                version,
                flags: U24Be(0),
            },
            base_media_decode_time,
        }
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for TrackFragmentBaseMediaDecodeTimeBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let base_media_decode_time = if full_header.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        Ok(Self {
            full_header,
            base_media_decode_time,
        })
    }
}

impl Serialize for TrackFragmentBaseMediaDecodeTimeBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        if self.full_header.version == 1 {
            self.base_media_decode_time.serialize(&mut writer)?;
        } else {
            (self.base_media_decode_time as u32).serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for TrackFragmentBaseMediaDecodeTimeBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size(); // header
        if self.full_header.version == 1 {
            size += 8; // base_media_decode_time
        } else {
            size += 4; // base_media_decode_time
        }
        Self::add_header_size(size)
    }
}

/// Level assignment box
///
/// ISO/IEC 14496-12 - 8.8.13
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"leva", crate_path = crate)]
pub struct LevelAssignmentBox {
    pub full_header: FullBoxHeader,
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
                grouping_type: <[u8; 4]>::deserialize(&mut reader)?,
            },
            1 => LevelAssignmentBoxLevelAssignmentType::Type1 {
                grouping_type: <[u8; 4]>::deserialize(&mut reader)?,
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

impl Serialize for LevelAssignmentBoxLevel {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.track_id.serialize(&mut writer)?;

        let mut byte = (self.padding_flag as u8) << 7;
        byte |= self.assignment_type.assignment_type() & 0b01111111;
        byte.serialize(&mut writer)?;

        match self.assignment_type {
            LevelAssignmentBoxLevelAssignmentType::Type0 { grouping_type } => {
                grouping_type.serialize(&mut writer)?;
            }
            LevelAssignmentBoxLevelAssignmentType::Type1 {
                grouping_type,
                grouping_type_parameter,
            } => {
                grouping_type.serialize(&mut writer)?;
                grouping_type_parameter.serialize(&mut writer)?;
            }
            LevelAssignmentBoxLevelAssignmentType::Type4 { sub_track_id } => {
                sub_track_id.serialize(&mut writer)?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl IsoSized for LevelAssignmentBoxLevel {
    fn size(&self) -> usize {
        let mut size = 4; // track_id
        size += 1; // padding_flag + assignment_type

        match self.assignment_type {
            LevelAssignmentBoxLevelAssignmentType::Type0 { .. } => size += 4,
            LevelAssignmentBoxLevelAssignmentType::Type1 { .. } => size += 8,
            LevelAssignmentBoxLevelAssignmentType::Type4 { .. } => size += 4,
            _ => {}
        }

        size
    }
}

#[derive(Debug)]
pub enum LevelAssignmentBoxLevelAssignmentType {
    Type0 {
        grouping_type: [u8; 4],
    },
    Type1 {
        grouping_type: [u8; 4],
        grouping_type_parameter: u32,
    },
    Type2,
    Type3,
    Type4 {
        sub_track_id: u32,
    },
    Other(u8),
}

impl LevelAssignmentBoxLevelAssignmentType {
    pub fn assignment_type(&self) -> u8 {
        match self {
            LevelAssignmentBoxLevelAssignmentType::Type0 { .. } => 0,
            LevelAssignmentBoxLevelAssignmentType::Type1 { .. } => 1,
            LevelAssignmentBoxLevelAssignmentType::Type2 => 2,
            LevelAssignmentBoxLevelAssignmentType::Type3 => 3,
            LevelAssignmentBoxLevelAssignmentType::Type4 { .. } => 4,
            LevelAssignmentBoxLevelAssignmentType::Other(v) => *v,
        }
    }
}

/// Track Extension Properties box
///
/// ISO/IEC 14496-12 - 8.8.15
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trep", crate_path = crate)]
pub struct TrackExtensionPropertiesBox<'a> {
    pub full_header: FullBoxHeader,
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"assp", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct AlternativeStartupSequencePropertiesBox {
    pub full_header: FullBoxHeader,
    pub version: AlternativeStartupSequencePropertiesBoxVersion,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for AlternativeStartupSequencePropertiesBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let version = match full_header.version {
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
                            entries.push(AlternativeStartupSequencePropertiesBoxVersion1Entry::deserialize(
                                &mut reader,
                            )?);
                        }

                        entries
                    },
                }
            }
            v => AlternativeStartupSequencePropertiesBoxVersion::Other(v),
        };

        Ok(Self { full_header, version })
    }
}

impl Serialize for AlternativeStartupSequencePropertiesBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        match &self.version {
            AlternativeStartupSequencePropertiesBoxVersion::Version0 {
                min_initial_alt_startup_offset,
            } => {
                min_initial_alt_startup_offset.serialize(&mut writer)?;
            }
            AlternativeStartupSequencePropertiesBoxVersion::Version1 { num_entries, entries } => {
                num_entries.serialize(&mut writer)?;
                for entry in entries {
                    entry.serialize(&mut writer)?;
                }
            }
            _ => {}
        }

        Ok(())
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

impl IsoSized for AlternativeStartupSequencePropertiesBoxVersion {
    fn size(&self) -> usize {
        match self {
            AlternativeStartupSequencePropertiesBoxVersion::Version0 { .. } => 4,
            AlternativeStartupSequencePropertiesBoxVersion::Version1 { entries, .. } => 4 + entries.size(),
            _ => 0,
        }
    }
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

impl Serialize for AlternativeStartupSequencePropertiesBoxVersion1Entry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.grouping_type_parameter.serialize(&mut writer)?;
        self.min_initial_alt_startup_offset.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for AlternativeStartupSequencePropertiesBoxVersion1Entry {
    fn size(&self) -> usize {
        4 + 4
    }
}
