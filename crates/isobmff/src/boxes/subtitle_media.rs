use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, ZeroCopyReader};

use super::{BitRateBox, SampleEntry, TextConfigBox};
use crate::{FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8List, Utf8String};

/// Subtitle media header box
///
/// ISO/IEC 14496-12 - 12.6.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sthd", crate_path = crate)]
pub struct SubtitleMediaHeaderBox {
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
    pub sample_entry: SubtitleSampleEntry,
    pub namespace: Utf8List,
    pub schema_location: Utf8List,
    pub auxiliary_mime_types: Utf8List,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Text subtitle sample entry
///
/// ISO/IEC 14496-12 - 12.6.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"sbtt", crate_path = crate)]
pub struct TextSubtitleSampleEntry<'a> {
    pub sample_entry: SubtitleSampleEntry,
    pub content_encoding: Utf8String,
    pub mime_format: Utf8String,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub txtc: Option<TextConfigBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
