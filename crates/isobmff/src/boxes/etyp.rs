use super::Tyco;
use crate::{BoxHeader, IsoBox};

/// Extended type box
///
/// ISO/IEC 14496-12 - 4.4
#[derive(IsoBox)]
#[iso_box(box_type = b"etyp", crate_path = "crate")]
pub struct Etyp {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(remaining)]
    pub compatible_combinations: Vec<Tyco>,
}
