use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be, ZeroCopyReader};
use scuffle_bytes_util::{BitReader, BitWriter};

use super::{
    CompositionToDecodeBox, MetaBox, SampleAuxiliaryInformationOffsetsBox, SampleAuxiliaryInformationSizesBox,
    SampleGroupDescriptionBox, SampleToGroupBox, SubSampleInformationBox, UserDataBox,
};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Movie extends box
///
/// ISO/IEC 14496-12 - 8.8.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mvex", crate_path = crate)]
pub struct MovieExtendsBox {
    /// The contained [`MovieExtendsHeaderBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub mehd: Option<MovieExtendsHeaderBox>,
    /// The contained [`TrackExtendsBox`]es. (exactly one for each track in the [`MovieBox`](super::MovieBox))
    #[iso_box(nested_box(collect))]
    pub trex: Vec<TrackExtendsBox>,
    /// The contained [`LevelAssignmentBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub leva: Option<LevelAssignmentBox>,
}

/// Movie extends header box
///
/// ISO/IEC 14496-12 - 8.8.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mehd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct MovieExtendsHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A number associated with this fragment.
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

/// Sample flags
///
/// ISO/IEC 14496-12 - 8.8.3
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SampleFlags {
    /// Reserved 4 bits, must be 0.
    pub reserved: u8,
    /// - `0`: The leading nature of this sample is unknown;
    /// - `1`: This sample is a leading sample that has a dependency before the referenced I-picture (and is
    ///   therefore not decodable);
    /// - `2`: This sample is not a leading sample;
    /// - `3`: This sample is a leading sample that has no dependency before the referenced I-picture (and is
    ///   therefore decodable);
    pub is_leading: u8,
    /// - `0`: The dependency of this sample is unknown;
    /// - `1`: This sample does depend on others (not an I picture);
    /// - `2`: This sample does not depend on others (I picture);
    /// - `3`: Reserved;
    pub sample_depends_on: u8,
    /// - `0`: The dependency of other samples on this sample is unknown;
    /// - `1`: Other samples may depend on this one (not disposable);
    /// - `2`: No other sample depends on this one (disposable);
    /// - `3`: Reserved;
    pub sample_is_depended_on: u8,
    /// - `0`: It is unknown whether there is redundant coding in this sample;
    /// - `1`: There is redundant coding in this sample;
    /// - `2`: There is no redundant coding in this sample;
    /// - `3`: Reserved;
    pub sample_has_redundancy: u8,
    /// A value from 0 to 7, indicating the number of padding bits at the end of sample.
    pub sample_padding_value: u8,
    /// Provides the same information as the sync sample table 8.6.2.
    /// When this value is set to 0 for a sample, it is the same as if the sample were not in a movie fragment and
    /// marked with an entry in the sync sample table (or, if all samples are sync samples, the sync sample table
    /// were absent).
    pub sample_is_non_sync_sample: bool,
    /// An integer specifying the degradation priority for each sample.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"trex", crate_path = crate)]
pub struct TrackExtendsBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Identifies the track; this shall be the `track_ID` of a track in the [`MovieBox`](super::MovieBox).
    pub track_id: u32,
    /// Indicates the index of the sample entry that describes, by default,
    /// the samples in the track fragments.
    pub default_sample_description_index: u32,
    /// Indicates the default duration of the samples in the track fragments.
    pub default_sample_duration: u32,
    /// Indicates the default size of the samples in the track fragments.
    pub default_sample_size: u32,
    /// Indicate the default flags values for the samples in the track fragments.
    pub default_sample_flags: SampleFlags,
}

impl TrackExtendsBox {
    /// Creates a new [`TrackExtendsBox`] with the given `track_id`.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"moof", crate_path = crate)]
pub struct MovieFragmentBox<'a> {
    /// The contained [`MovieFragmentHeaderBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub mfhd: MovieFragmentHeaderBox,
    /// The contained [`MovieExtendsBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    /// The contained [`TrackFragmentBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub traf: Vec<TrackFragmentBox<'a>>,
    /// The contained [`UserDataBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Movie fragment header box
///
/// ISO/IEC 14496-12 - 8.8.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mfhd", crate_path = crate)]
pub struct MovieFragmentHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A number associated with this fragment.
    pub sequence_number: u32,
}

