use crate::{FullBoxHeader, IsoBox};

/// Sub track box
///
/// ISO/IEC 14496-12 - 8.14.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"strk", crate_path = crate)]
pub struct SubTrackBox {
    /// The contained [`SubTrackInformationBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub stri: SubTrackInformationBox,
    /// The contained [`SubTrackDefinitionBox`]. (optional)
    ///
    /// According to the official specification this is mandatory, but
    /// there is one official sample file in which it is not present.
    /// See <https://github.com/MPEGGroup/FileFormatConformance/issues/155>.
    #[iso_box(nested_box(collect))]
    pub strd: Option<SubTrackDefinitionBox>,
}

/// Sub track information box
///
/// ISO/IEC 14496-12 - 8.14.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stri", crate_path = crate)]
pub struct SubTrackInformationBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that specifies a group or collection of tracks and/or sub tracks. If this field
    /// is 0 (default value), then there is no information on whether the sub track can be used for switching
    /// during playing or streaming. If this integer is not 0 it shall be the same for tracks and/or sub tracks
    /// that can be used for switching between each other. Tracks that belong to the same switch group
    /// shall belong to the same alternate group. A switch group may have only one member.
    pub switch_group: i16,
    /// An integer that specifies a group or collection of tracks and/or sub tracks. If this
    /// field is 0 (default value), then there is no information on possible relations to other tracks/sub-
    /// tracks. If this field is not 0, it should be the same for tracks/sub-tracks that contain alternate data
    /// for one another and different for tracks/sub-tracks belonging to different such groups. Only one
    /// track/sub-track within an alternate group should be played or streamed at any one time.
    pub alternate_group: i16,
    /// An integer. A non-zero value uniquely identifies the sub track locally within the track.
    /// A zero value (default) means that sub_track_ID is not assigned.
    pub sub_track_id: u32,
    /// A list, to the end of the box, of attributes. The attributes in this list should be used as
    /// descriptions of sub tracks or differentiating criteria for tracks and sub tracks in the same alternate
    /// or switch group.
    #[iso_box(repeated)]
    pub attribute_list: Vec<u32>,
}

/// Sub track definition box
///
/// ISO/IEC 14496-12 - 8.14.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"strd", crate_path = crate)]
pub struct SubTrackDefinitionBox {
    /// The contained [`SubTrackSampleGroupBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub stsg: Vec<SubTrackSampleGroupBox>,
}

/// Sub track sample group box
///
/// ISO/IEC 14496-12 - 8.14.6
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stsg", crate_path = crate)]
pub struct SubTrackSampleGroupBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that identifies the sample grouping. The value shall be the same as in the
    /// corresponding SampleToGroupBox and [`SampleGroupDescriptionBox`](super::SampleGroupDescriptionBox).
    pub grouping_type: [u8; 4],
    /// Counts the number of sample groups listed in this box.
    pub item_count: u16,
    /// An integer that gives the index of the sample group entry which describes
    /// the samples in the group.
    #[iso_box(repeated)]
    pub group_description_index: Vec<u32>,
}
