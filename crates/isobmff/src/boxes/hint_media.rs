use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, ZeroCopyReader};

use super::SampleEntry;
use crate::{FullBoxHeader, IsoBox, IsoSized};

/// Hint media header box
///
/// ISO/IEC 14496-12 - 12.4.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"hmhd", crate_path = crate)]
pub struct HintMediaHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The size in bytes of the largest PDU in this (hint) stream.
    pub max_pdu_size: u16,
    /// The average size of a PDU over the entire presentation.
    pub avg_pdu_size: u16,
    /// The maximum rate in bits/second over any window of one second.
    pub maxbitrate: u32,
    /// The average rate in bits/second over the entire presentation.
    pub avgbitrate: u32,
    /// Reserved 32 bits, must be set to zero.
    pub reserved: u32,
}

/// Hint sample entry
///
/// ISO/IEC 14496-12 - 12.4.4
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct HintSampleEntry {
    /// The sample entry that this box inherits from.
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for HintSampleEntry {
    fn deserialize<R: ZeroCopyReader<'a>>(reader: R) -> io::Result<Self> {
        let sample_entry = SampleEntry::deserialize(reader)?;
        Ok(HintSampleEntry { sample_entry })
    }
}

impl Serialize for HintSampleEntry {
    fn serialize<W: io::Write>(&self, writer: W) -> io::Result<()> {
        self.sample_entry.serialize(writer)
    }
}

impl IsoSized for HintSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
    }
}
