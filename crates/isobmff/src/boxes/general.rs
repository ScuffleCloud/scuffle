//! File structure and general boxes defined in ISO/IEC 14496-12 - 8.1

use std::fmt::Debug;

use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized};

/// Media data box
///
/// ISO/IEC 14496-12 - 8.1.1
#[derive(IsoBox)]
#[iso_box(box_type = b"mdat", crate_path = crate)]
pub struct MediaDataBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub data: BytesCow<'a>,
}

impl Debug for MediaDataBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaDataBox")
            .field("header", &self.header)
            .field("data.len", &self.data.len())
            .finish()
    }
}

/// Free space box
///
/// ISO/IEC 14496-12 - 8.1.2
#[derive(IsoBox)]
#[iso_box(box_type = b"free", crate_path = crate)]
pub struct FreeSpaceBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub data: BytesCow<'a>,
}

impl Debug for FreeSpaceBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FreeSpaceBox")
            .field("header", &self.header)
            .field("data.len", &self.data.len())
            .finish()
    }
}

/// Free space box
///
/// ISO/IEC 14496-12 - 8.1.2
#[derive(IsoBox)]
#[iso_box(box_type = b"skip", crate_path = crate)]
pub struct SkipBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub data: BytesCow<'a>,
}

impl Debug for SkipBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkipBox")
            .field("header", &self.header)
            .field("data.len", &self.data.len())
            .finish()
    }
}

/// Progressive download information box
///
/// ISO/IEC 14496-12 - 8.1.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"pdin", crate_path = crate)]
pub struct ProgressiveDownloadInfoBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(repeated)]
    pub properties: Vec<ProgressiveDownloadInfoBoxProperties>,
}

#[derive(Debug)]
pub struct ProgressiveDownloadInfoBoxProperties {
    pub rate: u32,
    pub initial_delay: u32,
}

impl<'a> Deserialize<'a> for ProgressiveDownloadInfoBoxProperties {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let rate = u32::deserialize(&mut reader)?;
        let initial_delay = u32::deserialize(&mut reader)?;

        Ok(ProgressiveDownloadInfoBoxProperties { rate, initial_delay })
    }
}

impl Serialize for ProgressiveDownloadInfoBoxProperties {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.rate.serialize(&mut writer)?;
        self.initial_delay.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for ProgressiveDownloadInfoBoxProperties {
    fn size(&self) -> usize {
        self.rate.size() + self.initial_delay.size()
    }
}

/// Identified media data box
///
/// ISO/IEC 14496-12 - 8.1.4
#[derive(IsoBox)]
#[iso_box(box_type = b"imda", crate_path = crate)]
pub struct IdentifiedMediaDataBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub imda_identifier: u32,
    pub data: BytesCow<'a>,
}

impl Debug for IdentifiedMediaDataBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentifiedMediaDataBox")
            .field("header", &self.header)
            .field("imda_identifier", &self.imda_identifier)
            .field("data.len", &self.data.len())
            .finish()
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::zero_copy::{Deserialize, Slice};

    use super::MediaDataBox;
    use crate::{BoxHeaderProperties, BoxSize};

    #[test]
    fn demux_mdat() {
        #[rustfmt::skip]
        let data = [
            0x00, 0x00, 0x00, 0x0C, // size
            b'm', b'd', b'a', b't', // type
            0x42, 0x00, 0x42, 0x00, // data
            0x01,
        ];

        let mdat = MediaDataBox::deserialize(Slice::from(&data[..])).unwrap();
        assert_eq!(mdat.header.size, BoxSize::Short(12));
        assert!(mdat.header.box_type.is_four_cc(b"mdat"));
        assert_eq!(mdat.header.payload_size(), Some(4));
        assert_eq!(mdat.data.len(), 4);
        assert_eq!(mdat.data.as_bytes()[0], 0x42);
        assert_eq!(mdat.data.as_bytes()[1], 0x00);
        assert_eq!(mdat.data.as_bytes()[2], 0x42);
        assert_eq!(mdat.data.as_bytes()[3], 0x00);
    }
}
