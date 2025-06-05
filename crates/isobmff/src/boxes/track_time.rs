use std::fmt::Debug;
use std::io;

use fixed::FixedI32;
use fixed::types::extra::U16;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, ZeroCopyReader};

use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Time to sample box
///
/// ISO/IEC 14496-12 - 8.6.1.2
#[derive(IsoBox, Debug, PartialEq, Eq, Default)]
#[iso_box(box_type = b"stts", crate_path = crate)]
pub struct TimeToSampleBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// `sample_count` and `sample_delta`.
    #[iso_box(repeated)]
    pub entries: Vec<TimeToSampleBoxEntry>,
}

/// Entry in the [`TimeToSampleBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct TimeToSampleBoxEntry {
    /// An integer that counts the number of consecutive samples that have the given duration.
    pub sample_count: u32,
    /// An integer that gives the difference between the decoding timestamp of the next
    /// sample and this one, in the time-scale of the media.
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
#[derive(IsoBox, PartialEq, Eq)]
#[iso_box(box_type = b"ctts", skip_impl(deserialize_seed), crate_path = crate)]
pub struct CompositionOffsetBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// `sample_count` and `sample_offset`.
    #[iso_box(repeated)]
    pub entries: Vec<CompositionOffsetBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for CompositionOffsetBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;
        let entry_count = u32::deserialize(&mut reader)?;

        let mut entries = Vec::new();
        if full_header.version == 0 || full_header.version == 1 {
            for _ in 0..entry_count {
                entries.push(CompositionOffsetBoxEntry::deserialize_seed(&mut reader, full_header.version)?);
            }
        }

        Ok(Self {
            full_header,
            entry_count,
            entries,
        })
    }
}

impl Debug for CompositionOffsetBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositionOffsetBox")
            .field("full_header", &self.full_header)
            .field("entry_count", &self.entry_count)
            .field("entries.len", &self.entries.len())
            .finish()
    }
}

/// Entry in the [`CompositionOffsetBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct CompositionOffsetBoxEntry {
    /// An integer that counts the number of consecutive samples that have the given offset.
    pub sample_count: u32,
    /// An integer that gives the offset between CT and DT, such that `CT[n] = DT[n] + sample_offset[n]`.
    pub sample_offset: i64,
}

impl<'a> DeserializeSeed<'a, u8> for CompositionOffsetBoxEntry {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(Self {
            sample_count: u32::deserialize(&mut reader)?,
            sample_offset: if seed == 0 {
                u32::deserialize(&mut reader)? as i64
            } else if seed == 1 {
                i32::deserialize(&mut reader)? as i64
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "cannot be called with version > 1",
                ));
            },
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"cslg", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct CompositionToDecodeBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// If this value is added to the composition timestamps (as calculated by the CTS
    /// offsets from the DTS), then for all samples, their CTS is guaranteed to be greater than or equal
    /// to their DTS, and the buffer model implied by the indicated profile/level will be honoured; if
    /// `leastDecodeToDisplayDelta` is positive or zero, this field can be 0; otherwise it should be at least
    /// (`-leastDecodeToDisplayDelta`)
    pub composition_to_dt_shift: i64,
    /// The smallest composition offset in the CompositionOffsetBox in this track.
    pub least_decode_to_display_delta: i64,
    /// The largest composition offset in the CompositionOffsetBox in this track.
    pub greatest_decode_to_display_delta: i64,
    /// The smallest computed composition timestamp (CTS) for any sample in the media of this track.
    pub composition_start_time: i64,
    /// The composition timestamp plus the composition duration, of the sample with the
    /// largest computed composition timestamp (CTS) in the media of this track; if this field takes the
    /// value 0, the composition end time is unknown.
    pub composition_end_time: i64,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for CompositionToDecodeBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let composition_to_dt_shift = if full_header.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let least_decode_to_display_delta = if full_header.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let greatest_decode_to_display_delta = if full_header.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let composition_start_time = if full_header.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };
        let composition_end_time = if full_header.version == 0 {
            i32::deserialize(&mut reader)? as i64
        } else {
            i64::deserialize(&mut reader)?
        };

        Ok(Self {
            full_header,
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
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        if self.full_header.version == 0 {
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
        let mut size = self.full_header.size();
        if self.full_header.version == 0 {
            size += 4 + 4 + 4 + 4 + 4;
        } else {
            size += 8 + 8 + 8 + 8 + 8;
        }

        Self::add_header_size(size)
    }
}

/// Sync sample box
///
/// ISO/IEC 14496-12 - 8.6.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stss", crate_path = crate)]
pub struct SyncSampleBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`sample_number`](Self::sample_number) vec.
    /// If `entry_count` is zero, there are no sync samples within the stream and the
    /// [`sample_number`](Self::sample_number) vec is empty.
    pub entry_count: u32,
    /// Gives, for each sync sample in the stream, its sample number.
    #[iso_box(repeated)]
    pub sample_number: Vec<u32>,
}

