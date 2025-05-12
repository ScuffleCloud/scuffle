//! ISO base media file format boxes for AVC/h264.
//!
//! ISO/IEC 14496-15 - 5.4

use isobmff::boxes::{MPEG4ExtensionDescriptorsBox, VisualSampleEntry};
use isobmff::{BoxHeader, IsoBox, UnknownBox};

use crate::AVCDecoderConfigurationRecord;

/// AVC configuration box
///
/// ISO/IEC 14496-15 - 5.4.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avcC")]
pub struct AVCConfigurationBox<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The AVC decoder configuration record.
    pub avc_config: AVCDecoderConfigurationRecord<'a>,
}

// This doesn't make sense to me, the following 4 box types (avc1 - avc4) are all the same

/// AVC sample entry
///
/// ISO/IEC 14496-15 - 5.4.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc1")]
pub struct AVCSampleEntry1<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The visual sample entry fields that this box inherits.
    pub visual_sample_entry: VisualSampleEntry,
    /// The AVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    /// The optional MPEG-4 extension descriptors box contained in this box.
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// AVC sample entry
///
/// ISO/IEC 14496-15 - 5.4.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc2")]
pub struct AVCSampleEntry2<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The visual sample entry fields that this box inherits.
    pub visual_sample_entry: VisualSampleEntry,
    /// The AVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    /// The optional MPEG-4 extension descriptors box contained in this box.
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// AVC sample entry
///
/// ISO/IEC 14496-15 - 5.4.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc3")]
pub struct AVCSampleEntry3<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The visual sample entry fields that this box inherits.
    pub visual_sample_entry: VisualSampleEntry,
    /// The AVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    /// The optional MPEG-4 extension descriptors box contained in this box.
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// AVC sample entry
///
/// ISO/IEC 14496-15 - 5.4.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc4")]
pub struct AVCSampleEntry4<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The visual sample entry fields that this box inherits.
    pub visual_sample_entry: VisualSampleEntry,
    /// The AVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    /// The optional MPEG-4 extension descriptors box contained in this box.
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}

/// AVC parameter sample entry
///
/// ISO/IEC 14496-15 - 5.4.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avcp")]
pub struct AVCParameterSampleEntry<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The visual sample entry fields that this box inherits.
    pub visual_sample_entry: VisualSampleEntry,
    /// The AVC configuration box contained in this box.
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    /// Any other boxes contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub sub_boxes: Vec<UnknownBox<'a>>,
}
