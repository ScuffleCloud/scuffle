use super::Tyco;
use crate::{BoxHeader, IsoBox, UnknownBox};

/// Extended type box
///
/// ISO/IEC 14496-12 - 4.4
#[derive(IsoBox)]
#[iso_box(box_type = b"etyp", crate_path = "crate")]
pub struct Etyp<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated = "box")]
    pub compatible_combinations: Vec<Tyco>,
    #[iso_box(repeated = "unknown_box")]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::zero_copy::{Deserialize, Slice};

    use crate::boxes::{Brand, Etyp};
    use crate::{BoxHeaderProperties, BoxSize};

    #[test]
    fn demux() {
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

        let mdat = Etyp::deserialize(Slice::from(&data[..])).unwrap();
        assert_eq!(mdat.header.size, BoxSize::Short(44));
        assert!(mdat.header.box_type.is_four_cc(b"etyp"));
        assert_eq!(mdat.header.payload_size(), Some(44 - 8));

        assert_eq!(mdat.compatible_combinations.len(), 2);

        assert_eq!(mdat.compatible_combinations[0].header.size, BoxSize::Short(12));
        assert!(mdat.compatible_combinations[0].header.box_type.is_four_cc(b"tyco"));
        assert_eq!(mdat.compatible_combinations[0].header.payload_size(), Some(4));
        assert_eq!(mdat.compatible_combinations[0].compatible_brands.len(), 1);
        assert_eq!(mdat.compatible_combinations[0].compatible_brands[0], Brand::IsoM);

        assert_eq!(mdat.compatible_combinations[1].header.size, BoxSize::Short(12));
        assert!(mdat.compatible_combinations[1].header.box_type.is_four_cc(b"tyco"));
        assert_eq!(mdat.compatible_combinations[1].header.payload_size(), Some(4));
        assert_eq!(mdat.compatible_combinations[1].compatible_brands.len(), 1);
        assert_eq!(mdat.compatible_combinations[1].compatible_brands[0], Brand::Iso6);

        assert_eq!(mdat.unknown_boxes.len(), 1);
        assert_eq!(mdat.unknown_boxes[0].header.size, BoxSize::Short(12));
        assert!(mdat.unknown_boxes[0].header.box_type.is_four_cc(b"unkn"));
        assert_eq!(mdat.unknown_boxes[0].header.payload_size(), Some(4));
        assert_eq!(mdat.unknown_boxes[0].data.len(), 4);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[0], 0x42);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[1], 0x00);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[2], 0x42);
        assert_eq!(mdat.unknown_boxes[0].data.as_bytes()[3], 0x00);
    }
}
