use scuffle_bytes_util::BytesCow;

use crate::IsoBox;

/// MPEG-4 extension descriptors box
///
/// ISO/IEC 14496-15 - 5.4.2
///
/// This box is used by multiple codecs and not specific to a codec.
/// Therefore, it is part of this crate.
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"m4ds", crate_path = crate)]
pub struct MPEG4ExtensionDescriptorsBox<'a> {
    /// A descriptor that should be placed in the `ElementaryStreamDescriptor` when this
    /// stream is used in an MPEG-4 systems context. This does not include `SLConfigDescriptor` or
    /// `DecoderConfigDescriptor`, but includes the other descriptors in order to be placed after
    /// the `SLConfigDescriptor`.
    pub descr: BytesCow<'a>,
}
