//! Track structure boxes defined in ISO/IEC 14496-12 - 8.3

use byteorder::ReadBytesExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::{Brand, MediaBox, UserDataBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox};

/// Track box
///
/// ISO/IEC 14496-12 - 8.3.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trak", crate_path = "crate")]
pub struct TrackBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub tkhd: TrackHeaderBox,
    #[iso_box(nested_box(collect))]
    pub tref: Option<TrackReferenceBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub trgr: Option<TrackGroupBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub ttyp: Option<TrackTypeBox>,
    #[iso_box(nested_box)]
    pub mdia: MediaBox<'a>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Track header box
///
/// ISO/IEC 14496-12 - 8.3.2
#[derive(Debug)]
pub struct TrackHeaderBox {
    pub header: FullBoxHeader,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer: i16,
    pub alternate_group: i16,
    pub volume: i16,
    pub matrix: [i32; 9],
    pub width: u32,
    pub height: u32,
}

// Manual implementation because conditional fields are not supported in the macro

impl IsoBox for TrackHeaderBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"tkhd";
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let creation_time = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };
        let modification_time = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };
        let track_id = reader.as_std().read_u32::<byteorder::BigEndian>()?;
        reader.as_std().read_u32::<byteorder::BigEndian>()?; // reserved
        let duration = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };

        reader.as_std().read_u64::<byteorder::BigEndian>()?; // reserved

        let layer = reader.as_std().read_i16::<byteorder::BigEndian>()?;
        let alternate_group = reader.as_std().read_i16::<byteorder::BigEndian>()?;
        let volume = reader.as_std().read_i16::<byteorder::BigEndian>()?;

        reader.as_std().read_u16::<byteorder::BigEndian>()?; // reserved

        let mut matrix = [0; 9];
        for m in &mut matrix {
            *m = reader.as_std().read_i32::<byteorder::BigEndian>()?;
        }

        let width = reader.as_std().read_u32::<byteorder::BigEndian>()?;
        let height = reader.as_std().read_u32::<byteorder::BigEndian>()?;

        Ok(Self {
            header: seed,
            creation_time,
            modification_time,
            track_id,
            duration,
            layer,
            alternate_group,
            volume,
            matrix,
            width,
            height,
        })
    }
}

impl<'a> Deserialize<'a> for TrackHeaderBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

/// Track reference box
///
/// ISO/IEC 14496-12 - 8.3.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"tref", crate_path = "crate")]
pub struct TrackReferenceBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect_unknown))]
    pub track_reference_type: Vec<UnknownBox<'a>>,
}

/// Track group box
///
/// ISO/IEC 14496-12 - 8.3.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trgr", crate_path = "crate")]
pub struct TrackGroupBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect_unknown))]
    pub track_group_type: Vec<UnknownBox<'a>>,
}

/// Track type box
///
/// ISO/IEC 14496-12 - 8.3.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ttyp", crate_path = "crate")]
pub struct TrackTypeBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
