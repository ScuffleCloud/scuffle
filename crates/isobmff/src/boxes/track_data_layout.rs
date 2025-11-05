use std::fmt::Debug;
use std::{io, iter};

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be, ZeroCopyReader};
use scuffle_bytes_util::{BytesCow, IoResultExt};

use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8String};

/// Data information box
///
/// ISO/IEC 14496-12 - 8.7.1
#[derive(IsoBox, Debug, PartialEq, Eq, Default)]
#[iso_box(box_type = b"dinf", crate_path = crate)]
pub struct DataInformationBox<'a> {
    /// The contained [`DataReferenceBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub dref: DataReferenceBox<'a>,
}

/// Data entry url box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(IsoBox, Debug, PartialEq, Eq, Default)]
#[iso_box(box_type = b"url ", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct DataEntryUrlBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A URL, and is required in a URL entry and optional in a URN entry, where it gives a location
    /// to find the resource with the given name. The URL type should be of a service that delivers a file
    /// (e.g. URLs of type file, http, ftp etc.), and which services ideally also permit random access. Relative
    /// URLs are permissible and are relative to the file that contains this data reference.
    ///
    /// The official spec says that this field is mandatory but there are files that don't have it (e.g. `assets/avc_aac.mp4`).
    pub location: Option<Utf8String>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for DataEntryUrlBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let location = Utf8String::deserialize(&mut reader).eof_to_none()?;

        Ok(Self { full_header, location })
    }
}

impl Serialize for DataEntryUrlBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        if let Some(location) = &self.location {
            location.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Data entry urn box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"urn ", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct DataEntryUrnBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A URN.
    pub name: Utf8String,
    /// A URL, and is required in a URL entry and optional in a URN entry, where it gives a location
    /// to find the resource with the given name. The URL type should be of a service that delivers a file
    /// (e.g. URLs of type file, http, ftp etc.), and which services ideally also permit random access. Relative
    /// URLs are permissible and are relative to the file that contains this data reference.
    pub location: Option<Utf8String>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for DataEntryUrnBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let name = Utf8String::deserialize(&mut reader)?;
        let location = Utf8String::deserialize(&mut reader).eof_to_none()?;

        Ok(Self {
            full_header,
            name,
            location,
        })
    }
}

impl Serialize for DataEntryUrnBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.name.serialize(&mut writer)?;
        if let Some(location) = &self.location {
            location.serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Data entry imda box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"imdt", crate_path = crate)]
pub struct DataEntryImdaBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Identifies the [`IdentifiedMediaDataBox`](super::IdentifiedMediaDataBox) containing the
    /// media data accessed through the `data_reference_index` corresponding to this [`DataEntryImdaBox`].
    /// The referred [`IdentifiedMediaDataBox`](super::IdentifiedMediaDataBox) contains `imda_identifier`
    /// that is equal to `imda_ref_identifier`.
    pub imda_ref_identifier: u32,
}

/// Data entry sequence number imda box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"snim", crate_path = crate)]
pub struct DataEntrySeqNumImdaBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
}

/// Data reference box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"dref", crate_path = crate)]
pub struct DataReferenceBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that counts the sub boxes.
    pub entry_count: u32,
    /// Data entry URL boxes.
    #[iso_box(nested_box(collect))]
    pub url: Vec<DataEntryUrlBox>,
    /// Data entry URN boxes.
    #[iso_box(nested_box(collect))]
    pub urn: Vec<DataEntryUrnBox>,
    /// Data entry IMDA boxes.
    #[iso_box(nested_box(collect))]
    pub imda: Vec<DataEntryImdaBox>,
    /// Data entry sequence number IMDA boxes.
    #[iso_box(nested_box(collect))]
    pub snim: Vec<DataEntrySeqNumImdaBox>,
    /// Any other unknown boxes.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

impl Default for DataReferenceBox<'_> {
    fn default() -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            entry_count: 1,
            url: vec![DataEntryUrlBox::default()],
            urn: vec![],
            imda: vec![],
            snim: vec![],
            unknown_boxes: vec![],
        }
    }
}

