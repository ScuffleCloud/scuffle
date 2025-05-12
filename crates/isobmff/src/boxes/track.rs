//! Track structure boxes defined in ISO/IEC 14496-12 - 8.3

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::{Brand, EditBox, MediaBox, MetaBox, UserDataBox};
use crate::{BoxHeader, BoxType, FullBoxHeader, IsoBox, UnknownBox};

/// Track box
///
/// ISO/IEC 14496-12 - 8.3.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trak", crate_path = crate)]
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
    pub edts: Option<EditBox>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
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

    const TYPE: BoxType = BoxType::FourCc(*b"tkhd");
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let creation_time = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let modification_time = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let track_id = u32::deserialize(&mut reader)?;
        u32::deserialize(&mut reader)?; // reserved
        let duration = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        u64::deserialize(&mut reader)?; // reserved

        let layer = i16::deserialize(&mut reader)?;
        let alternate_group = i16::deserialize(&mut reader)?;
        let volume = i16::deserialize(&mut reader)?;

        u16::deserialize(&mut reader)?; // reserved

        let mut matrix = [0; 9];
        for m in &mut matrix {
            *m = i32::deserialize(&mut reader)?;
        }

        let width = u32::deserialize(&mut reader)?;
        let height = u32::deserialize(&mut reader)?;

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
#[iso_box(box_type = b"tref", crate_path = crate)]
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
#[iso_box(box_type = b"trgr", crate_path = crate)]
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
#[iso_box(box_type = b"ttyp", crate_path = crate)]
pub struct TrackTypeBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
