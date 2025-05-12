//! Other boxes that are used by multiple codecs or are not specific to a codec.

use crate::{BoxHeader, IsoBox};

/// MPEG-4 extension descriptors box
///
/// ISO/IEC 14496-15 - 5.4.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"m4ds", crate_path = crate)]
pub struct MPEG4ExtensionDescriptorsBox {
    /// Header of the box.
    #[iso_box(header)]
    pub header: BoxHeader,
    /// A descriptor that should be placed in the `ElementaryStreamDescriptor` when this
    /// stream is used in an MPEG-4 systems context. This does not include `SLConfigDescriptor` or
    /// `DecoderConfigDescriptor`, but includes the other descriptors in order to be placed after
    /// the `SLConfigDescriptor`.
    #[iso_box(repeated)]
    pub descr: Vec<u8>,
}
