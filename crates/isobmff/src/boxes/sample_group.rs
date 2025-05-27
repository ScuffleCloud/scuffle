use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, SerializeSeed, U24Be, ZeroCopyReader};
use scuffle_bytes_util::{BitWriter, BytesCow, IoResultExt};

use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Sample to group box
///
/// ISO/IEC 14496-12 - 8.9.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sbgp", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct SampleToGroupBox {
    pub full_header: FullBoxHeader,
    pub grouping_type: [u8; 4],
    pub grouping_type_parameter: Option<u32>,
    pub entry_count: u32,
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

#[derive(Debug, PartialEq, Eq)]
pub struct SampleToGroupBoxEntry {
    pub sample_count: u32,
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
    pub full_header: FullBoxHeader,
    pub grouping_type: [u8; 4],
    pub default_length: Option<u32>,
    pub default_group_description_index: Option<u32>,
    pub entry_count: u32,
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
            entries.push(SampleGroupDescriptionEntry::deserialize_seed(
                &mut reader,
                (grouping_type, default_length),
            )?);
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
            entry.serialize_seed(&mut writer, self)?;
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
        size += self.entries.iter().map(|entry| entry.size(self)).sum::<usize>();

        Self::add_header_size(size)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SampleGroupDescriptionEntry<'a> {
    pub description_length: Option<u32>,
    pub sample_group_description_entry: SampleGroupDescriptionEntryType<'a>,
}

impl<'a> DeserializeSeed<'a, ([u8; 4], Option<u32>)> for SampleGroupDescriptionEntry<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: ([u8; 4], Option<u32>)) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let (grouping_type, default_length) = seed;

        let description_length = if default_length.is_some_and(|l| l == 0) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let sample_group_description_entry =
            SampleGroupDescriptionEntryType::deserialize_seed(reader, (grouping_type, default_length))?;

        Ok(Self {
            description_length,
            sample_group_description_entry,
        })
    }
}

impl SerializeSeed<&SampleGroupDescriptionBox<'_>> for SampleGroupDescriptionEntry<'_> {
    fn serialize_seed<W>(&self, mut writer: W, seed: &SampleGroupDescriptionBox) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if seed.full_header.version >= 1 && seed.default_length.is_some_and(|l| l == 0) {
            self.description_length
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "description_length is required"))?
                .serialize(&mut writer)?;
        }
        self.sample_group_description_entry.serialize(&mut writer)?;

        Ok(())
    }
}