/// Shadow sync sample box
///
/// ISO/IEC 14496-12 - 8.6.3.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stsh", crate_path = crate)]
pub struct ShadowSyncSampleBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// `shadowed_sample_number` and `sync_sample_number`.
    #[iso_box(repeated)]
    pub entries: Vec<ShadowSyncSampleBoxEntry>,
}

/// Entry in the [`ShadowSyncSampleBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct ShadowSyncSampleBoxEntry {
    /// Gives the number of a sample for which there is an alternative sync sample.
    pub shadowed_sample_number: u32,
    /// Gives the number of the alternative sync sample.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sdtp", crate_path = crate)]
pub struct SampleDependencyTypeBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// `is_leading`, `sample_depends_on`, `sample_is_depended_on`, and `sample_has_redundancy`.
    #[iso_box(from = "u8", repeated)]
    pub entries: Vec<SampleDependencyTypeBoxEntry>,
}

/// Entry in the [`SampleDependencyTypeBox`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SampleDependencyTypeBoxEntry {
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"edts", crate_path = crate)]
pub struct EditBox {
    /// The contained [`EditListBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub elst: Option<EditListBox>,
}

/// Edit list box
///
/// ISO/IEC 14496-12 - 8.6.6
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"elst", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct EditListBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`entries`](Self::entries) vec.
    pub entry_count: u32,
    /// `edit_duration`, `media_time`, and `media_rate`.
    #[iso_box(repeated)]
    pub entries: Vec<EditListBoxEntry>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for EditListBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let entry_count = u32::deserialize(&mut reader)?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            entries.push(EditListBoxEntry::deserialize_seed(&mut reader, full_header.version)?);
        }

        Ok(Self {
            full_header,
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
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;
        self.entry_count.serialize(&mut writer)?;

        for entry in &self.entries {
            entry.serialize(&mut writer, self.full_header.version)?;
        }

        Ok(())
    }
}

impl IsoSized for EditListBox {
    fn size(&self) -> usize {
        let mut size = 0;
        size += self.full_header.size();
        size += self.entry_count.size();
        size += self
            .entries
            .iter()
            .map(|entry| entry.size(self.full_header.version))
            .sum::<usize>();

        Self::add_header_size(size)
    }
}

/// Entry in the [`EditListBox`].
#[derive(Debug, PartialEq, Eq)]
pub struct EditListBoxEntry {
    /// An integer that specifies the duration of this edit in units of the timescale in the
    /// [`MovieHeaderBox`](super::MovieHeaderBox).
    pub edit_duration: u64,
    /// An integer containing the starting time within the media of this edit entry (in media time
    /// scale units, in composition time). If this field is set to –1, it is an empty edit. The last edit in a track
    /// shall never be an empty edit. Any difference between the duration in the [`MovieHeaderBox`](super::MovieHeaderBox),
    /// and the track’s duration is expressed as an implicit empty edit at the end.
    pub media_time: i64,
    /// Specifies the relative rate at which to play the media corresponding to this edit entry. If
    /// this value is 0, then the edit is specifying a 'dwell': the media at media-time is presented for the
    /// edit_duration. The normal value, indicating normal-speed forward play, is 1.0.
    pub media_rate: FixedI32<U16>,
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
        let media_rate = FixedI32::from_bits(i32::deserialize(&mut reader)?);

        Ok(Self {
            edit_duration,
            media_time,
            media_rate,
        })
    }
}

impl EditListBoxEntry {
    fn serialize<W>(&self, mut writer: W, version: u8) -> io::Result<()>
    where
        W: std::io::Write,
    {
        if version == 1 {
            self.edit_duration.serialize(&mut writer)?;
            self.media_time.serialize(&mut writer)?;
        } else {
            (self.edit_duration as u32).serialize(&mut writer)?;
            (self.media_time as i32).serialize(&mut writer)?;
        }
        self.media_rate.to_bits().serialize(&mut writer)?;

        Ok(())
    }
}

impl EditListBoxEntry {
    /// Returns the size of this entry in bytes, depending on the version.
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
