use std::fmt::Debug;
use std::{io, iter};

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be, ZeroCopyReader};
use scuffle_bytes_util::{BytesCow, IoResultExt};

use crate::{BoxHeader, BoxHeaderProperties, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// Data information box
///
/// ISO/IEC 14496-12 - 8.7.1
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"dinf", crate_path = crate)]
pub struct DataInformationBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub dref: DataReferenceBox<'a>,
}

/// Data entry url box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"url ", crate_path = crate)]
pub struct DataEntryUrlBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub location: Utf8String,
}

/// Data entry urn box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"urn ", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct DataEntryUrnBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub name: Utf8String,
    pub location: Option<Utf8String>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for DataEntryUrnBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let name = Utf8String::deserialize(&mut reader)?;
        let location = Utf8String::deserialize(&mut reader).eof_to_none()?;

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"imdt", crate_path = crate)]
pub struct DataEntryImdaBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub imda_ref_identifier: u32,
}

/// Data entry sequence number imda box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"snim", crate_path = crate)]
pub struct DataEntrySeqNumImdaBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
}

/// Data reference box
///
/// ISO/IEC 14496-12 - 8.7.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"dref", crate_path = crate)]
pub struct DataReferenceBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(nested_box(collect))]
    pub url: Vec<DataEntryUrlBox>,
    #[iso_box(nested_box(collect))]
    pub urn: Vec<DataEntryUrnBox>,
    #[iso_box(nested_box(collect))]
    pub imda: Vec<DataEntryImdaBox>,
    #[iso_box(nested_box(collect))]
    pub snim: Vec<DataEntrySeqNumImdaBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Sample size box
///
/// ISO/IEC 14496-12 - 8.7.3.2
#[derive(IsoBox)]
#[iso_box(box_type = b"stsz", crate_path = crate)]
pub struct SampleSizeBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub sample_size: u32,
    pub sample_count: u32,
    #[iso_box(repeated)]
    pub entry_size: Vec<u32>,
}

impl Debug for SampleSizeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleSizeBox")
            .field("header", &self.header)
            .field("sample_size", &self.sample_size)
            .field("sample_count", &self.sample_count)
            .field("entry_size.len", &self.entry_size.len())
            .finish()
    }
}

/// Compact sample size box
///
/// ISO/IEC 14496-12 - 8.7.3.3
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"stz2", crate_path = crate)]
pub struct CompactSampleSizeBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub reserved: U24Be,
    pub field_size: u8,
    pub sample_count: u32,
    pub entry_size: BytesCow<'a>,
}

/// Sample to chunk box
///
/// ISO/IEC 14496-12 - 8.7.4
#[derive(IsoBox)]
#[iso_box(box_type = b"stsc", crate_path = crate)]
pub struct SampleToChunkBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub entries: Vec<SampleToChunkBoxEntry>,
}

impl Debug for SampleToChunkBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleToChunkBox")
            .field("header", &self.header)
            .field("entry_count", &self.entry_count)
            .field("entries.len", &self.entries.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct SampleToChunkBoxEntry {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
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

/// Chunk offset box
///
/// ISO/IEC 14496-12 - 8.7.5
#[derive(IsoBox)]
#[iso_box(box_type = b"stco", crate_path = crate)]
pub struct ChunkOffsetBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub chunk_offset: Vec<u32>,
}

impl Debug for ChunkOffsetBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkOffsetBox")
            .field("header", &self.header)
            .field("entry_count", &self.entry_count)
            .field("chunk_offset.len", &self.chunk_offset.len())
            .finish()
    }
}

/// Chunk large offset box
///
/// ISO/IEC 14496-12 - 8.7.5
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"co64", crate_path = crate)]
pub struct ChunkLargeOffsetBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub chunk_offset: Vec<u64>,
}