impl SampleGroupDescriptionEntry<'_> {
    pub fn size(&self, parent: &SampleGroupDescriptionBox) -> usize {
        let mut size = 0;
        if parent.full_header.version >= 1 && parent.default_length.is_some_and(|l| l == 0) {
            size += 4; // description_length
        }
        size += self.sample_group_description_entry.size();
        size
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SampleGroupDescriptionEntryType<'a> {
    // "roll"
    RollRecoveryEntry {
        roll_distance: i16,
    },
    // "alst"
    AlternativeStartupEntry {
        roll_count: u16,
        first_output_sample: u16,
        sample_offset: Vec<u32>,
        nums: Vec<AlternativeStartupEntryNums>,
    },
    // "rap "
    VisualRandomAccessEntry {
        num_leading_samples_known: bool,
        num_leading_samples: u8,
    },
    // "tele"
    TemporalLevelEntry {
        level_independently_decodable: bool,
    },
    // "drap"
    VisualDRAPEntry {
        drap_type: u8,
    },
    // "pasr"
    PixelAspectRatioEntry {
        h_spacing: u32,
        v_spacing: u32,
    },
    // "casg"
    CleanApertureEntry {
        clean_aperture_width_n: u32,
        clean_aperture_width_d: u32,
        clean_aperture_height_n: u32,
        clean_aperture_height_d: u32,
        horiz_off_n: u32,
        horiz_off_d: u32,
        vert_off_n: u32,
        vert_off_d: u32,
    },
    // "prol"
    AudioPreRollEntry {
        roll_distance: i16,
    },
    // "rash"
    RateShareEntry {
        operation_point_count: u16,
        operation_points: Vec<RateShareEntryOperationPoint>,
        maximum_bitrate: u32,
        minimum_bitrate: u32,
        discard_priority: u8,
    },
    // "sap "
    SAPEntry {
        dependent_flag: bool,
        sap_type: u8,
    },
    // "stmi"
    SampleToMetadataItemEntry {
        meta_box_handler_type: u32,
        num_items: u32,
        item_id: Vec<u32>,
    },
    Unknown {
        grouping_type: [u8; 4],
        data: BytesCow<'a>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct AlternativeStartupEntryNums {
    pub num_output_samples: u16,
    pub num_total_samples: u16,
}

impl<'a> Deserialize<'a> for AlternativeStartupEntryNums {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            num_output_samples: u16::deserialize(&mut reader)?,
            num_total_samples: u16::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for AlternativeStartupEntryNums {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.num_output_samples.serialize(&mut writer)?;
        self.num_total_samples.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for AlternativeStartupEntryNums {
    fn size(&self) -> usize {
        2 + 2 // num_output_samples + num_total_samples
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RateShareEntryOperationPoint {
    pub target_rate_share: u16,
    pub available_bitrate: Option<u32>,
}

impl<'a> Deserialize<'a> for RateShareEntryOperationPoint {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let available_bitrate = u32::deserialize(&mut reader)?;
        let target_rate_share = u16::deserialize(&mut reader)?;

        Ok(Self {
            available_bitrate: Some(available_bitrate),
            target_rate_share,
        })
    }
}

impl Serialize for RateShareEntryOperationPoint {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if let Some(available_bitrate) = &self.available_bitrate {
            available_bitrate.serialize(&mut writer)?;
        }
        self.target_rate_share.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for RateShareEntryOperationPoint {
    fn size(&self) -> usize {
        if self.available_bitrate.is_some() {
            4 + 2 // available_bitrate + target_rate_share
        } else {
            2 // target_rate_share
        }
    }
}

impl<'a> DeserializeSeed<'a, ([u8; 4], Option<u32>)> for SampleGroupDescriptionEntryType<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: ([u8; 4], Option<u32>)) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let (grouping_type, description_length) = seed;

        match &grouping_type {
            b"roll" => {
                let roll_distance = i16::deserialize(&mut reader)?;
                Ok(Self::RollRecoveryEntry { roll_distance })
            }
            b"alst" => {
                let roll_count = u16::deserialize(&mut reader)?;
                let first_output_sample = u16::deserialize(&mut reader)?;

                let mut sample_offset = Vec::with_capacity(roll_count as usize);
                for _ in 0..roll_count {
                    sample_offset.push(u32::deserialize(&mut reader)?);
                }

                let mut nums = Vec::new();
                loop {
                    let Some(num) = AlternativeStartupEntryNums::deserialize(&mut reader).eof_to_none()? else {
                        break;
                    };
                    nums.push(num);
                }

                Ok(Self::AlternativeStartupEntry {
                    roll_count,
                    first_output_sample,
                    sample_offset,
                    nums,
                })
            }
            b"rap " => {
                let byte = u8::deserialize(&mut reader)?;
                let num_leading_samples_known = (byte & 0b1000_0000) != 0;
                let num_leading_samples = byte & 0b0111_1111;

                Ok(Self::VisualRandomAccessEntry {
                    num_leading_samples_known,
                    num_leading_samples,
                })
            }
            b"tele" => {
                let level_independently_decodable = (u8::deserialize(&mut reader)? & 0b1000_0000) != 0;

                Ok(Self::TemporalLevelEntry {
                    level_independently_decodable,
                })
            }
            b"drap" => {
                let drap_type = ((u32::deserialize(&mut reader)? >> 29) & 0b111) as u8;
                Ok(Self::VisualDRAPEntry { drap_type })
            }
            b"pasr" => Ok(Self::PixelAspectRatioEntry {
                h_spacing: u32::deserialize(&mut reader)?,
                v_spacing: u32::deserialize(&mut reader)?,
            }),
            b"casg" => Ok(Self::CleanApertureEntry {
                clean_aperture_width_n: u32::deserialize(&mut reader)?,
                clean_aperture_width_d: u32::deserialize(&mut reader)?,
                clean_aperture_height_n: u32::deserialize(&mut reader)?,
                clean_aperture_height_d: u32::deserialize(&mut reader)?,
                horiz_off_n: u32::deserialize(&mut reader)?,
                horiz_off_d: u32::deserialize(&mut reader)?,
                vert_off_n: u32::deserialize(&mut reader)?,
                vert_off_d: u32::deserialize(&mut reader)?,
            }),
            b"prol" => Ok(Self::AudioPreRollEntry {
                roll_distance: i16::deserialize(&mut reader)?,
            }),
            b"rash" => {
                let operation_point_count = u16::deserialize(&mut reader)?;
                let mut operation_points = Vec::with_capacity(operation_point_count as usize);

                if operation_point_count == 1 {
                    let target_rate_share = u16::deserialize(&mut reader)?;
                    operation_points.push(RateShareEntryOperationPoint {
                        target_rate_share,
                        available_bitrate: None,
                    });
                } else {
                    for _ in 0..operation_point_count {
                        operation_points.push(RateShareEntryOperationPoint::deserialize(&mut reader)?);
                    }
                }

                let maximum_bitrate = u32::deserialize(&mut reader)?;
                let minimum_bitrate = u32::deserialize(&mut reader)?;
                let discard_priority = u8::deserialize(&mut reader)?;

                Ok(Self::RateShareEntry {
                    operation_point_count,
                    operation_points,
                    maximum_bitrate,
                    minimum_bitrate,
                    discard_priority,
                })
            }
            b"sap " => {
                // x000 xxxx
                let byte = u8::deserialize(&mut reader)?;
                let dependent_flag = (byte & 0b1000_0000) != 0;
                let sap_type = byte & 0b0000_1111;

                Ok(Self::SAPEntry {
                    dependent_flag,
                    sap_type,
                })
            }
            b"stmi" => {
                let meta_box_handler_type = u32::deserialize(&mut reader)?;
                let num_items = u32::deserialize(&mut reader)?;

                let mut item_id = Vec::with_capacity(num_items as usize);
                for _ in 0..num_items {
                    item_id.push(u32::deserialize(&mut reader)?);
                }

                Ok(Self::SampleToMetadataItemEntry {
                    meta_box_handler_type,
                    num_items,
                    item_id,
                })
            }
            _ => {
                let data = if let Some(description_length) = description_length {
                    reader.try_read(description_length as usize)?
                } else {
                    BytesCow::new()
                };

                Ok(Self::Unknown { grouping_type, data })
            }
        }
    }
}

impl Serialize for SampleGroupDescriptionEntryType<'_> {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Self::RollRecoveryEntry { roll_distance } => roll_distance.serialize(&mut writer)?,
            Self::AlternativeStartupEntry {
                roll_count,
                first_output_sample,
                sample_offset,
                nums,
            } => {
                roll_count.serialize(&mut writer)?;
                first_output_sample.serialize(&mut writer)?;
                for offset in sample_offset {
                    offset.serialize(&mut writer)?;
                }
                for num in nums {
                    num.serialize(&mut writer)?;
                }
            }
            Self::VisualRandomAccessEntry {
                num_leading_samples_known,
                num_leading_samples,
            } => {
                let mut byte = (*num_leading_samples_known as u8) << 7;
                byte |= *num_leading_samples;
                byte.serialize(&mut writer)?;
            }
            Self::TemporalLevelEntry {
                level_independently_decodable,
            } => {
                ((*level_independently_decodable as u8) << 7).serialize(&mut writer)?;
            }
            Self::VisualDRAPEntry { drap_type } => {
                let byte = ((*drap_type & 0b1111) as u32) << 29;
                byte.serialize(&mut writer)?;
            }
            Self::PixelAspectRatioEntry { h_spacing, v_spacing } => {
                h_spacing.serialize(&mut writer)?;
                v_spacing.serialize(&mut writer)?;
            }
            Self::CleanApertureEntry {
                clean_aperture_width_n,
                clean_aperture_width_d,
                clean_aperture_height_n,
                clean_aperture_height_d,
                horiz_off_n,
                horiz_off_d,
                vert_off_n,
                vert_off_d,
            } => {
                clean_aperture_width_n.serialize(&mut writer)?;
                clean_aperture_width_d.serialize(&mut writer)?;
                clean_aperture_height_n.serialize(&mut writer)?;
                clean_aperture_height_d.serialize(&mut writer)?;
                horiz_off_n.serialize(&mut writer)?;
                horiz_off_d.serialize(&mut writer)?;
                vert_off_n.serialize(&mut writer)?;
                vert_off_d.serialize(&mut writer)?;
            }
            Self::AudioPreRollEntry { roll_distance } => roll_distance.serialize(&mut writer)?,
            Self::RateShareEntry {
                operation_point_count,
                operation_points,
                maximum_bitrate,
                minimum_bitrate,
                discard_priority,
            } => {
                operation_point_count.serialize(&mut writer)?;
                for operation_point in operation_points {
                    if *operation_point_count > 1 && operation_point.available_bitrate.is_none() {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "available_bitrate is required"));
                    }
                    operation_point.serialize(&mut writer)?;
                }
                maximum_bitrate.serialize(&mut writer)?;
                minimum_bitrate.serialize(&mut writer)?;
                discard_priority.serialize(&mut writer)?;
            }
            Self::SAPEntry {
                dependent_flag,
                sap_type,
            } => {
                let mut byte = (*dependent_flag as u8) << 7;
                byte |= *sap_type & 0b0000_1111;
                byte.serialize(&mut writer)?;
            }
            Self::SampleToMetadataItemEntry {
                meta_box_handler_type,
                num_items,
                item_id,
            } => {
                meta_box_handler_type.serialize(&mut writer)?;
                num_items.serialize(&mut writer)?;
                for id in item_id {
                    id.serialize(&mut writer)?;
                }
            }
            Self::Unknown { data, .. } => {
                data.serialize(&mut writer)?;
            }
        }

        Ok(())
    }
}

