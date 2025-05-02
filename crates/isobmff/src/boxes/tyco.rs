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
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::zero_copy::{Deserialize, Slice};

    use crate::boxes::{Brand, Tyco};
    use crate::{BoxHeaderProperties, BoxSize};

    #[test]
    fn demux() {
        #[rustfmt::skip]
        let data = [
            0x00, 0x00, 0x00, 0x0C, // size
            b't', b'y', b'c', b'o', // type
            b'i', b's', b'o', b'6', // data
            0x01,
        ];

        let mdat = Tyco::deserialize(Slice::from(&data[..])).unwrap();
        assert_eq!(mdat.header.size, BoxSize::Short(12));
        assert!(mdat.header.box_type.is_four_cc(b"tyco"));
        assert_eq!(mdat.header.payload_size(), Some(4));
        assert_eq!(mdat.compatible_brands.len(), 1);
        assert_eq!(mdat.compatible_brands[0], Brand::Iso6);
    }
}