/// Sample size box
///
/// ISO/IEC 14496-12 - 8.7.3.2
#[derive(IsoBox, PartialEq, Eq, Default)]
#[iso_box(box_type = b"stsz", crate_path = crate)]
pub struct SampleSizeBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer specifying the default sample size. If all the samples are the same size, this field
    /// contains that size value. If this field is set to 0, then the samples have different sizes, and those sizes
    /// are stored in the sample size table. If this field is not 0, it specifies the constant sample size, and no
    /// array follows.
    pub sample_size: u32,
    /// An integer that gives the number of samples in the track; if sample-size is 0, then it is
    /// also the number of entries in the [`entry_size`](SampleSizeBox::entry_size) vec.
    pub sample_count: u32,
    /// Integers specifying the size of a sample, indexed by its number.
    #[iso_box(repeated)]
    pub entry_size: Vec<u32>,
}

impl Debug for SampleSizeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleSizeBox")
            .field("full_header", &self.full_header)
            .field("sample_size", &self.sample_size)
            .field("sample_count", &self.sample_count)
            .field("entry_size.len", &self.entry_size.len())
            .finish()
    }
}

/// Compact sample size box
///
/// ISO/IEC 14496-12 - 8.7.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stz2", crate_path = crate)]
pub struct CompactSampleSizeBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Reserved 24 bits, must be 0.
    pub reserved: U24Be,
    /// An integer specifying the size in bits of the entries in the following table; it shall take the
    /// value 4, 8 or 16. If the value 4 is used, then each byte contains two values: `entry[i]<<4 + entry[i+1]`;
    /// if the sizes do not fill an integral number of bytes, the last byte is padded with zeros.
    pub field_size: u8,
    /// An integer that gives the number of entries in the [`entry_size`](Self::entry_size) vec.
    pub sample_count: u32,
    /// Integers specifying the size of a sample, indexed by its number.
    pub entry_size: BytesCow<'a>,
}

/// Sample to chunk box
///
/// ISO/IEC 14496-12 - 8.7.4
#[derive(IsoBox, PartialEq, Eq, Default)]
#[iso_box(box_type = b"stsc", crate_path = crate)]
pub struct SampleToChunkBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// `first_chunk`, `samples_per_chunk` and `sample_description_index`.
    #[iso_box(repeated)]
    pub entries: Vec<SampleToChunkBoxEntry>,
}

impl Debug for SampleToChunkBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleToChunkBox")
            .field("full_header", &self.full_header)
            .field("entry_count", &self.entry_count)
            .field("entries.len", &self.entries.len())
            .finish()
    }
}

/// Entry in [`SampleToChunkBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct SampleToChunkBoxEntry {
    /// An integer that gives the index of the first chunk in this run of chunks that share the
    /// same samples-per-chunk and sample-description-index; the index of the first chunk in a track has
    /// the value 1 (the `first_chunk` field in the first record of this box has the value 1, identifying that the
    /// first sample maps to the first chunk).
    pub first_chunk: u32,
    /// An integer that gives the number of samples in each of these chunks.
    pub samples_per_chunk: u32,
    /// An integer that gives the index of the sample entry that describes
    /// the samples in this chunk. The index ranges from 1 to the number of sample entries in the
    /// [`SampleDescriptionBox`](super::SampleDescriptionBox).
    pub sample_description_index: u32,
}

impl<'a> Deserialize<'a> for SampleToChunkBoxEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            first_chunk: u32::deserialize(&mut reader)?,
            samples_per_chunk: u32::deserialize(&mut reader)?,
            sample_description_index: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for SampleToChunkBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.first_chunk.serialize(&mut writer)?;
        self.samples_per_chunk.serialize(&mut writer)?;
        self.sample_description_index.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for SampleToChunkBoxEntry {
    fn size(&self) -> usize {
        4 + 4 + 4 // 3 u32s
    }
}

