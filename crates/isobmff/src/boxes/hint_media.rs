use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, ZeroCopyReader};

use super::SampleEntry;
use crate::{FullBoxHeader, IsoBox};

/// Hint media header box
///
/// ISO/IEC 14496-12 - 12.4.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"hmhd", crate_path = crate)]
pub struct HintMediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub max_pdu_size: u16,
    pub avg_pdu_size: u16,
    pub maxbitrate: u32,
    pub avgbitrate: u32,
    pub reserved: u32,
}

/// Hint sample entry
///
/// ISO/IEC 14496-12 - 12.4.4
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug)]
pub struct HintSampleEntry {
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for HintSampleEntry {
    fn deserialize<R: ZeroCopyReader<'a>>(reader: R) -> io::Result<Self> {
        let sample_entry = SampleEntry::deserialize(reader)?;
        Ok(HintSampleEntry { sample_entry })
    }
}
