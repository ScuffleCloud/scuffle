use scuffle_bytes_util::zero_copy::Deserialize;

use super::SampleEntry;
use crate::{FullBoxHeader, IsoBox};

/// Volumetric visual media header box
///
/// ISO/IEC 14496-12 - 12.10.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"vvhd", crate_path = crate)]
pub struct VolumetricVisualMediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    // empty
}

/// Volumetric visual sample entry
///
/// ISO/IEC 14496-12 - 12.10.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug)]
pub struct VolumetricVisualSampleEntry {
    pub sample_entry: SampleEntry,
    pub compressorname: [char; 32],
}

impl<'a> Deserialize<'a> for VolumetricVisualSampleEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let sample_entry = SampleEntry::deserialize(&mut reader)?;
        let compressorname = <[char; 32]>::deserialize(&mut reader)?;

        Ok(Self {
            sample_entry,
            compressorname,
        })
    }
}
