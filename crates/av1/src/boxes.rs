//! ISO base media file format boxes for AV1.
//!
//! <https://aomediacodec.github.io/av1-isobmff>

use isobmff::boxes::VisualSampleEntry;
use isobmff::{IsoBox, UnknownBox};

use crate::AV1CodecConfigurationRecord;

/// AV1 Sample Entry
///
/// <https://aomediacodec.github.io/av1-isobmff/#av1sampleentry-section>
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"av01")]
pub struct AV1SampleEntry<'a> {
    /// The visual sample entry fields that this box inherits.
    pub sample_entry: VisualSampleEntry,
    /// The AV1 codec configuration box contained in this box.
    #[iso_box(nested_box)]
    pub av1c: AV1CodecConfigurationBox<'a>,
    /// Other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// AV1 Codec Configuration Box
///
/// <https://aomediacodec.github.io/av1-isobmff/#av1codecconfigurationbox-section>
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"av1C")]
pub struct AV1CodecConfigurationBox<'a> {
    /// The AV1 codec configuration record.
    pub av1_config: AV1CodecConfigurationRecord<'a>,
}