/// Padding bits box
///
/// ISO/IEC 14496-12 - 8.7.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"padb", crate_path = crate)]
pub struct PaddingBitsBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub sample_count: u32,
    #[iso_box(from = "u8", repeated)]
    pub entry: Vec<PaddingBitsBoxEntry>,
}

#[derive(Debug, Clone, Copy)]
pub struct PaddingBitsBoxEntry {
    pub pad1: u8,
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

/// Sub-sample information box
///
/// ISO/IEC 14496-12 - 8.7.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"subs", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct SubSampleInformationBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<SubSampleInformationBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SubSampleInformationBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let entry_count = u32::deserialize(&mut reader)?;

        let entries = {
            if let Some(payload_size) = seed.payload_size() {
                let mut payload_reader = reader.take(payload_size);
                iter::from_fn(|| {
                    SubSampleInformationBoxEntry::deserialize_seed(&mut payload_reader, seed.version)
                        .eof_to_none()
                        .transpose()
                })
                .collect::<Result<Vec<SubSampleInformationBoxEntry>, io::Error>>()?
            } else {
                iter::from_fn(|| {
                    SubSampleInformationBoxEntry::deserialize_seed(&mut reader, seed.version)
                        .eof_to_none()
                        .transpose()
                })
                .collect::<Result<Vec<SubSampleInformationBoxEntry>, io::Error>>()?
            }
        };

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.sample_delta.serialize(&mut writer)?;

            entry.subsample_count.serialize(&mut writer)?;
            for subsample in &entry.subsample_info {
                if self.header.version == 1 {
                    subsample.subsample_size.serialize(&mut writer)?;
                } else {
                    (subsample.subsample_size as u16).serialize(&mut writer)?;
                }
                subsample.subsample_priority.serialize(&mut writer)?;
                subsample.discardable.serialize(&mut writer)?;
                subsample.codec_specific_parameters.serialize(&mut writer)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SubSampleInformationBoxEntry {
    pub sample_delta: u32,
    pub subsample_count: u16,
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

#[derive(Debug)]
pub struct SubSampleInformationBoxEntrySubSample {
    pub subsample_size: u32,
    pub subsample_priority: u8,
    pub discardable: u8,
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

/// Sample auxiliary information sizes box
///
/// ISO/IEC 14496-12 - 8.7.8
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"saiz", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct SampleAuxiliaryInformationSizesBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub aux_info_type: Option<u32>,
    pub aux_info_type_parameter: Option<u32>,
    pub default_sample_info_size: u8,
    pub sample_count: u32,
    pub sample_info_size: Option<BytesCow<'a>>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SampleAuxiliaryInformationSizesBox<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let aux_info_type = if (*seed.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let aux_info_type_parameter = if (*seed.flags & 0b1) == 1 {
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
            header: seed,
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
        self.header.serialize(&mut writer)?;

        if (*self.header.flags & 0b1) == 1 {
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"saio", skip_impl(deserialize_seed, serialize), crate_path = crate)]
pub struct SampleAuxiliaryInformationOffsetsBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub aux_info_type: Option<u32>,
    pub aux_info_type_parameter: Option<u32>,
    pub entry_count: u32,
    pub offset: Vec<u64>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SampleAuxiliaryInformationOffsetsBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let aux_info_type = if (*seed.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let aux_info_type_parameter = if (*seed.flags & 0b1) == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let entry_count = u32::deserialize(&mut reader)?;

        let offset = if seed.version == 0 {
            (0..entry_count)
                .map(|_| u32::deserialize(&mut reader).map(|v| v as u64))
                .collect::<Result<Vec<u64>, io::Error>>()?
        } else {
            (0..entry_count)
                .map(|_| u64::deserialize(&mut reader))
                .collect::<Result<Vec<u64>, io::Error>>()?
        };

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;

        if (*self.header.flags & 0b1) == 1 {
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
            if self.header.version == 0 {
                (*entry as u32).serialize(&mut writer)?;
            } else {
                entry.serialize(&mut writer)?;
            }
        }

        Ok(())
    }
}
