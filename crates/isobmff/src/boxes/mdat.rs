use crate::{BoxHeader, IsoBox};

#[derive(IsoBox)]
#[iso_box(box_type = b"mdat", crate_path = "crate")]
pub struct Mdat {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(remaining)]
    pub data: Vec<u8>,
}
