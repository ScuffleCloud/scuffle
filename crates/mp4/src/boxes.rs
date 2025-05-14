//! ISO base media file format boxes for MP4.
//!
//! ISO/IEC 14496-14 - 6

use isobmff::boxes::{AudioSampleEntry, SampleEntry, VisualSampleEntry};
use isobmff::{BoxHeader, FullBoxHeader, IsoBox};
use scuffle_bytes_util::BytesCow;

/// Object Descriptor Box
///
/// ISO/IEC 14496-14 - 6.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"iods")]
pub struct ObjectDescriptorBox<'a> {
    /// Header of the box
    #[iso_box(header)]
    pub header: FullBoxHeader,
    /// The object descriptor contained in this box.
    ///
    /// Defined in ISO/IEC 14496-1.
    pub od: BytesCow<'a>,
}

// TODO: Mpeg4MediaHeaderBox ISO/IEC 14496-14 - 6.6

/// MP4 visual sample description box
///
/// ISO/IEC 14496-14 - 6.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mp4v")]
pub struct MP4VisualSampleEntry<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The visual sample entry fields that this box inherits.
    pub sample_entry: VisualSampleEntry,
    /// The ES Descriptor for this stream.
    #[iso_box(nested_box)]
    pub es: ESDBox<'a>,
}

/// MP4 audio sample description box
///
/// ISO/IEC 14496-14 - 6.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mp4a")]
pub struct MP4AudioSampleEntry<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The audio sample entry fields that this box inherits.
    pub sample_entry: AudioSampleEntry,
    /// The ES Descriptor for this stream.
    #[iso_box(nested_box)]
    pub es: ESDBox<'a>,
}

/// Mpeg sample description box
///
/// ISO/IEC 14496-14 - 6.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mp4s")]
pub struct MpegSampleEntry<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The sample entry fields that this box inherits.
    pub sample_entry: SampleEntry,
    /// The ES Descriptor for this stream.
    #[iso_box(nested_box)]
    pub es: ESDBox<'a>,
}

/// MP4 audio enhancement sample description box
///
/// ISO/IEC 14496-14 - 6.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"m4ae")]
pub struct MP4AudioEnhancementSampleEntry<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// The audio sample entry fields that this box inherits.
    pub sample_entry: AudioSampleEntry,
    /// The ES Descriptor for this stream.
    #[iso_box(nested_box)]
    pub es: ESDBox<'a>,
}

/// Elementary Stream Descriptor Box
///
/// ISO/IEC 14496-14 - 6.7
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"esds")]
pub struct ESDBox<'a> {
    /// Header of the box.
    #[iso_box(header)]
    pub header: FullBoxHeader,
    /// The ES Descriptor for this stream.
    ///
    /// Defined in ISO/IEC 14496-1.
    pub es: BytesCow<'a>,
}