impl IsoSized for SampleGroupDescriptionEntryType<'_> {
    fn size(&self) -> usize {
        match self {
            Self::RollRecoveryEntry { .. } => 2, // roll_distance
            Self::AlternativeStartupEntry { sample_offset, nums, .. } => 2 + 2 + sample_offset.size() + nums.size(),
            Self::VisualRandomAccessEntry { .. } => 1, // num_leading_samples_known + num_leading_samples
            Self::TemporalLevelEntry { .. } => 1,      // level_independently_decodable
            Self::VisualDRAPEntry { .. } => 4,         // drap_type
            Self::PixelAspectRatioEntry { .. } => 4 + 4, // h_spacing + v_spacing
            Self::CleanApertureEntry { .. } => 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4, /* clean_aperture_width_n + clean_aperture_width_d + clean_aperture_height_n + clean_aperture_height_d + horiz_off_n + horiz_off_d + vert_off_n + vert_off_d */
            Self::AudioPreRollEntry { .. } => 2,                              // roll_distance
            Self::RateShareEntry { operation_points, .. } => 2 + operation_points.size() + 4 + 4 + 1,
            Self::SAPEntry { .. } => 1, // dependent_flag + sap_type
            Self::SampleToMetadataItemEntry { item_id, .. } => {
                4 + 4 + item_id.size() // meta_box_handler_type + num_items + item_id
            }
            Self::Unknown { data, .. } => data.size(),
        }
    }
}

