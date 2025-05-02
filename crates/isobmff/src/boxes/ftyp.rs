use nutype_enum::nutype_enum;

use crate::{BoxHeader, IsoBox};

/// File-type box
///
/// ISO/IEC 14496-12 - 4.3
#[derive(IsoBox)]
#[iso_box(box_type = b"ftyp", crate_path = "crate")]
pub struct Ftyp {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}

nutype_enum! {
    pub enum Brand([u8; 4]) {
        IsoM = *b"isom",
        Avc1 = *b"avc1",
        Iso2 = *b"iso2",
        Mp71 = *b"mp71",
        Iso3 = *b"iso3",
        Iso4 = *b"iso4",
        Iso5 = *b"iso5",
        Iso6 = *b"iso6",
        Iso7 = *b"iso7",
        Iso8 = *b"iso8",
        Iso9 = *b"iso9",
        IsoA = *b"isoa",
        IsoB = *b"isob",
        Relo = *b"relo",
        IsoC = *b"isoc",
        Comp = *b"comp",
        Unif = *b"unif",
    }
}