/// Chunk offset box
///
/// ISO/IEC 14496-12 - 8.7.5
#[derive(IsoBox, PartialEq, Eq, Default)]
#[iso_box(box_type = b"stco", crate_path = crate)]
pub struct ChunkOffsetBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`chunk_offset`](Self::chunk_offset) vec.
    pub entry_count: u32,
    /// Integers that give the offset of the start of a chunk. If the referenced
    /// data reference entry is [`DataEntryImdaBox`] or [`DataEntrySeqNumImdaBox`],
    /// the value of `chunk_offset` is relative to the first byte of the payload of the
    /// [`IdentifiedMediaDataBox`](super::IdentifiedMediaDataBox) corresponding to the data reference entry.
    /// Otherwise, the value of `chunk_offset` is relative to the start of the containing media file.
    #[iso_box(repeated)]
    pub chunk_offset: Vec<u32>,
}

impl Debug for ChunkOffsetBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkOffsetBox")
            .field("full_header", &self.full_header)
            .field("entry_count", &self.entry_count)
            .field("chunk_offset.len", &self.chunk_offset.len())
            .finish()
    }
}

/// Chunk large offset box
///
/// ISO/IEC 14496-12 - 8.7.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"co64", crate_path = crate)]
pub struct ChunkLargeOffsetBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`chunk_offset`](Self::chunk_offset) vec.
    pub entry_count: u32,
    /// Integers that give the offset of the start of a chunk. If the referenced
    /// data reference entry is [`DataEntryImdaBox`] or [`DataEntrySeqNumImdaBox`],
    /// the value of `chunk_offset` is relative to the first byte of the payload of the
    /// [`IdentifiedMediaDataBox`](super::IdentifiedMediaDataBox) corresponding to the data reference entry.
    /// Otherwise, the value of `chunk_offset` is relative to the start of the containing media file.
    #[iso_box(repeated)]
    pub chunk_offset: Vec<u64>,
}

/// Padding bits box
///
/// ISO/IEC 14496-12 - 8.7.6
#[derive(IsoBox, PartialEq, Eq)]
#[iso_box(box_type = b"padb", crate_path = crate)]
pub struct PaddingBitsBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Counts the number of samples in the track; it should match the count in other tables.
    pub sample_count: u32,
    /// `pad1` and `pad2`
    #[iso_box(from = "u8", repeated)]
    pub entry: Vec<PaddingBitsBoxEntry>,
}

impl Debug for PaddingBitsBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaddingBitsBox")
            .field("full_header", &self.full_header)
            .field("sample_count", &self.sample_count)
            .field("entry.len", &self.entry.len())
            .finish()
    }
}

/// Entry in [`PaddingBitsBox`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PaddingBitsBoxEntry {
    /// A value from 0 to 7, indicating the number of padding bits at the end of sample `(i*2)+1`.
    pub pad1: u8,
    /// A value from 0 to 7, indicating the number of padding bits at the end of sample `(i*2)+2`.
    pub pad2: u8,
}

impl From<u8> for PaddingBitsBoxEntry {
    fn from(value: u8) -> Self {
        // 0xxx 0xxx
        Self {
            pad1: (value >> 4) & 0b0111,
            pad2: value & 0b0111,
        }
    }
}

impl From<PaddingBitsBoxEntry> for u8 {
    fn from(value: PaddingBitsBoxEntry) -> Self {
        (value.pad1 << 4) | value.pad2
    }
}

impl IsoSized for PaddingBitsBoxEntry {
    fn size(&self) -> usize {
        1
    }
}

/// Sub-sample information box
///
/// ISO/IEC 14496-12 - 8.7.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"subs", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SubSampleInformationBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// `sample_delta`, `subsample_count` and subsample information.
    pub entries: Vec<SubSampleInformationBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SubSampleInformationBox {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let entry_count = u32::deserialize(&mut reader)?;

        let entries = {
            if let Some(payload_size) = seed.size.size() {
                let mut payload_reader = reader.take(payload_size);
                iter::from_fn(|| {
                    SubSampleInformationBoxEntry::deserialize_seed(&mut payload_reader, full_header.version)
                        .eof_to_none()
                        .transpose()
                })
                .collect::<Result<Vec<SubSampleInformationBoxEntry>, io::Error>>()?
            } else {
                iter::from_fn(|| {
                    SubSampleInformationBoxEntry::deserialize_seed(&mut reader, full_header.version)
                        .eof_to_none()
                        .transpose()
                })
                .collect::<Result<Vec<SubSampleInformationBoxEntry>, io::Error>>()?
            }
        };

        Ok(Self {
            full_header,
            entry_count,
            entries,
        })
    }
}

