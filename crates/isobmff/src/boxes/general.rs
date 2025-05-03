//! File structure and general boxes defined in ISO/IEC 14496-12 - 8.1

use std::fmt::Debug;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::zero_copy::Deserialize;

use crate::{BoxHeader, FullBoxHeader, IsoBox};

/// Media data box
///
/// ISO/IEC 14496-12 - 8.1.1
#[derive(IsoBox)]
#[iso_box(box_type = b"mdat", crate_path = "crate")]
pub struct MediaDataBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}

impl Debug for MediaDataBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaDataBox")
            .field("header", &self.header)
            .field("data(size)", &self.data.len())
            .finish()
    }
}

/// Free space box
///
/// ISO/IEC 14496-12 - 8.1.2
#[derive(IsoBox)]
#[iso_box(box_type = b"free", crate_path = "crate")]
pub struct FreeSpaceBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}

impl Debug for FreeSpaceBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FreeSpaceBox")
            .field("header", &self.header)
            .field("data(size)", &self.data.len())
            .finish()
    }
}

/// Free space box
///
/// ISO/IEC 14496-12 - 8.1.2
#[derive(IsoBox)]
#[iso_box(box_type = b"skip", crate_path = "crate")]
pub struct SkipBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}

impl Debug for SkipBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkipBox")
            .field("header", &self.header)
            .field("data(size)", &self.data.len())
            .finish()
    }
}

/// Progressive download information box
///
/// ISO/IEC 14496-12 - 8.1.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"pdin", crate_path = "crate")]
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
        let rate = reader.as_std().read_u32::<byteorder::BigEndian>()?;
        let initial_delay = reader.as_std().read_u32::<byteorder::BigEndian>()?;

        Ok(ProgressiveDownloadInfoBoxProperties { rate, initial_delay })
    }
}

/// Identified media data box
///
/// ISO/IEC 14496-12 - 8.1.4
#[derive(IsoBox)]
#[iso_box(box_type = b"imda", crate_path = "crate")]
pub struct IdentifiedMediaDataBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub imda_identifier: u32,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}

impl Debug for IdentifiedMediaDataBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentifiedMediaDataBox")
            .field("header", &self.header)
            .field("imda_identifier", &self.imda_identifier)
            .field("data(size)", &self.data.len())
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
        assert_eq!(mdat.data[0], 0x42);
        assert_eq!(mdat.data[1], 0x00);
        assert_eq!(mdat.data[2], 0x42);
        assert_eq!(mdat.data[3], 0x00);
    }
}
