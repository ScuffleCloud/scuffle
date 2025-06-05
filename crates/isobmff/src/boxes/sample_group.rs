use std::io;

use scuffle_bytes_util::BitWriter;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be, ZeroCopyReader};

use super::SampleGroupDescriptionEntry;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Sample to group box
///
/// ISO/IEC 14496-12 - 8.9.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sbgp", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SampleToGroupBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that identifies the type (i.e. criterion used to form the sample groups)
    /// of the sample grouping and links it to its sample group description table with the same value for
    /// `grouping_type`. At most one occurrence of this box with the same value for `grouping_type` (and,
    /// if used, `grouping_type_parameter`) shall exist for a track.
    pub grouping_type: [u8; 4],
    /// An indication of the sub-type of the grouping.
    pub grouping_type_parameter: Option<u32>,
    /// An integer that gives the number of entries in the following table.
    pub entry_count: u32,
    /// `sample_count` and `group_description_index`
    pub entries: Vec<SampleToGroupBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SampleToGroupBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let grouping_type = <[u8; 4]>::deserialize(&mut reader)?;
        let grouping_type_parameter = if full_header.version == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let entry_count = u32::deserialize(&mut reader)?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(SampleToGroupBoxEntry::deserialize(&mut reader)?);
        }

        Ok(Self {
            full_header,
            grouping_type,
            grouping_type_parameter,
            entry_count,
            entries,
        })
    }
}

impl Serialize for SampleToGroupBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        self.grouping_type.serialize(&mut writer)?;
        if self.full_header.version == 1 {
            self.grouping_type_parameter
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "grouping_type_parameter is required",
                ))?
                .serialize(&mut writer)?;
        }

        self.entry_count.serialize(&mut writer)?;
        for entry in &self.entries {
            entry.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for SampleToGroupBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        size += 4; // grouping_type
        if self.full_header.version == 1 {
            size += 4; // grouping_type_parameter
        }
        size += 4; // entry_count
        size += self.entries.size();

        Self::add_header_size(size)
    }
}

/// Entry in [`SampleToGroupBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct SampleToGroupBoxEntry {
    /// An integer that gives the number of consecutive samples with the same sample group
    /// descriptor. It is an error for the total in this box to be greater than the `sample_count` documented
    /// elsewhere, and the reader behaviour would then be undefined. If the sum of the sample count in
    /// this box is less than the total sample count, or there is no [`SampleToGroupBox`] that applies to some
    /// samples (e.g. it is absent from a track fragment), then those samples are associated with the group
    /// identified by the `default_group_description_index` in the [`SampleGroupDescriptionBox`], if any, or
    /// else with no group.
    pub sample_count: u32,
    /// An integer that gives the index of the sample group entry which describes
    /// the samples in this group. The index ranges from 1 to the number of sample group entries in the
    /// [`SampleGroupDescriptionBox`], or takes the value 0 to indicate that this sample is a member of no
    /// group of this type.
    pub group_description_index: u32,
}

impl<'a> Deserialize<'a> for SampleToGroupBoxEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            sample_count: u32::deserialize(&mut reader)?,
            group_description_index: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for SampleToGroupBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_count.serialize(&mut writer)?;
        self.group_description_index.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for SampleToGroupBoxEntry {
    fn size(&self) -> usize {
        4 + 4 // sample_count + group_description_index
    }
}

/// Sample group description box
///
/// ISO/IEC 14496-12 - 8.9.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sgpd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SampleGroupDescriptionBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that identifies the [`SampleToGroupBox`] that is associated with this sample group description.
    pub grouping_type: [u8; 4],
    /// Indicates the length of every group entry (if the length is constant), or zero (0) if it is variable.
    pub default_length: Option<u32>,
    /// Specifies the index of the sample group description entry which
    /// applies to all samples in the track for which no sample to group mapping is provided through a
    /// [`SampleToGroupBox`]. The default value of this field is zero (indicating that the samples are mapped to
    /// no group description of this type).
    pub default_group_description_index: Option<u32>,
    /// An integer that gives the number of entries in the following table.
    pub entry_count: u32,
    /// The contained sample group description entries.
    pub entries: Vec<SampleGroupDescriptionEntry<'a>>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for SampleGroupDescriptionBox<'a> {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let grouping_type = <[u8; 4]>::deserialize(&mut reader)?;
        let default_length = if full_header.version >= 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_group_description_index = if full_header.version >= 2 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let entry_count = u32::deserialize(&mut reader)?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let description_length = if default_length.is_some_and(|l| l == 0) {
                Some(u32::deserialize(&mut reader)?)
            } else {
                None
            };

            let length = description_length.or(default_length);

            let sample_group_description_entry =
                SampleGroupDescriptionEntry::deserialize_seed(&mut reader, (grouping_type, length))?;

            entries.push(sample_group_description_entry);
        }

        Ok(Self {
            full_header,
            grouping_type,
            default_length,
            default_group_description_index,
            entry_count,
            entries,
        })
    }
}

