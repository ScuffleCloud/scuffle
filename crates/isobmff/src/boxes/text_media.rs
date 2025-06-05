use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, ZeroCopyReader};

use super::{BitRateBox, SampleEntry, TextConfigBox};
use crate::{IsoBox, IsoSized, UnknownBox, Utf8String};

/// Plain Text sample entry
///
/// ISO/IEC 14496-12 - 12.5.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct PlainTextSampleEntry {
    /// The sample entry that this box inherits from.
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for PlainTextSampleEntry {
    fn deserialize<R: ZeroCopyReader<'a>>(reader: R) -> std::io::Result<Self> {
        Ok(Self {
            sample_entry: SampleEntry::deserialize(reader)?,
        })
    }
}

impl Serialize for PlainTextSampleEntry {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(writer)
    }
}

impl IsoSized for PlainTextSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
    }
}

/// Simple text sample entry
///
/// ISO/IEC 14496-12 - 12.5.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stxt", crate_path = crate)]
pub struct SimpleTextSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: PlainTextSampleEntry,
    /// A MIME type which identifies the content encoding of the timed text. It is
    /// defined in the same way as for an [`ItemInfoEntry`](super::ItemInfoEntry) in this document.
    /// If not present (an empty string is supplied) the timed text is not encoded.
    /// An example for this field is 'application/zip'.
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
