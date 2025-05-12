use std::io;

use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, ZeroCopyReader};

use crate::{BoxHeader, BoxType, FullBoxHeader, IsoBox};

/// Sample to group box
///
/// ISO/IEC 14496-12 - 8.9.2
#[derive(Debug)]
pub struct SampleToGroupBox {
    pub header: FullBoxHeader,
    pub grouping_type: [u8; 4],
    pub grouping_type_parameter: Option<u32>,
    pub entry_count: u32,
    pub entries: Vec<SampleToGroupBoxEntry>,
}

impl IsoBox for SampleToGroupBox {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"sbgp");
}

impl<'a> Deserialize<'a> for SampleToGroupBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SampleToGroupBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let grouping_type = <[u8; 4]>::deserialize(&mut reader)?;
        let grouping_type_parameter = if seed.version == 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let entry_count = u32::deserialize(&mut reader)?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let entry = SampleToGroupBoxEntry::deserialize(&mut reader)?;
            entries.push(entry);
        }

        Ok(Self {
            header: seed,
            grouping_type,
            grouping_type_parameter,
            entry_count,
            entries,
        })
    }
}

#[derive(Debug)]
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

/// Sample group description box
///
/// ISO/IEC 14496-12 - 8.9.3
#[derive(Debug)]
pub struct SampleGroupDescriptionBox {
    pub header: FullBoxHeader,
    pub grouping_type: [u8; 4],
    pub default_length: Option<u32>,
    pub default_group_description_index: Option<u32>,
    pub entry_count: u32,
    pub entries: Vec<SampleGroupDescriptionEntry>,
}

impl IsoBox for SampleGroupDescriptionBox {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"sgpd");
}

impl<'a> Deserialize<'a> for SampleGroupDescriptionBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for SampleGroupDescriptionBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let grouping_type = <[u8; 4]>::deserialize(&mut reader)?;
        let default_length = if seed.version >= 1 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_group_description_index = if seed.version >= 2 {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        let entry_count = u32::deserialize(&mut reader)?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let entry = SampleGroupDescriptionEntry::deserialize_seed(&mut reader, (grouping_type, default_length))?;
            entries.push(entry);
        }

        Ok(Self {
            header: seed,
            grouping_type,
            default_length,
            default_group_description_index,
            entry_count,
            entries,
        })
    }
}

#[derive(Debug)]
pub struct SampleGroupDescriptionEntry {
    pub description_length: Option<u32>,
    pub sample_group_description_entry: SampleGroupDescriptionEntryType,
}

impl<'a> DeserializeSeed<'a, ([u8; 4], Option<u32>)> for SampleGroupDescriptionEntry {
    fn deserialize_seed<R>(mut reader: R, seed: ([u8; 4], Option<u32>)) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let (grouping_type, default_length) = seed;

        let description_length = default_length
            .and_then(|default_length| (default_length == 0).then_some(u32::deserialize(&mut reader)))
            .transpose()?;

        let sample_group_description_entry = SampleGroupDescriptionEntryType::deserialize_seed(&mut reader, grouping_type)?;

        Ok(Self {
            description_length,
            sample_group_description_entry,
        })
    }
}

#[derive(Debug)]
pub enum SampleGroupDescriptionEntryType {
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
}

#[derive(Debug)]
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

#[derive(Debug)]
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

impl<'a> DeserializeSeed<'a, [u8; 4]> for SampleGroupDescriptionEntryType {
    fn deserialize_seed<R>(mut reader: R, seed: [u8; 4]) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        match &seed {
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
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown sample group description entry grouping type: {seed:?}"),
            )),
        }
    }
}

/// Compact sample to group box
///
/// ISO/IEC 14496-12 - 8.9.5
#[derive(Debug)]
pub struct CompactSampleToGroupBox {
    pub header: FullBoxHeader,
    pub grouping_type: [u8; 4],
    pub grouping_type_parameter: Option<u32>,
    pub pattern_count: u32,
    pub patterns: Vec<CompactSampleToGroupBoxPattern>,
    pub sample_group_description_index: Vec<Vec<CompactSampleToGroupBoxSampleGroupDescriptionIndex>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompactSampleToGroupBoxFlags {
    pub index_msb_indicates_fragment_local_description: bool,
    pub grouping_type_parameter_present: bool,
    pub pattern_size_code: u8,
    pub count_size_code: u8,
    pub index_size_code: u8,
}

impl From<u32> for CompactSampleToGroupBoxFlags {
    fn from(value: u32) -> Self {
        Self {
            index_msb_indicates_fragment_local_description: (value >> 7) & 0b1 != 0,
            grouping_type_parameter_present: (value >> 6) & 0b1 != 0,
            pattern_size_code: ((value >> 4) & 0b11) as u8,
            count_size_code: ((value >> 2) & 0b11) as u8,
            index_size_code: (value & 0b11) as u8,
        }
    }
}

impl IsoBox for CompactSampleToGroupBox {
    type Header = FullBoxHeader;

    const TYPE: BoxType = BoxType::FourCc(*b"csgp");
}

impl<'a> Deserialize<'a> for CompactSampleToGroupBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for CompactSampleToGroupBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let flags = CompactSampleToGroupBoxFlags::from(seed.flags);

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
            header: seed,
            grouping_type,
            grouping_type_parameter,
            pattern_count,
            patterns,
            sample_group_description_index,
        })
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CompactSampleToGroupBoxSampleGroupDescriptionIndex {
    pub value: u32,
    pub fragment_local: Option<bool>,
}
