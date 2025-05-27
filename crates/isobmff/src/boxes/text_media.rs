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
    pub sample_entry: PlainTextSampleEntry,
    pub content_encoding: Utf8String,
    pub mime_format: Utf8String,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub txtc: Option<TextConfigBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
