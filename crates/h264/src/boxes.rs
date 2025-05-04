// TODO
#![allow(missing_docs)]

use isobmff::boxes::{BitRateBox, CleanApertureBox, ColourInformationBox, PixelAspectRatioBox, VisualSampleEntry};
use isobmff::{BoxHeader, IsoBox};

use crate::AVCDecoderConfigurationRecord;

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avcC")]
pub struct AVCConfigurationBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub avc_config: AVCDecoderConfigurationRecord<'a>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"m4ds")]
pub struct MPEG4ExtensionDescriptorsBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub descr: Vec<u8>,
}

// This doesn't make sense to me, the following 4 boxe types (avc1 - avc4) are all the same

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc1")]
pub struct AVCSampleEntry1<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub visual_sample_entry: VisualSampleEntry,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub clap: Option<CleanApertureBox>,
    #[iso_box(nested_box(collect))]
    pub pasp: Option<PixelAspectRatioBox>,
    #[iso_box(nested_box(collect))]
    pub colr: Option<ColourInformationBox<'a>>,
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<isobmff::UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc2")]
pub struct AVCSampleEntry2<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub visual_sample_entry: VisualSampleEntry,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub clap: Option<CleanApertureBox>,
    #[iso_box(nested_box(collect))]
    pub pasp: Option<PixelAspectRatioBox>,
    #[iso_box(nested_box(collect))]
    pub colr: Option<ColourInformationBox<'a>>,
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<isobmff::UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc3")]
pub struct AVCSampleEntry3<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub visual_sample_entry: VisualSampleEntry,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub clap: Option<CleanApertureBox>,
    #[iso_box(nested_box(collect))]
    pub pasp: Option<PixelAspectRatioBox>,
    #[iso_box(nested_box(collect))]
    pub colr: Option<ColourInformationBox<'a>>,
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<isobmff::UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avc4")]
pub struct AVCSampleEntry4<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub visual_sample_entry: VisualSampleEntry,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub clap: Option<CleanApertureBox>,
    #[iso_box(nested_box(collect))]
    pub pasp: Option<PixelAspectRatioBox>,
    #[iso_box(nested_box(collect))]
    pub colr: Option<ColourInformationBox<'a>>,
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub mpeg4_extension: Option<MPEG4ExtensionDescriptorsBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<isobmff::UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"avcp")]
pub struct AVCParameterSampleEntry<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub visual_sample_entry: VisualSampleEntry,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub clap: Option<CleanApertureBox>,
    #[iso_box(nested_box(collect))]
    pub pasp: Option<PixelAspectRatioBox>,
    #[iso_box(nested_box(collect))]
    pub colr: Option<ColourInformationBox<'a>>,
    #[iso_box(nested_box)]
    pub config: AVCConfigurationBox<'a>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<isobmff::UnknownBox<'a>>,
}
