use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use super::SampleEntry;
use crate::IsoSized;

/// Haptic sample entry
///
/// ISO/IEC 14496-12 - 12.11.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug)]
pub struct HapticSampleEntry {
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for HapticSampleEntry {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            sample_entry: SampleEntry::deserialize(reader)?,
        })
    }
}

impl Serialize for HapticSampleEntry {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(writer)
    }
}

impl IsoSized for HapticSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
    }
}