impl MovieFragmentHeaderBox {
    /// Creates a new [`MovieFragmentHeaderBox`] with the given `sequence_number`.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"traf", crate_path = crate)]
pub struct TrackFragmentBox<'a> {
    /// The contained [`TrackFragmentHeaderBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub tfhd: TrackFragmentHeaderBox,
    /// The contained [`TrackRunBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub trun: Vec<TrackRunBox>,
    /// The contained [`SampleToGroupBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub sbgp: Vec<SampleToGroupBox>,
    /// The contained [`SampleGroupDescriptionBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub sgpd: Vec<SampleGroupDescriptionBox<'a>>,
    /// The contained [`SubSampleInformationBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    /// The contained [`SampleAuxiliaryInformationSizesBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    /// The contained [`SampleAuxiliaryInformationOffsetsBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
    /// The contained [`TrackFragmentBaseMediaDecodeTimeBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub tfdt: Option<TrackFragmentBaseMediaDecodeTimeBox>,
    /// The contained [`MetaBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    /// The contained [`UserDataBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Track fragment header box
///
/// ISO/IEC 14496-12 - 8.8.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tfhd", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct TrackFragmentHeaderBox {
    // full header:
    /// The version of the box.
    pub version: u8,
    /// The flags of the box.
    pub flags: TfFlags,
    // body:
    /// Identifies the track; this shall be the `track_ID` of a track in the [`MovieBox`](super::MovieBox).
    pub track_id: u32,
    /// The base offset to use when calculating data offsets.
    ///
    /// Present if the [`BaseDataOffsetPresent`](TfFlags::BaseDataOffsetPresent) flag is set.
    pub base_data_offset: Option<u64>,
    /// Indicates the index of the sample entry that describes, by default,
    /// the samples in the track fragments.
    ///
    /// Present if the [`SampleDescriptionIndexPresent`](TfFlags::SampleDescriptionIndexPresent) flag is set.
    pub sample_description_index: Option<u32>,
    /// Indicates the default duration of the samples in the track fragments.
    ///
    /// Present if the [`DefaultSampleDurationPresent`](TfFlags::DefaultSampleDurationPresent) flag is set.
    pub default_sample_duration: Option<u32>,
    /// Indicates the default size of the samples in the track fragments.
    ///
    /// Present if the [`DefaultSampleSizePresent`](TfFlags::DefaultSampleSizePresent) flag is set.
    pub default_sample_size: Option<u32>,
    /// Indicate the default flags values for the samples in the track fragments.
    ///
    /// Present if the [`DefaultSampleFlagsPresent`](TfFlags::DefaultSampleFlagsPresent) flag is set.
    pub default_sample_flags: Option<SampleFlags>,
}

