use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use super::SampleEntry;
use crate::{FullBoxHeader, IsoBox, IsoSized};

/// Volumetric visual media header box
///
/// ISO/IEC 14496-12 - 12.10.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"vvhd", crate_path = crate)]
pub struct VolumetricVisualMediaHeaderBox {
    pub full_header: FullBoxHeader,
    // empty
}

/// Volumetric visual sample entry
///
/// ISO/IEC 14496-12 - 12.10.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct VolumetricVisualSampleEntry {
    pub sample_entry: SampleEntry,
    pub compressorname: [u8; 32],
}

impl<'a> Deserialize<'a> for VolumetricVisualSampleEntry {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let sample_entry = SampleEntry::deserialize(&mut reader)?;
        let compressorname = <[u8; 32]>::deserialize(&mut reader)?;

        Ok(Self {
            sample_entry,
            compressorname,
        })
    }
}

impl Serialize for VolumetricVisualSampleEntry {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(&mut writer)?;
        self.compressorname.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for VolumetricVisualSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size() + self.compressorname.size()
    }
}
