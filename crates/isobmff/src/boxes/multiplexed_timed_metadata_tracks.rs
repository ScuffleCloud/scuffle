use super::{BitRateBox, MetaDataSampleEntry};
use crate::{IsoBox, UnknownBox};

/// Boxed metadata sample entry
///
/// ISO/IEC 14496-12 - 12.9.4.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mebx", crate_path = crate)]
pub struct BoxedMetadataSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: MetaDataSampleEntry,
    /// The contained [`MetadataKeyTableBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub keys: MetadataKeyTableBox<'a>,
    /// The contained [`BitRateBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Metadata key table box
///
/// ISO/IEC 14496-12 - 12.9.4.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"keys", crate_path = crate)]
pub struct MetadataKeyTableBox<'a> {
    /// The contained key boxes. (not further implemented)
    #[iso_box(nested_box(collect_unknown))]
    pub key_boxes: Vec<UnknownBox<'a>>,
}
