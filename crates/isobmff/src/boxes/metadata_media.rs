use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use super::{BitRateBox, SampleEntry};
use crate::{FullBoxHeader, IsoBox, IsoSized, UnknownBox, Utf8List, Utf8String};

/// Metadata sample entry
///
/// ISO/IEC 14496-12 - 12.3.3
///
/// Sub boxes:
/// - [`btrt`](super::BitRateBox)
/// - Any other boxes
#[derive(Debug, PartialEq, Eq)]
pub struct MetaDataSampleEntry {
    /// The sample entry that this box inherits from.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"metx", crate_path = crate)]
pub struct XMLMetaDataSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: MetaDataSampleEntry,
    /// A MIME type which identifies the content encoding of the timed metadata.
    /// It is defined in the same way as for an [`ItemInfoEntry`](super::ItemInfoEntry) in this document.
    /// If not present (an empty string is supplied) the timed metadata is not encoded.
    /// An example for this field is 'application/zip'.
    /// Note that no MIME types for BiM [ISO/IEC 23001-1] and TeM [ISO/IEC 15938-1] currently exist.
    /// Thus, the experimental MIME types 'application/x-BiM' and 'text/x-TeM' shall be used to identify
    /// these encoding mechanisms.
    pub content_encoding: Utf8String,
    /// One or more XML namespaces to which the sample documents conform. When
    /// used for metadata, this is needed for identifying its type, e.g. gBSD or AQoS [MPEG-21-7] and for
    /// decoding using XML aware encoding mechanisms such as BiM.
    pub namespace: Utf8List,
    /// Zero or more URLs for XML schema(s) to which the sample document
    /// conforms. If there is one namespace and one schema, then this field shall be the URL of the one
    /// schema. If there is more than one namespace, then the syntax of this field shall adhere to that
    /// for xsi:​schemaLocation attribute as defined by XML. When used for metadata, this is needed for
    /// decoding of the timed metadata by XML aware encoding mechanisms such as BiM.
    pub schema_location: Utf8List,
    /// The contained [`BitRateBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Text config box
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"txtC", crate_path = crate)]
pub struct TextConfigBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The initial text of each document which is prepended before the contents of each sync sample.
    pub text_config: Utf8String,
}

/// Text metadata sample entry
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mett", crate_path = crate)]
pub struct TextMetaDataSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: MetaDataSampleEntry,
    /// A MIME type which identifies the content encoding of the timed metadata.
    /// It is defined in the same way as for an [`ItemInfoEntry`](super::ItemInfoEntry) in this document.
    /// If not present (an empty string is supplied) the timed metadata is not encoded.
    /// An example for this field is 'application/zip'.
    /// Note that no MIME types for BiM [ISO/IEC 23001-1] and TeM [ISO/IEC 15938-1] currently exist.
    /// Thus, the experimental MIME types 'application/x-BiM' and 'text/x-TeM' shall be used to identify
    /// these encoding mechanisms.
    pub content_encoding: Utf8String,
    /// A MIME type which identifies the content format of the samples.
    /// Examples for this field include 'text/html' and 'text/plain'.
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

/// MIME box
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mime", crate_path = crate)]
pub struct MIMEBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A string corresponding to the MIME type each XML document carried in the stream
    /// would have if it were delivered on its own, possibly including sub-parameters.
    pub content_type: Utf8String,
}

/// URI box
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"uri ", crate_path = crate)]
pub struct URIBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// A URI formatted according to the rules in 6.3.3.
    pub the_uri: Utf8String,
}

/// URI init box
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"uriI", crate_path = crate)]
pub struct URIInitBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Opaque data whose form is defined in the documentation of the URI form.
    pub uri_initialization_data: BytesCow<'a>,
}

/// URI meta sample entry
///
/// ISO/IEC 14496-12 - 12.3.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"urim", crate_path = crate)]
pub struct URIMetaSampleEntry<'a> {
    /// The sample entry that this box inherits from.
    pub sample_entry: MetaDataSampleEntry,
    /// The contained [`BitRateBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub btrt: Option<BitRateBox>,
    /// The contained [`MIMEBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub the_label: URIBox,
    /// The contained [`URIInitBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub init: Option<URIInitBox<'a>>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
