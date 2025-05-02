use crate::{BoxHeader, IsoBox};

#[derive(IsoBox)]
#[iso_box(box_type = b"mdat", crate_path = "crate")]
pub struct Mdat {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(repeated)]
    pub data: Vec<u8>,
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::zero_copy::{Deserialize, Slice};

    use super::Mdat;
    use crate::{BoxHeaderProperties, BoxSize};

    #[test]
    fn demux() {
        #[rustfmt::skip]
        let data = [
            0x00, 0x00, 0x00, 0x0C, // size
            b'm', b'd', b'a', b't', // type
            0x42, 0x00, 0x42, 0x00, // data
            0x01,
        ];

        let mdat = Mdat::deserialize(Slice::from(&data[..])).unwrap();
        assert_eq!(mdat.header.size, BoxSize::Short(12));
        assert!(mdat.header.box_type.is_four_cc(b"mdat"));
        assert_eq!(mdat.header.payload_size(), Some(4));
        assert_eq!(mdat.data.len(), 4);
        assert_eq!(mdat.data[0], 0x42);
        assert_eq!(mdat.data[1], 0x00);
        assert_eq!(mdat.data[2], 0x42);
        assert_eq!(mdat.data[3], 0x00);
    }
}