impl Serialize for SubSampleInformationBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize(&mut writer, self.full_header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for SubSampleInformationBox {
    fn size(&self) -> usize {
        let mut size = 0;
        size += self.full_header.size();
        size += 4;
        size += self.entries.iter().map(|e| e.size(self.full_header.version)).sum::<usize>();

        Self::add_header_size(size)
    }
}

/// Entry in [`SubSampleInformationBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct SubSampleInformationBoxEntry {
    /// An integer that indicates the sample having sub‐sample structure. It is coded as the
    /// difference, in decoding order, between the desired sample number, and the sample number
    /// indicated in the previous entry. If the current entry is the first entry in the track, the value
    /// indicates the sample number of the first sample having sub-sample information, that is, the value
    /// is the difference between the sample number and zero (0). If the current entry is the first entry
    /// in a track fragment with preceding non-empty track fragments, the value indicates the difference
    /// between the sample number of the first sample having sub-sample information and the sample
    /// number of the last sample in the previous track fragment. If the current entry is the first entry in
    /// a track fragment without any preceding track fragments, the value indicates the sample number
    /// of the first sample having sub-sample information, that is, the value is the difference between the
    /// sample number and zero (0). This implies that the `sample_delta` for the first entry describing the
    /// first sample in the track or in the track fragment is always 1.
    pub sample_delta: u32,
    /// An integer that specifies the number of sub-sample for the current sample. If there
    /// is no sub-sample structure, then this field takes the value 0.
    pub subsample_count: u16,
    /// `subsample_size`, `subsample_priority`, `discardable` and `codec_specific_parameters`.
    pub subsample_info: Vec<SubSampleInformationBoxEntrySubSample>,
}

impl<'a> DeserializeSeed<'a, u8> for SubSampleInformationBoxEntry {
    fn deserialize_seed<R: ZeroCopyReader<'a>>(mut reader: R, seed: u8) -> io::Result<Self> {
        let sample_delta = u32::deserialize(&mut reader)?;
        let subsample_count = u16::deserialize(&mut reader)?;

        let mut subsample_info = Vec::with_capacity(subsample_count as usize);
        for _ in 0..subsample_count {
            subsample_info.push(SubSampleInformationBoxEntrySubSample::deserialize_seed(&mut reader, seed)?);
        }

        Ok(Self {
            sample_delta,
            subsample_count,
            subsample_info,
        })
    }
}

impl SubSampleInformationBoxEntry {
    fn serialize<W>(&self, mut writer: W, version: u8) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_delta.serialize(&mut writer)?;

        self.subsample_count.serialize(&mut writer)?;
        for subsample in &self.subsample_info {
            subsample.serialize(&mut writer, version)?;
        }

        Ok(())
    }
}

impl SubSampleInformationBoxEntry {
    /// Returns the size of this entry in bytes, depending on the version.
    pub fn size(&self, version: u8) -> usize {
        4 + 2 + self.subsample_info.iter().map(|s| s.size(version)).sum::<usize>()
    }
}

/// Sub-sample information in a [`SubSampleInformationBoxEntry`].
#[derive(Debug, PartialEq, Eq)]
pub struct SubSampleInformationBoxEntrySubSample {
    /// An integer that specifies the size, in bytes, of the current sub-sample.
    pub subsample_size: u32,
    /// An integer specifying the degradation priority for each sub-sample. Higher
    /// values of subsample_priority, indicate sub-samples which are important to, and have a greater
    /// impact on, the decoded quality.
    pub subsample_priority: u8,
    /// Equal to 0 means that the sub-sample is required to decode the current sample, while
    /// equal to 1 means the sub-sample is not required to decode the current sample but may be used
    /// for enhancements, e.g., the sub-sample consists of supplemental enhancement information (SEI)
    /// messages.
    pub discardable: u8,
    /// Defined by the codec in use. If no such definition is available, this field
    /// shall be set to 0.
    pub codec_specific_parameters: u32,
}

