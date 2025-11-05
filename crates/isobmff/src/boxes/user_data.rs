use super::{LoudnessBox, SubTrackBox};
use crate::{FullBoxHeader, IsoBox, Langauge, UnknownBox, Utf8String};

/// User data box
///
/// ISO/IEC 14496-12 - 8.10.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"udta", crate_path = crate)]
pub struct UserDataBox<'a> {
    /// The contained [`CopyrightBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub cprt: Vec<CopyrightBox>,
    /// The contained [`TrackSelectionBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub tsel: Option<TrackSelectionBox>,
    /// The contained [`KindBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub kind: Vec<KindBox>,
    /// The contained [`SubTrackBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub strk: Vec<SubTrackBox>,
    /// The contained [`LoudnessBox`]es. (any quantity)
    #[iso_box(nested_box(collect))]
    pub ludt: Vec<LoudnessBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Copyright box
///
/// ISO/IEC 14496-12 - 8.10.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"cprt", crate_path = crate)]
pub struct CopyrightBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Declares the language code for the following text.
    pub language: Langauge,
    /// Gives a copyright notice.
    pub notice: Utf8String,
}

/// Track selection box
///
/// ISO/IEC 14496-12 - 8.10.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tsel", crate_path = crate)]
pub struct TrackSelectionBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that specifies a group or collection of tracks. If this field is 0 (default value)
    /// or if the [`TrackSelectionBox`] is absent there is no information on whether the track can be used for
    /// switching during playing or streaming. If this integer is not 0 it shall be the same for tracks that can
    /// be used for switching between each other. Tracks that belong to the same switch group shall belong
    /// to the same alternate group. A switch group may have only one member.
    pub switch_group: i32,
    /// A list, to the end of the box, of attributes. The attributes in this list should be used
    /// as descriptions of tracks or differentiation criteria for tracks in the same alternate or switch
    /// group. Each differentiating attribute is associated with a pointer to the field or information that
    /// distinguishes the track.
    #[iso_box(repeated)]
    pub attribute_list: Vec<[u8; 4]>,
}

/// Track kind box
///
/// ISO/IEC 14496-12 - 8.10.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"kind", crate_path = crate)]
pub struct KindBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Declares either the identifier of the kind, if no value follows, or the identifier of the naming
    /// scheme for the following value.
    pub scheme_uri: Utf8String,
    /// A name from the declared scheme.
    pub value: Utf8String,
}
