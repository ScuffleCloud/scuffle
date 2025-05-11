use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, ZeroCopyReader};

use super::SampleEntry;

/// Sample entry
///
/// ISO/IEC 14496-12 - 12.4.4
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