/// Compact sample to group box
///
/// ISO/IEC 14496-12 - 8.9.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"csgp", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct CompactSampleToGroupBox {
    // full header:
    pub version: u8,
    pub flags: CompactSampleToGroupBoxFlags,
    // body:
    pub grouping_type: [u8; 4],
    pub grouping_type_parameter: Option<u32>,
    pub pattern_count: u32,
    pub patterns: Vec<CompactSampleToGroupBoxPattern>,
    pub sample_group_description_index: Vec<Vec<CompactSampleToGroupBoxSampleGroupDescriptionIndex>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CompactSampleToGroupBoxFlags {
    pub index_msb_indicates_fragment_local_description: bool,
    pub grouping_type_parameter_present: bool,
    pub pattern_size_code: u8,
    pub count_size_code: u8,
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

#[derive(Debug, PartialEq, Eq)]
pub struct CompactSampleToGroupBoxPattern {
    pub pattern_length: u32,
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

#[derive(Debug, PartialEq, Eq)]
pub struct CompactSampleToGroupBoxSampleGroupDescriptionIndex {
    pub value: u32,
    pub fragment_local: Option<bool>,
}

impl CompactSampleToGroupBoxSampleGroupDescriptionIndex {
    pub fn to_value(&self, size: u8) -> u32 {
        if let Some(fl) = self.fragment_local {
            let mut value = (fl as u32) << (size - 1);
            value |= self.value & ((1 << (size - 1)) - 1);
            value
        } else {
            self.value
        }
    }
}
