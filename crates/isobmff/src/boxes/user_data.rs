use crate::{BoxHeader, IsoBox, UnknownBox};

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"udta", crate_path = "crate")]
pub struct UserDataBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