impl Serialize for SampleGroupDescriptionBox<'_> {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        self.grouping_type.serialize(&mut writer)?;
        if self.full_header.version >= 1 {
            self.default_length
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "default_length is required"))?
                .serialize(&mut writer)?;
        }
        if self.full_header.version >= 2 {
            self.default_group_description_index
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "default_group_description_index is required",
                ))?
                .serialize(&mut writer)?;
        }

        self.entry_count.serialize(&mut writer)?;
        for entry in &self.entries {
            if self.full_header.version >= 1 && self.default_length.is_some_and(|l| l == 0) {
                (entry.size() as u32).serialize(&mut writer)?;
            }
            entry.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for SampleGroupDescriptionBox<'_> {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        size += 4; // grouping_type
        if self.full_header.version >= 1 {
            size += 4; // default_length
        }
        if self.full_header.version >= 2 {
            size += 4; // default_group_description_index
        }
        size += 4; // entry_count
        size += self
            .entries
            .iter()
            .map(|entry| {
                let mut size = 0;
                if self.full_header.version >= 1 && self.default_length.is_some_and(|l| l == 0) {
                    size += 4; // description_length
                }
                size += entry.size();
                size
            })
            .sum::<usize>();

        Self::add_header_size(size)
    }
}

/// Compact sample to group box
///
/// ISO/IEC 14496-12 - 8.9.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"csgp", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct CompactSampleToGroupBox {
    // full header:
    /// The version of the box.
    pub version: u8,
    /// The flags of the box.
    pub flags: CompactSampleToGroupBoxFlags,
    // body:
    /// An integer that identifies the type (i.e. criterion used to form the sample groups) of the
    /// sample grouping and links it to its sample group description table with the same value for grouping
    /// type. At most one occurrence of either the ['csgp'](CompactSampleToGroupBox) or ['sbgp'](SampleToGroupBox) with
    /// the same value for `grouping_type` (and, if used, `grouping_type_parameter`) shall exist for a track.
    pub grouping_type: [u8; 4],
    /// An indication of the sub-type of the grouping.
    pub grouping_type_parameter: Option<u32>,
    /// Indicates the length of the associated pattern in the pattern array that follows it. The
    /// sum of the included sample_count values indicates the number of mapped samples.
    pub pattern_count: u32,
    /// `pattern_length` and `sample_count`
    pub patterns: Vec<CompactSampleToGroupBoxPattern>,
    /// `sample_group_description_index[j][k]`
    ///
    /// See [`CompactSampleToGroupBoxSampleGroupDescriptionIndex::value`] for details.
    pub sample_group_description_index: Vec<Vec<CompactSampleToGroupBoxSampleGroupDescriptionIndex>>,
}

/// Flags for [`CompactSampleToGroupBox`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CompactSampleToGroupBoxFlags {
    /// A flag that shall be zero when this box appears inside a [`TrackBox`](super::TrackBox)
    /// but may be 0 or 1 when this box appears inside a [`TrackFragmentBox`](super::TrackFragmentBox).
    /// When it is 1, it indicates that the most significant bit (MSB) of every `sample_group_description_index`
    /// does not form part of the index number but instead indicates which [`SampleGroupDescriptionBox`] the
    /// group description is to be found in: if the MSB is 0, the index identifies a group description from
    /// the [`TrackBox`](super::TrackBox)'s [`SampleGroupDescriptionBox`];
    /// if the MSB is 1, the index identifies a group description from the [`TrackFragmentBox`](super::TrackFragmentBox)'s
    /// [`SampleGroupDescriptionBox`].
    pub index_msb_indicates_fragment_local_description: bool,
    /// If set, [`CompactSampleToGroupBox::grouping_type_parameter`] is present.
    pub grouping_type_parameter_present: bool,
    /// Inidicates the size of [`CompactSampleToGroupBoxPattern::pattern_length`].
    ///
    /// - `0`: 4 bits
    /// - `1`: 8 bits
    /// - `2`: 16 bits
    /// - `3`: 32 bits
    pub pattern_size_code: u8,
    /// Inidicates the size of [`CompactSampleToGroupBoxPattern::sample_count`].
    ///
    /// - `0`: 4 bits
    /// - `1`: 8 bits
    /// - `2`: 16 bits
    /// - `3`: 32 bits
    pub count_size_code: u8,
    /// The size of [`CompactSampleToGroupBoxSampleGroupDescriptionIndex::value`].
    ///
    /// - `0`: 4 bits
    /// - `1`: 8 bits
    /// - `2`: 16 bits
    /// - `3`: 32 bits
    pub index_size_code: u8,
}

