use crate::{FullBoxHeader, IsoBox};

/// Sub track box
///
/// ISO/IEC 14496-12 - 8.14.3
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"strk", crate_path = crate)]
pub struct SubTrackBox {
    #[iso_box(nested_box)]
    pub stri: SubTrackInformationBox,
    #[iso_box(nested_box)]
    pub strd: SubTrackDefinitionBox,
}

/// Sub track information box
///
/// ISO/IEC 14496-12 - 8.14.4
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"stri", crate_path = crate)]
pub struct SubTrackInformationBox {
    pub full_header: FullBoxHeader,
    pub switch_group: i16,
    pub alternate_group: i16,
    pub sub_track_id: u32,
    #[iso_box(repeated)]
    pub attribute_list: Vec<u32>,
}

/// Sub track definition box
///
/// ISO/IEC 14496-12 - 8.14.5
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"strd", crate_path = crate)]
pub struct SubTrackDefinitionBox {
    #[iso_box(nested_box(collect))]
    pub stsg: Vec<SubTrackSampleGroupBox>,
}

/// Sub track sample group box
///
/// ISO/IEC 14496-12 - 8.14.6
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"stsg", crate_path = crate)]
pub struct SubTrackSampleGroupBox {
    pub full_header: FullBoxHeader,
    pub grouping_type: [u8; 4],
    pub item_count: u16,
    #[iso_box(repeated)]
    pub group_description_index: Vec<u32>,
}
