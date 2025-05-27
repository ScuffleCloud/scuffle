//! ISO base media file format boxes for HEVC/h265.
//!
//! ISO/IEC 14496-15 - 8.4

use isobmff::boxes::{MPEG4ExtensionDescriptorsBox, VisualSampleEntry};
use isobmff::{IsoBox, UnknownBox};

use crate::HEVCDecoderConfigurationRecord;

/// HEVC Configuration Box
///
/// ISO/IEC 14496-15 - 8.4.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"hvcC")]
pub struct HEVCConfigurationBox<'a> {
    /// The HEVC decoder configuration record contained in this box.
    pub hevc_config: HEVCDecoderConfigurationRecord<'a>,
}

impl<'a> HEVCConfigurationBox<'a> {
    /// Creates a new HEVC configuration box.
    pub fn new(hevc_config: HEVCDecoderConfigurationRecord<'a>) -> Self {
        Self { hevc_config }
    }
}

/// HEVC Sample Entry
///
/// ISO/IEC 14496-15 - 8.4.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"hvc1")]
pub struct HEVCSampleEntryHvc1<'a> {
    /// The visual sample entry fields that this box inherits.
    pub sample_entry: VisualSampleEntry,
    /// The HEVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: HEVCConfigurationBox<'a>,
    /// The optional MPEG-4 extension descriptors box contained in this box.
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox<'a>>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// HEVC Sample Entry
///
/// ISO/IEC 14496-15 - 8.4.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"hev1")]
pub struct HEVCSampleEntryHev1<'a> {
    /// The visual sample entry fields that this box inherits.
    pub sample_entry: VisualSampleEntry,
    /// The HEVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: HEVCConfigurationBox<'a>,
    /// The optional MPEG-4 extension descriptors box contained in this box.
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox<'a>>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}