impl TrackFragmentHeaderBox {
    /// Creates a new [`TrackFragmentHeaderBox`] with the given parameters.
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
    /// Track fragment header flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TfFlags: u32 {
        /// Indicates the presence of the base-data-offset field. This
        /// provides an explicit anchor for the data offsets in each track run.
        const BaseDataOffsetPresent = 0x000001;
        /// Indicates the presence of this field, which over-rides, in
        /// this fragment, the default set up in the [`TrackExtendsBox`].
        const SampleDescriptionIndexPresent = 0x000002;
        /// Indicates the presence of the [`default_sample_duration`](TrackFragmentHeaderBox::default_sample_duration) field.
        const DefaultSampleDurationPresent = 0x000008;
        /// Indicates the presence of the [`default_sample_size`](TrackFragmentHeaderBox::default_sample_size) field.
        const DefaultSampleSizePresent = 0x000010;
        /// Indicates the presence of the [`default_sample_flags`](TrackFragmentHeaderBox::default_sample_flags) field.
        const DefaultSampleFlagsPresent = 0x000020;
        /// This indicates that the duration provided in either
        /// [`default_sample_duration`](TrackFragmentHeaderBox::default_sample_duration), or by the
        /// [`default_sample_duration`](TrackExtendsBox::default_sample_duration) in the [`TrackExtendsBox`],
        /// is empty, i.e. that there are no samples for this time interval.
        /// It is an error to make a presentation that has both edit lists in the [`MovieBox`](super::MovieBox),
        /// and empty-duration fragments.
        const DurationIsEmpty = 0x010000;
        /// If [`BaseDataOffsetPresent`](Self::BaseDataOffsetPresent) is set, this flag is ignored. Support for the
        /// [`DefaultBaseIsMoof`](Self::DefaultBaseIsMoof) is required under the 'iso5' brand,
        /// and it shall not be used in brands or compatible brands earlier than 'iso5'.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"trun", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackRunBox {
    // full header:
    /// The version of the box.
    pub version: u8,
    /// The flags of the box.
    pub flags: TrFlags,
    // body:
    /// The number of samples being added in this run; also the number of rows in the [`samples`](Self::samples) vec.
    pub sample_count: u32,
    /// Added to the implicit or explicit `data_offset` established in the track fragment header.
    pub data_offset: Option<i32>,
    /// Provides a set of flags for the first sample only of this run.
    pub first_sample_flags: Option<SampleFlags>,
    /// The samples in this run.
    pub samples: Vec<TrackRunBoxSample>,
}

impl TrackRunBox {
    /// Creates a new [`TrackRunBox`] with the given samples and optional first sample flags.
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
            sample.serialize(&mut writer, (self.version, self.flags))?;
        }

        Ok(())
    }
}

/// Sample in a [`TrackRunBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct TrackRunBoxSample {
    /// Duration of this sample.
    ///
    /// Present if the [`SampleDurationPresent`](TrFlags::SampleDurationPresent) flag is set.
    pub sample_duration: Option<u32>,
    /// Size of this sample.
    ///
    /// Present if the [`SampleSizePresent`](TrFlags::SampleSizePresent) flag is set.
    pub sample_size: Option<u32>,
    /// Flags for this sample.
    ///
    /// Present if the [`SampleFlagsPresent`](TrFlags::SampleFlagsPresent) flag is set.
    pub sample_flags: Option<SampleFlags>,
    /// Composition time offset for this sample.
    /// Either a signed or unsigned 32-bit integer.
    ///
    /// Present if the [`SampleCompositionTimeOffsetsPresent`](TrFlags::SampleCompositionTimeOffsetsPresent) flag is set.
    pub sample_composition_time_offset: Option<i64>,
}

