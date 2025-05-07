use std::fmt::Debug;
use std::{io, iter};

use scuffle_bytes_util::IoResultExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, ZeroCopyReader};

use crate::{BoxHeader, FullBoxHeader, IsoBox};

/// Time to sample box
///
/// ISO/IEC 14496-12 - 8.6.1.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stts", crate_path = crate)]
pub struct TimeToSampleBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub entries: Vec<TimeToSampleBoxEntry>,
}

#[derive(Debug)]
pub struct TimeToSampleBoxEntry {
    pub sample_count: u32,
    pub sample_delta: u32,
}

impl<'a> Deserialize<'a> for TimeToSampleBoxEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let sample_count = u32::deserialize(&mut reader)?;
        let sample_delta = u32::deserialize(&mut reader)?;

        Ok(Self {
            sample_count,
            sample_delta,
        })
    }
}

/// Composition time to sample box
///
/// ISO/IEC 14496-12 - 8.6.1.3
#[derive(IsoBox)]
#[iso_box(box_type = b"ctts", crate_path = crate)]
pub struct CompositionOffsetBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub entries: Vec<CompositionOffsetBoxEntry>,
}

impl Debug for CompositionOffsetBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositionOffsetBox")
            .field("header", &self.header)
            .field("entry_count", &self.entry_count)
            .field("entries.len", &self.entries.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct CompositionOffsetBoxEntry {
    pub sample_count: u32,
    /// This should be interpreted as signed when the version is 1
    pub sample_offset: u32,
}

impl<'a> Deserialize<'a> for CompositionOffsetBoxEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let sample_count = u32::deserialize(&mut reader)?;
        let sample_offset = u32::deserialize(&mut reader)?;
        Ok(Self {
            sample_count,
            sample_offset,
        })
    }
}

/// Composition to decode box
///
/// ISO/IEC 14496-12 - 8.6.1.4
#[derive(Debug)]
pub struct CompositionToDecodeBox {
    pub header: FullBoxHeader,
    pub composition_to_dt_shift: i64,
    pub least_decode_to_display_delta: i64,
    pub greatest_decode_to_display_delta: i64,
    pub composition_start_time: i64,
    pub composition_end_time: i64,
}

// Manual implementation because conditional fields are not supported in the macro

impl IsoBox for CompositionToDecodeBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"cslg";
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for CompositionToDecodeBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let composition_to_dt_shift = if seed.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let least_decode_to_display_delta = if seed.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let greatest_decode_to_display_delta = if seed.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let composition_start_time = if seed.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let composition_end_time = if seed.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };

        Ok(Self {
            header: seed,
            composition_to_dt_shift,
            least_decode_to_display_delta,
            greatest_decode_to_display_delta,
            composition_start_time,
            composition_end_time,
        })
    }
}

impl<'a> Deserialize<'a> for CompositionToDecodeBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

/// Sync sample box
///
/// ISO/IEC 14496-12 - 8.6.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stss", crate_path = crate)]
pub struct SyncSampleBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub sample_number: Vec<u32>,
}

/// Shadow sync sample box
///
/// ISO/IEC 14496-12 - 8.6.3.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stsh", crate_path = crate)]
pub struct ShadowSyncSampleBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub entries: Vec<ShadowSyncSampleBoxEntry>,
}

#[derive(Debug)]
pub struct ShadowSyncSampleBoxEntry {
    pub shadowed_sample_number: u32,
    pub sync_sample_number: u32,
}

impl<'a> Deserialize<'a> for ShadowSyncSampleBoxEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let shadowed_sample_number = u32::deserialize(&mut reader)?;
        let sync_sample_number = u32::deserialize(&mut reader)?;

        Ok(Self {
            shadowed_sample_number,
            sync_sample_number,
        })
    }
}

/// Independent and disposable samples box
///
/// ISO/IEC 14496-12 - 8.6.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"sdtp", crate_path = crate)]
pub struct SampleDependencyTypeBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(from = "u8", repeated)]
    pub entries: Vec<SampleDependencyTypeBoxEntry>,
}

#[derive(Debug)]
pub struct SampleDependencyTypeBoxEntry {
    pub is_leading: u8,
    pub sample_depends_on: u8,
    pub sample_is_depended_on: u8,
    pub sample_has_redundancy: u8,
}

impl From<u8> for SampleDependencyTypeBoxEntry {
    fn from(value: u8) -> Self {
        Self {
            is_leading: (value >> 6) & 0b11,
            sample_depends_on: (value >> 4) & 0b11,
            sample_is_depended_on: (value >> 2) & 0b11,
            sample_has_redundancy: value & 0b11,
        }
    }
}

/// Edit box
///
/// ISO/IEC 14496-12 - 8.6.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"edts", crate_path = crate)]
pub struct EditBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub elst: Option<EditListBox>,
}

/// Edit list box
///
/// ISO/IEC 14496-12 - 8.6.6
#[derive(Debug)]
pub struct EditListBox {
    pub header: FullBoxHeader,
    pub entry_count: u32,
    pub entries: Vec<EditListBoxEntry>,
}

impl IsoBox for EditListBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"elst";
}

impl<'a> Deserialize<'a> for EditListBox {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for EditListBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let entry_count = u32::deserialize(&mut reader)?;

        let entries = {
            if let Some(payload_size) = crate::BoxHeaderProperties::payload_size(&seed) {
                let mut payload_reader = ZeroCopyReader::take(&mut reader, payload_size);
                iter::from_fn(|| {
                    EditListBoxEntry::deserialize_seed(&mut payload_reader, seed.version)
                        .eof_to_none()
                        .transpose()
                })
                .collect::<Result<Vec<EditListBoxEntry>, io::Error>>()?
            } else {
                iter::from_fn(|| {
                    EditListBoxEntry::deserialize_seed(&mut reader, seed.version)
                        .eof_to_none()
                        .transpose()
                })
                .collect::<Result<Vec<EditListBoxEntry>, io::Error>>()?
            }
        };

        Ok(Self {
            header: seed,
            entry_count,
            entries,
        })
    }
}

#[derive(Debug)]
pub struct EditListBoxEntry {
    pub edit_duration: u64,
    pub media_time: i64,
    pub media_rate_integer: i16,
    pub media_rate_fraction: i16,
}

impl<'a> DeserializeSeed<'a, u8> for EditListBoxEntry {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let edit_duration = if seed == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let media_time = if seed == 1 {
            i64::deserialize(&mut reader)?
        } else {
            i32::deserialize(&mut reader)? as i64
        };
        let media_rate_integer = i16::deserialize(&mut reader)?;
        let media_rate_fraction = i16::deserialize(&mut reader)?;

        Ok(Self {
            edit_duration,
            media_time,
            media_rate_integer,
            media_rate_fraction,
        })
    }
}