impl<'a> DeserializeSeed<'a, u8> for SubSampleInformationBoxEntrySubSample {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let subsample_size = if seed == 1 {
            u32::deserialize(&mut reader)?
        } else {
            u16::deserialize(&mut reader)? as u32
        };
        let subsample_priority = u8::deserialize(&mut reader)?;
        let discardable = u8::deserialize(&mut reader)?;
        let codec_specific_parameters = u32::deserialize(&mut reader)?;

        Ok(Self {
            subsample_size,
            subsample_priority,
            discardable,
            codec_specific_parameters,
        })
    }
}

impl SubSampleInformationBoxEntrySubSample {
    fn serialize<W>(&self, mut writer: W, version: u8) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if version == 1 {
            self.subsample_size.serialize(&mut writer)?;
        } else {
            (self.subsample_size as u16).serialize(&mut writer)?;
        }
        self.subsample_priority.serialize(&mut writer)?;
        self.discardable.serialize(&mut writer)?;
        self.codec_specific_parameters.serialize(&mut writer)?;

        Ok(())
    }
}

impl SubSampleInformationBoxEntrySubSample {
    /// Returns the size of this entry in bytes, depending on the version.
    pub fn size(&self, version: u8) -> usize {
        if version == 1 { 4 + 1 + 1 + 4 } else { 2 + 1 + 1 + 4 }
    }
}

/// Sample auxiliary information sizes box
///
/// ISO/IEC 14496-12 - 8.7.8
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"saiz", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct SampleAuxiliaryInformationSizesBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that identifies the type of the sample auxiliary information. At most one
    /// occurrence of this box with the same values for `aux_info_type` and `aux_info_type_parameter` shall
    /// exist in the containing box.
    pub aux_info_type: Option<u32>,
    /// Identifies the "stream" of auxiliary information having the same value of
    /// `aux_info_type` and associated to the same track. The semantics of `aux_info_type_parameter` are
    /// determined by the value of `aux_info_type`.
    pub aux_info_type_parameter: Option<u32>,
    /// An integer specifying the sample auxiliary information size for the case
    /// where all the indicated samples have the same sample auxiliary information size. If the size varies
    /// then this field shall be zero.
    pub default_sample_info_size: u8,
    /// An integer that gives the number of samples for which a size is defined.
    ///
    /// For a [`SampleAuxiliaryInformationSizesBox`] appearing in the [`SampleTableBox`](super::SampleTableBox)
    /// this shall be the same as, or less than, the `sample_count` within the [`SampleSizeBox`] or [`CompactSampleSizeBox`].
    ///
    /// For a [`SampleAuxiliaryInformationSizesBox`] appearing in a [`TrackFragmentBox`](super::TrackFragmentBox) this
    /// shall be the same as, or less than, the sum of the `sample_count` entries within the
    /// [`TrackRunBox`](super::TrackRunBox)es of the track fragment.
    ///
    /// If this is less than the number of samples, then auxiliary information is supplied for the initial samples, and the
    /// remaining samples have no associated auxiliary information.
    pub sample_count: u32,
    /// Gives the size of the sample auxiliary information in bytes. This may be zero to
    /// indicate samples with no associated auxiliary information.
    ///
    /// If set, length is [`sample_count`](Self::sample_count).
    pub sample_info_size: Option<BytesCow<'a>>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SampleAuxiliaryInformationSizesBox<'a> {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let aux_info_type = if (*full_header.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let aux_info_type_parameter = if (*full_header.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let default_sample_info_size = u8::deserialize(&mut reader)?;
        let sample_count = u32::deserialize(&mut reader)?;

        let sample_info_size = if default_sample_info_size == 0 {
            Some(reader.try_read(sample_count as usize)?)
        } else {
            None
        };

        Ok(Self {
            full_header,
            aux_info_type,
            aux_info_type_parameter,
            default_sample_info_size,
            sample_count,
            sample_info_size,
        })
    }
}

impl Serialize for SampleAuxiliaryInformationSizesBox<'_> {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        if (*self.full_header.flags & 0b1) == 1 {
            self.aux_info_type
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "aux_info_type is required"))?
                .serialize(&mut writer)?;
            self.aux_info_type_parameter
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "aux_info_type_parameter is required",
                ))?
                .serialize(&mut writer)?;
        }

        self.default_sample_info_size.serialize(&mut writer)?;
        self.sample_count.serialize(&mut writer)?;
        if self.default_sample_info_size == 0 {
            self.sample_info_size
                .as_ref()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "sample_info_size is required"))?
                .serialize(&mut writer)?;
        }

        Ok(())
    }
}