impl<'a> Deserialize<'a> for CompactSampleToGroupBoxFlags {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let value = U24Be::deserialize(reader)?.0;

        Ok(Self {
            index_msb_indicates_fragment_local_description: (value >> 7) & 0b1 != 0,
            grouping_type_parameter_present: (value >> 6) & 0b1 != 0,
            pattern_size_code: ((value >> 4) & 0b11) as u8,
            count_size_code: ((value >> 2) & 0b11) as u8,
            index_size_code: (value & 0b11) as u8,
        })
    }
}

impl Serialize for CompactSampleToGroupBoxFlags {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);
        bit_writer.write_bit(self.index_msb_indicates_fragment_local_description)?;
        bit_writer.write_bit(self.grouping_type_parameter_present)?;
        bit_writer.write_bits(self.pattern_size_code as u64, 2)?;
        bit_writer.write_bits(self.count_size_code as u64, 2)?;
        bit_writer.write_bits(self.index_size_code as u64, 2)?;
        Ok(())
    }
}

impl IsoSized for CompactSampleToGroupBoxFlags {
    fn size(&self) -> usize {
        3
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for CompactSampleToGroupBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let version = u8::deserialize(&mut reader)?;
        let flags = CompactSampleToGroupBoxFlags::deserialize(&mut reader)?;

        let grouping_type = <[u8; 4]>::deserialize(&mut reader)?;
        let grouping_type_parameter = if flags.grouping_type_parameter_present {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let pattern_count = u32::deserialize(&mut reader)?;

        let mut patterns = Vec::with_capacity(pattern_count as usize);
        for _ in 0..pattern_count {
            patterns.push(CompactSampleToGroupBoxPattern::deserialize_seed(&mut reader, flags)?);
        }

        let mut sample_group_description_index = Vec::with_capacity(pattern_count as usize);

        let mut next_value = None; // used as a buffer for 4 bit values
        for pattern in patterns.iter_mut() {
            let mut current = Vec::with_capacity(pattern.pattern_length as usize);

            for _ in 0..pattern.pattern_length {
                match flags.index_size_code {
                    0 => match next_value {
                        None => {
                            let byte = u8::deserialize(&mut reader)?;

                            let mut value = byte >> 4;
                            let fragment_local = flags.index_msb_indicates_fragment_local_description.then(|| {
                                // Most significant bit indicates the fragment local flag
                                let msb = (value >> 3) & 0b1 != 0;
                                value &= 0b0111;
                                msb
                            });

                            current.push(CompactSampleToGroupBoxSampleGroupDescriptionIndex {
                                value: value as u32,
                                fragment_local,
                            });
                            next_value = Some(byte & 0b0000_1111);
                        }
                        Some(mut value) => {
                            let fragment_local = flags.index_msb_indicates_fragment_local_description.then(|| {
                                // Most significant bit indicates the fragment local flag
                                let msb = (value >> 3) & 0b1 != 0;
                                value &= 0b0111;
                                msb
                            });

                            current.push(CompactSampleToGroupBoxSampleGroupDescriptionIndex {
                                value: value as u32,
                                fragment_local,
                            });
                            next_value = None;
                        }
                    },
                    1 => {
                        let mut value = u8::deserialize(&mut reader)?;
                        let fragment_local = flags.index_msb_indicates_fragment_local_description.then(|| {
                            // Most significant bit indicates the fragment local flag
                            let msb = (value >> 7) & 0b1 != 0;
                            value &= 0b0111_1111;
                            msb
                        });
                        current.push(CompactSampleToGroupBoxSampleGroupDescriptionIndex {
                            value: value as u32,
                            fragment_local,
                        });
                    }
                    2 => {
                        let mut value = u16::deserialize(&mut reader)?;
                        let fragment_local = flags.index_msb_indicates_fragment_local_description.then(|| {
                            // Most significant bit indicates the fragment local flag
                            let msb = (value >> 15) & 0b1 != 0;
                            value &= 0b0111_1111_1111_1111;
                            msb
                        });

                        current.push(CompactSampleToGroupBoxSampleGroupDescriptionIndex {
                            value: value as u32,
                            fragment_local,
                        });
                    }
                    3 => {
                        let mut value = u32::deserialize(&mut reader)?;
                        let fragment_local = flags.index_msb_indicates_fragment_local_description.then(|| {
                            // Most significant bit indicates the fragment local flag
                            let msb = (value >> 31) & 0b1 != 0;
                            value &= 0b0111_1111_1111_1111_1111_1111_1111_1111;
                            msb
                        });

                        current.push(CompactSampleToGroupBoxSampleGroupDescriptionIndex { value, fragment_local });
                    }
                    _ => unreachable!(),
                }
            }

            sample_group_description_index.push(current);
        }

        Ok(Self {
            version,
            flags,
            grouping_type,
            grouping_type_parameter,
            pattern_count,
            patterns,
            sample_group_description_index,
        })
    }
}

fn f(index: u8) -> u8 {
    match index {
        0 => 4,
        1 => 8,
        2 => 16,
        3 => 32,
        _ => unreachable!(),
    }
}

impl Serialize for CompactSampleToGroupBox {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        self.serialize_box_header(&mut bit_writer)?;
        self.version.serialize(&mut bit_writer)?;
        self.flags.serialize(&mut bit_writer)?;

