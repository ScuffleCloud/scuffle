use super::{LoudnessBox, SubTrackBox};
use crate::{FullBoxHeader, IsoBox, Langauge, UnknownBox, Utf8String};

/// User data box
///
/// ISO/IEC 14496-12 - 8.10.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"udta", crate_path = crate)]
pub struct UserDataBox<'a> {
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"cprt", crate_path = crate)]
pub struct CopyrightBox {
    pub full_header: FullBoxHeader,
    pub language: Langauge,
    pub notice: Utf8String,
}

/// Track selection box
///
/// ISO/IEC 14496-12 - 8.10.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tsel", crate_path = crate)]
pub struct TrackSelectionBox {
    pub full_header: FullBoxHeader,
    pub switch_group: i32,
    #[iso_box(repeated)]
    pub attribute_list: Vec<u32>,
}

/// Track kind box
///
/// ISO/IEC 14496-12 - 8.10.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"kind", crate_path = crate)]
pub struct KindBox {
    pub full_header: FullBoxHeader,
    pub scheme_uri: Utf8String,
    pub value: Utf8String,
}
