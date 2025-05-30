//! File organization boxes defined in ISO/IEC 14496-12 - 4

use nutype_enum::nutype_enum;

use crate::{IsoBox, IsoSized, UnknownBox};

nutype_enum! {
    /// A four character code, registered with ISO, that identifies a precise specification.
    ///
    /// See: https://mp4ra.org/registered-types/brands
    pub enum Brand([u8; 4]) {
        Mp41 = *b"mp41",
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

impl IsoSized for Brand {
    fn size(&self) -> usize {
        4
    }
}

/// File-type box
///
/// ISO/IEC 14496-12 - 4.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ftyp", crate_path = crate)]
pub struct FileTypeBox {
    /// The "best use" brand of the file which will provide the greatest compatibility.
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    /// Minor version of the major brand.
    pub minor_version: u32,
    /// A list of compatible brands.
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}

/// Type combination box
///
/// ISO/IEC 14496-12 - 4.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tyco", crate_path = crate)]
pub struct TypeCombinationBox {
    /// A list of compatible brands.
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}

/// Extended type box
///
/// ISO/IEC 14496-12 - 4.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"etyp", crate_path = crate)]
pub struct ExtendedTypeBox<'a> {
    /// A list of [`TypeCombinationBox`]es.
    #[iso_box(nested_box(collect))]
    pub compatible_combinations: Vec<TypeCombinationBox>,
    /// A list of unknown boxes that were encountered while parsing the box.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::zero_copy::{Deserialize, Slice};

    use crate::boxes::{Brand, ExtendedTypeBox, TypeCombinationBox};

    #[test]
    fn demux_tyco() {
        #[rustfmt::skip]
        let data = [
            0x00, 0x00, 0x00, 0x0C, // size
            b't', b'y', b'c', b'o', // type
            b'i', b's', b'o', b'6', // data
            0x01,
        ];

        let mdat = TypeCombinationBox::deserialize(Slice::from(&data[..])).unwrap();
        assert_eq!(mdat.compatible_brands.len(), 1);
        assert_eq!(mdat.compatible_brands[0], Brand::Iso6);
    }

    #[test]
    fn demux_etyp() {
        #[rustfmt::skip]
        let data = [
            0x00, 0x00, 0x00, 44, // size
            b'e', b't', b'y', b'p', // type

            0x00, 0x00, 0x00, 12, // tyco size
            b't', b'y', b'c', b'o', // tyco type
            b'i', b's', b'o', b'm', // data

            0x00, 0x00, 0x00, 12, // tyco size
            b't', b'y', b'c', b'o', // tyco type
            b'i', b's', b'o', b'6', // data

            0x00, 0x00, 0x00, 12, // unknown size
            b'u', b'n', b'k', b'n', // unknown type
            0x42, 0x00, 0x42, 0x00, // data
        ];

        let mdat = ExtendedTypeBox::deserialize(Slice::from(&data[..])).unwrap();

        assert_eq!(mdat.compatible_combinations.len(), 2);

        assert_eq!(mdat.compatible_combinations[0].compatible_brands.len(), 1);
        assert_eq!(mdat.compatible_combinations[0].compatible_brands[0], Brand::IsoM);

        assert_eq!(mdat.compatible_combinations[1].compatible_brands.len(), 1);
        assert_eq!(mdat.compatible_combinations[1].compatible_brands[0], Brand::Iso6);

        assert_eq!(mdat.unknown_boxes.len(), 1);
        assert_eq!(mdat.unknown_boxes[0].data.len(), 4);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[0], 0x42);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[1], 0x00);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[2], 0x42);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[3], 0x00);
    }
}
