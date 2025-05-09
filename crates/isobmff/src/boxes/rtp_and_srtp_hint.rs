use super::{SchemeInformationBox, SchemeTypeBox};
use crate::{BoxHeader, IsoBox};

/// SRTP process box
///
/// ISO/IEC 14496-12 - 9.1.2.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"srpp", crate_path = crate)]
pub struct SRTPProcessBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub encryption_algorithm_rtp: [u8; 4],
    pub encryption_algorithm_rtcp: [u8; 4],
    pub integrity_algorithm_rtp: [u8; 4],
    pub integrity_algorithm_rtcp: [u8; 4],
    #[iso_box(nested_box)]
    pub scheme_type_box: SchemeTypeBox,
    #[iso_box(nested_box)]
    pub info: SchemeInformationBox<'a>,
}
