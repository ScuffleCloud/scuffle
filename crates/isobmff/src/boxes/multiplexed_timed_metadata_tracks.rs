use super::{BitRateBox, MetaDataSampleEntry};
use crate::{IsoBox, UnknownBox};

/// Boxed metadata sample entry
///
/// ISO/IEC 14496-12 - 12.9.4.1
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mebx", crate_path = crate)]
pub struct BoxedMetadataSampleEntry<'a> {
    pub sample_entry: MetaDataSampleEntry,
    #[iso_box(nested_box)]
    pub keys: MetadataKeyTableBox<'a>,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Metadata key table box
///
/// ISO/IEC 14496-12 - 12.9.4.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"keys", crate_path = crate)]
pub struct MetadataKeyTableBox<'a> {
    #[iso_box(nested_box(collect_unknown))]
    pub key_boxes: Vec<UnknownBox<'a>>,
}
