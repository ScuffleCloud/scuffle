use std::fmt::Debug;
use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, SerializeSeed, ZeroCopyReader};

use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

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
        Ok(Self {
            sample_count: u32::deserialize(&mut reader)?,
            sample_delta: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for TimeToSampleBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_count.serialize(&mut writer)?;
        self.sample_delta.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for TimeToSampleBoxEntry {
    fn size(&self) -> usize {
        self.sample_count.size() + self.sample_delta.size()
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
        Ok(Self {
            sample_count: u32::deserialize(&mut reader)?,
            sample_offset: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for CompositionOffsetBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_count.serialize(&mut writer)?;
        self.sample_offset.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for CompositionOffsetBoxEntry {
    fn size(&self) -> usize {
        self.sample_count.size() + self.sample_offset.size()
    }
}

/// Composition to decode box
///
/// ISO/IEC 14496-12 - 8.6.1.4
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"cslg", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct CompositionToDecodeBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub composition_to_dt_shift: i64,
    pub least_decode_to_display_delta: i64,
    pub greatest_decode_to_display_delta: i64,
    pub composition_start_time: i64,
    pub composition_end_time: i64,
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

impl Serialize for CompositionToDecodeBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.header.serialize(&mut writer)?;
        if self.header.version == 0 {
            (self.composition_to_dt_shift as i32).serialize(&mut writer)?;
            (self.least_decode_to_display_delta as i32).serialize(&mut writer)?;
            (self.greatest_decode_to_display_delta as i32).serialize(&mut writer)?;
            (self.composition_start_time as i32).serialize(&mut writer)?;
            (self.composition_end_time as i32).serialize(&mut writer)?;
        } else {
            self.composition_to_dt_shift.serialize(&mut writer)?;
            self.least_decode_to_display_delta.serialize(&mut writer)?;
            self.greatest_decode_to_display_delta.serialize(&mut writer)?;
            self.composition_start_time.serialize(&mut writer)?;
            self.composition_end_time.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl IsoSized for CompositionToDecodeBox {
    fn size(&self) -> usize {
        let mut size = self.header.size();
        if self.header.version == 0 {
            size += 4 + 4 + 4 + 4 + 4;
        } else {
            size += 8 + 8 + 8 + 8 + 8;
        }
        size
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
        Ok(Self {
            shadowed_sample_number: u32::deserialize(&mut reader)?,
            sync_sample_number: u32::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for ShadowSyncSampleBoxEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.shadowed_sample_number.serialize(&mut writer)?;
        self.sync_sample_number.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for ShadowSyncSampleBoxEntry {
    fn size(&self) -> usize {
        self.shadowed_sample_number.size() + self.sync_sample_number.size()
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

#[derive(Debug, Clone, Copy)]
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

impl From<SampleDependencyTypeBoxEntry> for u8 {
    fn from(val: SampleDependencyTypeBoxEntry) -> Self {
        ((val.is_leading & 0b11) << 6)
            | ((val.sample_depends_on & 0b11) << 4)
            | ((val.sample_is_depended_on & 0b11) << 2)
            | (val.sample_has_redundancy & 0b11)
    }
}

impl IsoSized for SampleDependencyTypeBoxEntry {
    fn size(&self) -> usize {
        1
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
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"elst", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct EditListBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub entries: Vec<EditListBoxEntry>,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for EditListBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let entry_count = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(EditListBoxEntry::deserialize_seed(&mut reader, seed.version)?);
        }

        Ok(Self {
            header: seed,
            entry_count,
            entries,
        })
    }
}

impl Serialize for EditListBox {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize_seed(&mut writer, self.header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for EditListBox {
    fn size(&self) -> usize {
        self.header.size()
            + self.entry_count.size()
            + self
                .entries
                .iter()
                .map(|entry| entry.size(self.header.version))
                .sum::<usize>()
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

impl SerializeSeed<u8> for EditListBoxEntry {
    fn serialize_seed<W>(&self, mut writer: W, seed: u8) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if seed == 1 {
            self.edit_duration.serialize(&mut writer)?;
            self.media_time.serialize(&mut writer)?;
        } else {
            (self.edit_duration as u32).serialize(&mut writer)?;
            (self.media_time as i32).serialize(&mut writer)?;
        }
        self.media_rate_integer.serialize(&mut writer)?;
        self.media_rate_fraction.serialize(&mut writer)?;

        Ok(())
    }
}

impl EditListBoxEntry {
    pub fn size(&self, version: u8) -> usize {
        let mut size = 0;
        if version == 1 {
            size += 8 + 8;
        } else {
            size += 4 + 4;
        }
        size + 2 + 2
    }
}
