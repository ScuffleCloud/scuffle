use super::{DataInformationBox, HandlerBox};
use crate::{FullBoxHeader, IsoBox, UnknownBox};

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"meta", crate_path = crate)]
pub struct MetaBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    #[iso_box(nested_box(collect))]
    pub dinf: Option<DataInformationBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}
