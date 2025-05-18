use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use super::{BitRateBox, SampleEntry};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8List, Utf8String};

/// Metadata sample entry
///
/// ISO/IEC 14496-12 - 12.3.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug)]
pub struct MetaDataSampleEntry {
    pub sample_entry: SampleEntry,
}

impl<'a> Deserialize<'a> for MetaDataSampleEntry {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            sample_entry: SampleEntry::deserialize(reader)?,
        })
    }
}

impl Serialize for MetaDataSampleEntry {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.sample_entry.serialize(writer)
    }
}

impl IsoSized for MetaDataSampleEntry {
    fn size(&self) -> usize {
        self.sample_entry.size()
    }
}

/// XML metadata sample entry
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"metx", crate_path = crate)]
pub struct XMLMetaDataSampleEntry<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub sample_entry: MetaDataSampleEntry,
    pub content_encoding: Utf8String,
    pub namespace: Utf8List,
    pub schema_location: Utf8List,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Text config box
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"txtC", crate_path = crate)]
pub struct TextConfigBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub text_config: Utf8String,
}

/// Text metadata sample entry
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mett", crate_path = crate)]
pub struct TextMetaDataSampleEntry<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub sample_entry: MetaDataSampleEntry,
    pub content_encoding: Utf8String,
    pub mime_format: Utf8String,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box(collect))]
    pub txtc: Option<TextConfigBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mime", crate_path = crate)]
pub struct MIMEBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub content_type: Utf8String,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"uri ", crate_path = crate)]
pub struct URIBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub the_uri: Utf8String,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"uriI", crate_path = crate)]
pub struct URIInitBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub uri_initialization_data: BytesCow<'a>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"urim", crate_path = crate)]
pub struct URIMetaSampleEntry<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub sample_entry: MetaDataSampleEntry,
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    #[iso_box(nested_box)]
    pub the_label: URIBox,
    #[iso_box(nested_box(collect))]
    pub init: Option<URIInitBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
