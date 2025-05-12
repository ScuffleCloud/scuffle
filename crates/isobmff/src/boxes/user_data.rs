use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::{LoudnessBox, SubTrackBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox, Utf8String};

/// User data box
///
/// ISO/IEC 14496-12 - 8.10.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"udta", crate_path = crate)]
pub struct UserDataBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub cprt: Vec<CopyrightBox>,
    #[iso_box(nested_box(collect))]
    pub tsel: Option<TrackSelectionBox>,
    #[iso_box(nested_box(collect))]
    pub kind: Vec<KindBox>,
    #[iso_box(nested_box(collect))]
    pub strk: Vec<SubTrackBox>,
    #[iso_box(nested_box(collect))]
    pub ludt: Vec<LoudnessBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Copyright box
///
/// ISO/IEC 14496-12 - 8.10.2
#[derive(Debug)]
pub struct CopyrightBox {
    pub header: FullBoxHeader,
    pub language: [u8; 3],
    pub notice: Utf8String,
}

impl IsoBox for CopyrightBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"cprt";
}

impl<'a> Deserialize<'a> for CopyrightBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for CopyrightBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        // 0 xxxxx xxxxx xxxxx
        let language = u16::deserialize(&mut reader)?;
        let language = [
            ((language >> 10) & 0b11111) as u8,
            ((language >> 5) & 0b11111) as u8,
            (language & 0b11111) as u8,
        ];
        let notice = Utf8String::deserialize(&mut reader)?;

        Ok(Self {
            header: seed,
            language,
            notice,
        })
    }
}

/// Track selection box
///
/// ISO/IEC 14496-12 - 8.10.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"tsel", crate_path = crate)]
pub struct TrackSelectionBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub switch_group: i32,
    #[iso_box(repeated)]
    pub attribute_list: Vec<u32>,
}

/// Track kind box
///
/// ISO/IEC 14496-12 - 8.10.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"kind", crate_path = crate)]
pub struct KindBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub scheme_uri: Utf8String,
    pub value: Utf8String,
}
