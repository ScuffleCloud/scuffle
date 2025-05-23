use super::OriginalFormatBox;
use crate::IsoBox;

/// Complete track information box
///
/// ISO/IEC 14496-12 - 8.17.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"cinf", crate_path = crate)]
pub struct CompleteTrackInfoBox {
    #[iso_box(nested_box)]
    pub original_format: OriginalFormatBox,
}