/// Sample auxiliary information offsets box
///
/// ISO/IEC 14496-12 - 8.7.9
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"saio", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SampleAuxiliaryInformationOffsetsBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that identifies the type of the sample auxiliary information. At most one
    /// occurrence of this box with the same values for `aux_info_type` and `aux_info_type_parameter` shall
    /// exist in the containing box.
    pub aux_info_type: Option<u32>,
    /// Identifies the "stream" of auxiliary information having the same value of
    /// `aux_info_type` and associated to the same track. The semantics of `aux_info_type_parameter` are
    /// determined by the value of `aux_info_type`.
    pub aux_info_type_parameter: Option<u32>,
    /// Gives the number of entries in the following table.
    ///
    /// For a [`SampleAuxiliaryInformationOffsetsBox`] appearing in a [`SampleTableBox`](super::SampleTableBox),
    /// this shall be equal to one or to the value of the `entry_count`
    /// field in the [`ChunkOffsetBox`] or [`ChunkLargeOffsetBox`].
    ///
    /// For a [`SampleAuxiliaryInformationOffsetsBox`] appearing in a [`TrackFragmentBox`](super::TrackFragmentBox),
    /// this shall be equal to one or to the number of TrackRunBoxes in the [`TrackFragmentBox`](super::TrackFragmentBox).
    pub entry_count: u32,
    /// Gives the position in the file of the Sample Auxiliary Information for each Chunk or Track
    /// Fragment Run. If `entry_count` is one, then the Sample Auxiliary Information for all Chunks or Runs
    /// is contiguous in the file in chunk or run order. When in the [`SampleTableBox`](super::SampleTableBox),
    /// the offsets are relative to the same base offset as derived for the respective samples through the
    /// `data_reference_index` of the sample entry referenced by the samples.
    /// In a [`TrackFragmentBox`](super::TrackFragmentBox), this value is relative to the base offset established by the
    /// [`TrackFragmentHeaderBox`](super::TrackFragmentHeaderBox) in the same track fragment (see 8.8.14).
    pub offset: Vec<u64>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SampleAuxiliaryInformationOffsetsBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let aux_info_type = if (*full_header.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let aux_info_type_parameter = if (*full_header.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let entry_count = u32::deserialize(&mut reader)?;

        let offset = if full_header.version == 0 {
            (0..entry_count)
                .map(|_| u32::deserialize(&mut reader).map(|v| v as u64))
                .collect::<Result<Vec<u64>, io::Error>>()?
        } else {
            (0..entry_count)
                .map(|_| u64::deserialize(&mut reader))
                .collect::<Result<Vec<u64>, io::Error>>()?
        };

        Ok(Self {
            full_header,
            aux_info_type,
            aux_info_type_parameter,
            entry_count,
            offset,
        })
    }
}

impl Serialize for SampleAuxiliaryInformationOffsetsBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        if (*self.full_header.flags & 0b1) == 1 {
            self.aux_info_type
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "aux_info_type is required"))?
                .serialize(&mut writer)?;
            self.aux_info_type_parameter
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "aux_info_type_parameter is required",
                ))?
                .serialize(&mut writer)?;
        }

        self.entry_count.serialize(&mut writer)?;
        for entry in &self.offset {
            if self.full_header.version == 0 {
                (*entry as u32).serialize(&mut writer)?;
            } else {
                entry.serialize(&mut writer)?;
            }
        }

        Ok(())
    }
}

impl IsoSized for SampleAuxiliaryInformationOffsetsBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        if (*self.full_header.flags & 0b1) == 1 {
            size += 4; // aux_info_type
            size += 4; // aux_info_type_parameter
        }
        size += 4; // entry_count
        size += if self.full_header.version == 0 {
            4 * self.offset.len() // u32
        } else {
            8 * self.offset.len() // u64
        };

        Self::add_header_size(size)
    }
}
