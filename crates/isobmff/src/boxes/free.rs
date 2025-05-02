use crate::{BoxHeader, IsoBox};

#[derive(IsoBox)]
#[iso_box(box_type = b"free", crate_path = "crate")]
pub struct FreeBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}

#[derive(IsoBox)]
#[iso_box(box_type = b"skip", crate_path = "crate")]
pub struct SkipBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}
