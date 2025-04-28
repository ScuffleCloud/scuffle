use super::Brand;
use crate::{BoxHeader, IsoBox};

/// Type combination box
///
/// ISO/IEC 14496-12 - 4.4
#[derive(IsoBox)]
#[iso_box(box_type = b"tyco", crate_path = "crate")]
pub struct Tyco {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(remaining, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