bitflags::bitflags! {
    /// Track run flags
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct TrFlags: u32 {
        /// Indicates the presence of the [`data_offset`](TrackRunBox::data_offset) field.
        const DataOffsetPresent = 0x000001;
        /// This overrides the default flags for the first sample only,
        /// defined in [8.8.3.1](TrackExtendsBox).
        /// This makes it possible to record a group of frames where the first is a key and the
        /// rest are difference frames, without supplying explicit flags for every sample. If this flag and field
        /// are used, [`SampleFlagsPresent`](Self::SampleFlagsPresent) shall not be set.
        ///
        /// If this flag is set, the [`first_sample_flags`](TrackRunBox::first_sample_flags) field is present.
        const FirstSampleFlagsPresent = 0x000004;
        /// Indicates that each sample has its own duration, otherwise the default is used.
        ///
        /// Indicates the presence of the [`sample_duration`](TrackRunBoxSample::sample_duration) field.
        const SampleDurationPresent = 0x000100;
        /// Each sample has its own size, otherwise the default is used.
        ///
        /// Indicates the presence of the [`sample_size`](TrackRunBoxSample::sample_size) field.
        const SampleSizePresent = 0x000200;
        /// Each sample has its own flags, otherwise the default is used.
        ///
        /// Indicates the presence of the [`sample_flags`](TrackRunBoxSample::sample_flags) field.
        const SampleFlagsPresent = 0x000400;
        /// Each sample has a composition time offset.
        ///
        /// Indicates the presence of the [`sample_composition_time_offset`](TrackRunBoxSample::sample_composition_time_offset)
        /// field.
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

impl TrackRunBoxSample {
    fn serialize<W>(&self, mut writer: W, full_header: (u8, TrFlags)) -> io::Result<()>
    where
        W: io::Write,
    {
        let (version, flags) = full_header;

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
    /// Returns the size of this sample in bytes, depending on the flags.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mfra", crate_path = crate)]
pub struct MovieFragmentRandomAccessBox {
    /// The contained [`TrackFragmentRandomAccessBox`]es. (zero or one per track)
    #[iso_box(nested_box(collect))]
    pub tfra: Vec<TrackFragmentRandomAccessBox>,
    /// The contained [`MovieFragmentRandomAccessOffsetBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub mfro: MovieFragmentRandomAccessOffsetBox,
}

/// Track fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.10
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tfra", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackFragmentRandomAccessBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer providing the track identifier for which random access information is provided.
    pub track_id: u32,
    /// Indicates the length in bytes of the traf_number field minus one.
    pub length_size_of_traf_num: u8,
    /// Indicates the length in bytes of the trun_number field minus one.
    pub length_size_of_trun_num: u8,
    /// Indicates the length in bytes of the sample_number field minus one.
    pub length_size_of_sample_num: u8,
    /// An integer that gives the number of the entries for this track. If this value is zero, it
    /// indicates that every sample is a sync sample and no table entry follows.
    pub number_of_entry: u32,
    /// `time`, `moof_offset`, `traf_number`, `trun_number`, and `sample_number`.
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
                sample_delta: sample_number,
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
            entry.serialize(&mut bit_writer, self)?;
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

/// Entry in a [`TrackFragmentRandomAccessBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct TrackFragmentRandomAccessBoxEntry {
    /// An integer that indicates the presentation time of the sync sample in units defined in
    /// the [`MediaHeaderBox`](super::MediaHeaderBox) of the associated track.
    pub time: u64,
    /// An integer that gives the offset of the ['moof'](MovieFragmentBox) used in this entry. Offset is the
    /// byte-offset between the beginning of the file and the beginning of the ['moof'](MovieFragmentBox).
    pub moof_offset: u64,
    /// Indicates the ['traf'](TrackFragmentBox) number that contains the sync sample. The number ranges from 1
    /// (the first ['traf'](TrackFragmentBox) is numbered 1) in each ['moof'](MovieFragmentBox).
    pub traf_number: u32,
    /// Indicates the ['trun'](TrackRunBox) number that contains the sync sample. The number ranges from 1 in
    /// each ['traf'](TrackFragmentBox).
    pub trun_number: u32,
    /// Indicates the sample number of the sync sample. It is coded as one plus the desired sample
    /// number minus the sample number of the first sample in the [`TrackRunBox`].
    pub sample_delta: u32,
}

impl TrackFragmentRandomAccessBoxEntry {
    fn serialize<W>(&self, writer: W, parent: &TrackFragmentRandomAccessBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        if parent.full_header.version == 1 {
            self.time.serialize(&mut bit_writer)?;
            self.moof_offset.serialize(&mut bit_writer)?;
        } else {
            (self.time as u32).serialize(&mut bit_writer)?;
            (self.moof_offset as u32).serialize(&mut bit_writer)?;
        }

        bit_writer.write_bits(self.traf_number as u64, (parent.length_size_of_traf_num + 1) * 8)?;
        bit_writer.write_bits(self.trun_number as u64, (parent.length_size_of_trun_num + 1) * 8)?;
        bit_writer.write_bits(self.sample_delta as u64, (parent.length_size_of_sample_num + 1) * 8)?;

        Ok(())
    }
}

impl TrackFragmentRandomAccessBoxEntry {
    /// Returns the size of this entry in bytes, depending on the parent box.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mfro", crate_path = crate)]
pub struct MovieFragmentRandomAccessOffsetBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of bytes of the enclosing [`MovieFragmentRandomAccessBox`]
    /// box. This field is placed last in the enclosing box to assist readers scanning from the end of the file
    /// in finding the [`MovieFragmentRandomAccessBox`].
    pub parent_size: u32,
}

/// Track fragment decode time box
///
/// ISO/IEC 14496-12 - 8.8.12
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tfdt", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackFragmentBaseMediaDecodeTimeBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer equal to the sum of the decode durations of all earlier samples in the
    /// media, expressed in the media's timescale. It does not include the samples added in the enclosing
    /// track fragment.
    pub base_media_decode_time: u64,
}

impl TrackFragmentBaseMediaDecodeTimeBox {
    /// Creates a new [`TrackFragmentBaseMediaDecodeTimeBox`] with the given base media decode time.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"leva", crate_path = crate)]
pub struct LevelAssignmentBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Specifies the number of levels each fraction is grouped into. `level_count` shall be greater
    /// than or equal to 2.
    pub level_count: u8,
    /// `track_ID`, `padding_flag`, `assignment_type` and [`LevelAssignmentBoxLevelAssignmentType`].
    #[iso_box(repeated)]
    pub levels: Vec<LevelAssignmentBoxLevel>,
}

/// Level in a [`LevelAssignmentBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct LevelAssignmentBoxLevel {
    /// For loop entry `j` specifies the track identifier of the track assigned to level `j`.
    pub track_id: u32,
    /// Equal to 1 indicates that a conforming fraction can be formed by concatenating any
    /// positive integer number of levels within a fraction and padding the last [`MediaDataBox`](super::MediaDataBox)
    /// by zero bytes up to the full size that is indicated in the header of the last [`MediaDataBox`](super::MediaDataBox).
    /// When `padding_flag` is equal to 0 this is not assured.
    pub padding_flag: bool,
    /// Specifies the type of level assignment.
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

/// Type of level assignment in a [`LevelAssignmentBox`].
#[derive(Debug, PartialEq, Eq)]
pub enum LevelAssignmentBoxLevelAssignmentType {
    /// Type 0: sample groups are used to specify levels, i.e., samples mapped to different sample group
    /// description indexes of a particular sample grouping lie in different levels within the identified
    /// track; other tracks are not affected and shall have all their data in precisely one level;
    Type0 {
        /// Specifies the sample grouping used to
        /// map sample group description entries in the [`SampleGroupDescriptionBox`] to levels. Level `n`
        /// contains the samples that are mapped to the [`SampleGroupDescriptionEntry`](super::SampleGroupDescriptionEntry)
        /// having index `n` in grouping_type the [`SampleGroupDescriptionBox`] having the same values of
        /// `grouping_type` and `grouping_type_parameter`, if present, as those provided in this box.
        grouping_type: [u8; 4],
    },
    /// Type 1: as for [`Type0`](Self::Type0) except assignment is by a parameterized sample group;
    Type1 {
        /// Specifies the sample grouping used to
        /// map sample group description entries in the [`SampleGroupDescriptionBox`] to levels. Level `n`
        /// contains the samples that are mapped to the [`SampleGroupDescriptionEntry`](super::SampleGroupDescriptionEntry)
        /// having index `n` in grouping_type the [`SampleGroupDescriptionBox`] having the same values of
        /// `grouping_type` and `grouping_type_parameter`, if present, as those provided in this box.
        grouping_type: [u8; 4],
        /// Specifies the sample grouping used to
        /// map sample group description entries in the [`SampleGroupDescriptionBox`] to levels. Level `n`
        /// contains the samples that are mapped to the [`SampleGroupDescriptionEntry`](super::SampleGroupDescriptionEntry)
        /// having index `n` in grouping_type the [`SampleGroupDescriptionBox`] having the same values of
        /// `grouping_type` and `grouping_type_parameter`, if present, as those provided in this box.
        grouping_type_parameter: u32,
    },
    /// Type 2: level assignment is by track (see the [`SubsegmentIndexBox`](super::SubsegmentIndexBox)
    /// for the difference in processing of these levels)
    Type2,
    /// Type 3: level assignment is by track (see the [`SubsegmentIndexBox`](super::SubsegmentIndexBox)
    /// for the difference in processing of these levels)
    Type3,
    /// Type 4: the respective level contains the samples for a sub-track. The sub-tracks are specified through
    /// the [`SubTrackBox`](super::SubTrackBox);
    /// other tracks are not affected and shall have all their data in precisely one level;
    Type4 {
        /// Specifies that the sub-track identified by `sub_track_ID` within loop entry `j` is mapped to level `j`.
        sub_track_id: u32,
    },
    /// Other assignment type.
    Other(u8),
}

impl LevelAssignmentBoxLevelAssignmentType {
    /// Returns the assignment type as a u8.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"trep", crate_path = crate)]
pub struct TrackExtensionPropertiesBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Indicates the track for which the track extension properties are provided in this box.
    pub track_id: u32,
    /// The contained [`CompositionToDecodeBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub cslg: Option<CompositionToDecodeBox>,
    /// The contained [`AlternativeStartupSequencePropertiesBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub assp: Option<AlternativeStartupSequencePropertiesBox>,
    /// Any other boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Alternative startup sequence properties box
///
/// ISO/IEC 14496-12 - 8.8.16
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"assp", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct AlternativeStartupSequencePropertiesBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// `min_initial_alt_startup_offset` or `grouping_type_parameter` and `min_initial_alt_startup_offset`
    /// depending on the version.
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

/// Version of the [`AlternativeStartupSequencePropertiesBox`].
#[derive(Debug, PartialEq, Eq)]
pub enum AlternativeStartupSequencePropertiesBoxVersion {
    /// Version 0.
    Version0 {
        /// No value of `sample_offset`(3GPP) of the referred sample group
        /// description entries of the alternative startup sequence sample grouping shall be smaller than
        /// min_initial_alt_startup_offset. In version 0 of this box, the alternative startup sequence sample
        /// grouping using version 0 of the Sample to Group box is referred to. In version 1 of this box, the
        /// alternative startup sequence sample grouping using version 1 of the [`SampleToGroupBox`] is referred
        /// to as further constrained by `grouping_type_parameter`.
        min_initial_alt_startup_offset: i32,
    },
    /// Version 1.
    Version1 {
        /// Indicates the number of alternative startup sequence sample groupings documented in this box.
        num_entries: u32,
        /// `grouping_type_parameter` and `min_initial_alt_startup_offset`.
        entries: Vec<AlternativeStartupSequencePropertiesBoxVersion1Entry>,
    },
    /// Any other version.
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

/// Entry in a [`AlternativeStartupSequencePropertiesBox`] version 1.
///
/// See [`AlternativeStartupSequencePropertiesBoxVersion`].
#[derive(Debug, PartialEq, Eq)]
pub struct AlternativeStartupSequencePropertiesBoxVersion1Entry {
    /// Indicates which one of the alternative sample groupings this loop entry applies to.
    pub grouping_type_parameter: u32,
    /// No value of `sample_offset`(3GPP) of the referred sample group
    /// description entries of the alternative startup sequence sample grouping shall be smaller than
    /// min_initial_alt_startup_offset. In version 0 of this box, the alternative startup sequence sample
    /// grouping using version 0 of the Sample to Group box is referred to. In version 1 of this box, the
    /// alternative startup sequence sample grouping using version 1 of the [`SampleToGroupBox`] is referred
    /// to as further constrained by `grouping_type_parameter`.
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