        self.grouping_type.serialize(&mut bit_writer)?;

        if let Some(grouping_type_parameter) = self.grouping_type_parameter {
            grouping_type_parameter.serialize(&mut bit_writer)?;
        }

        self.pattern_count.serialize(&mut bit_writer)?;
        for pattern in &self.patterns {
            bit_writer.write_bits(pattern.pattern_length as u64, f(self.flags.pattern_size_code))?;
            bit_writer.write_bits(pattern.sample_count as u64, f(self.flags.count_size_code))?;
        }

        for j in &self.sample_group_description_index {
            for k in j {
                let bit_count = f(self.flags.index_size_code);
                bit_writer.write_bits(k.to_value(bit_count) as u64, bit_count)?;
            }
        }

        bit_writer.align()?;

        Ok(())
    }
}

impl IsoSized for CompactSampleToGroupBox {
    fn size(&self) -> usize {
        let mut size = 0;
        size += self.version.size();
        size += self.flags.size();
        size += 4; // grouping_type
        if self.grouping_type_parameter.is_some() {
            size += 4; // grouping_type_parameter
        }
        size += 4; // pattern_count

        let mut bits = 0;
        bits += (f(self.flags.pattern_size_code) + f(self.flags.count_size_code)) * self.patterns.len() as u8;
        for j in &self.sample_group_description_index {
            bits += f(self.flags.index_size_code) * j.len() as u8;
        }
        size += (bits as usize).div_ceil(8);

        Self::add_header_size(size)
    }
}

/// A pattern in [`CompactSampleToGroupBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct CompactSampleToGroupBoxPattern {
    /// Corresponds to a pattern within the second array of `sample_group_description_index[j]` values.
    /// Each instance of `pattern_length[i]` shall be greater than 0.
    pub pattern_length: u32,
    /// Specifies the number of samples that use the `i`-th pattern.
    /// `sample_count[i]` shall be greater than zero, and `sample_count[i]` shall be
    /// greater than or equal to [`pattern_length[i]`](Self::pattern_length).
    pub sample_count: u32,
}

impl<'a> DeserializeSeed<'a, CompactSampleToGroupBoxFlags> for CompactSampleToGroupBoxPattern {
    fn deserialize_seed<R>(mut reader: R, seed: CompactSampleToGroupBoxFlags) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        if (seed.pattern_size_code == 0) != (seed.count_size_code == 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "If one of pattern size code and count size is 0, the other must be 0 too".to_string(),
            ));
        }

        let mut pattern_length = match seed.pattern_size_code {
            1 => u8::deserialize(&mut reader)? as u32,  // 8 bits
            2 => u16::deserialize(&mut reader)? as u32, // 16 bits
            3 => u32::deserialize(&mut reader)?,        // 32 bits
            _ => 0,                                     // skip
        };

        let sample_count = match seed.count_size_code {
            0 => {
                // 4 bits
                let byte = u8::deserialize(&mut reader)?;
                pattern_length = (byte >> 4) as u32;
                (byte & 0b0000_1111) as u32
            }
            1 => u8::deserialize(&mut reader)? as u32,  // 8 bits
            2 => u16::deserialize(&mut reader)? as u32, // 16 bits
            3 => u32::deserialize(&mut reader)?,        // 32 bits
            _ => unreachable!(),
        };

        Ok(Self {
            pattern_length,
            sample_count,
        })
    }
}

/// The `sample_group_description_index[j][k]` in [`CompactSampleToGroupBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct CompactSampleToGroupBoxSampleGroupDescriptionIndex {
    /// An integer that gives the index of the sample group entry
    /// which describes the samples in this group. The index ranges from 1 to the number of sample group
    /// entries in the SampleGroupDescriptionBox, inclusive, or takes the value 0 to indicate that this
    /// sample is a member of no group of this type.
    pub value: u32,
    /// If present, indicates fragment_local or global.
    pub fragment_local: Option<bool>,
}

impl CompactSampleToGroupBoxSampleGroupDescriptionIndex {
    /// Converts this value to a number with given size in bits.
    pub fn to_value(&self, size: u8) -> u32 {
        if let Some(fl) = self.fragment_local {
            let mut value = (fl as u32) << (size - 1);
            value |= self.value & ((1 << (size - 1)) - 1);
            value
        } else {
            self.value & ((1 << size) - 1)
        }
    }
}
