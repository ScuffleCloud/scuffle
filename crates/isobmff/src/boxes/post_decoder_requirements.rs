use super::{OriginalFormatBox, SchemeInformationBox, SchemeTypeBox};
use crate::{BoxHeader, IsoBox};

/// Restricted scheme information box
///
/// ISO/IEC 14496-12 - 8.15.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"rinf", crate_path = crate)]
pub struct RestrictedSchemeInfoBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub original_format: OriginalFormatBox,
    #[iso_box(nested_box)]
    pub scheme_type: SchemeTypeBox,
    #[iso_box(nested_box(collect))]
    pub info: Option<SchemeInformationBox<'a>>,
}
