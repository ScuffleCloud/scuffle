use scuffle_bytes_util::zero_copy::Deserialize;

use super::SampleEntry;

/// Font sample entry
///
/// ISO/IEC 14496-12 - 12.7.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug)]
pub struct FontSampleEntry {
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for FontSampleEntry {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            sample_entry: SampleEntry::deserialize(reader)?,
        })
    }
}
