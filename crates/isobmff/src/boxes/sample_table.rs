use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox};

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stbl", crate_path = "crate")]
pub struct SampleTableBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub stsd: SampleDescriptionBox<'a>,
    #[iso_box(nested_box(collect))]
    pub stdp: Option<DegradationPriorityBox>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stsd", crate_path = "crate")]
pub struct SampleDescriptionBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"btrt", crate_path = "crate")]
pub struct BitRateBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stdp", crate_path = "crate")]
pub struct DegradationPriorityBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(repeated)]
    pub priority: Vec<u16>,
}
