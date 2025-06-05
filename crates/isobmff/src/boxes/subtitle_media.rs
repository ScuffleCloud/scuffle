use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, ZeroCopyReader};

use super::{BitRateBox, SampleEntry, TextConfigBox};
use crate::{FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8List, Utf8String};

/// Subtitle media header box
///
/// ISO/IEC 14496-12 - 12.6.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sthd", crate_path = crate)]
pub struct SubtitleMediaHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    // empty
}

/// Subtitle sample entry
///
/// ISO/IEC 14496-12 - 12.6.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct SubtitleSampleEntry {
    /// The sample entry that this box inherits from.
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for SubtitleSampleEntry {
    fn deserialize<R: ZeroCopyReader<'a>>(reader: R) -> std::io::Result<Self> {
        Ok(Self {
            sample_entry: SampleEntry::deserialize(reader)?,
        })
    }
}

impl Serialize for SubtitleSampleEntry {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(writer)
    }
}

impl IsoSized for SubtitleSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
    }
}

/// XML subtitle sample entry
///
/// ISO/IEC 14496-12 - 12.6.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stpp", crate_path = crate)]
pub struct XMLSubtitleSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: SubtitleSampleEntry,
    /// One or more XML namespaces to which the sample documents conform. When used for
    /// metadata, this is needed for identifying its type, e.g. gBSD or AQoS [MPEG-21-7] and for decoding
    /// using XML aware encoding mechanisms such as BiM.
    pub namespace: Utf8List,
    /// Zero or more URLs for XML schema(s) to which the sample document conforms. If
    /// there is one namespace and one schema, then this field shall be the URL of the one schema. If there is
    /// more than one namespace, then the syntax of this field shall adhere to that for xsi:​schemaLocation
    /// attribute as defined by XML. When used for metadata, this is needed for decoding of the timed
    /// metadata by XML aware encoding mechanisms such as BiM.
    pub schema_location: Utf8List,
    /// The media type of all auxiliary resources, such as images and fonts, if present, stored as subtitle sub-samples.
    pub auxiliary_mime_types: Utf8List,
    /// The contained [`BitRateBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Text subtitle sample entry
///
/// ISO/IEC 14496-12 - 12.6.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sbtt", crate_path = crate)]
pub struct TextSubtitleSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: SubtitleSampleEntry,
    /// A MIME type which identifies the content encoding of the subtitles. It is
    /// defined in the same way as for an ItemInfoEntry in this document. If not present (an empty string
    /// is supplied) the subtitle samples are not encoded. An example for this field is 'application/zip'.
    pub content_encoding: Utf8String,
    /// A MIME type which identifies the content format of the samples. Examples for
    /// this field include 'text/html' and 'text/plain'.
    pub mime_format: Utf8String,
    /// The contained [`BitRateBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    /// The contained [`TextConfigBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub txtc: Option<TextConfigBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
