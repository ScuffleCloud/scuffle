use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, ZeroCopyReader};

use crate::{FullBoxHeader, IsoBox};

/// Time to sample box
///
/// ISO/IEC 14496-12 - 8.6.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stts", crate_path = "crate")]
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

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ctts", crate_path = "crate")]
pub struct CompositionOffsetBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(repeated)]
    pub entries: Vec<CompositionOffsetBoxEntry>,
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
