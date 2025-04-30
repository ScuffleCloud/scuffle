use crate::{BoxHeader, IsoBox};

#[derive(IsoBox)]
#[iso_box(box_type = b"mdat", crate_path = "crate")]
pub struct Mdat {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(remaining)]
    pub data: Vec<u8>,
}

// expanded
// impl<'a> IsoBox<'a> for Mdat {
//     const TYPE: [u8; 4] = *b"mdat";
//     fn demux<R: ::scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>>(
//         header: crate::BoxHeader,
//         mut payload_reader: R,
//     ) -> ::std::io::Result<Self> {
//         let data = {
//             let mut remaining = Vec::new();
//             loop {
//                 let value = {
//                     let data = match ::byteorder::ReadBytesExt::read_u8(
//                         &mut ::scuffle_bytes_util::zero_copy::ZeroCopyReader::as_std(
//                             &mut payload_reader,
//                         ),
//                     ) {
//                         Ok(v) => v,
//                         Err(e) if e.kind() == ::std::io::ErrorKind::UnexpectedEof => {
//                             break;
//                         }
//                         Err(e) => return Err(e),
//                     };
//                     data
//                 };
//                 remaining.push(value);
//             }
//             remaining
//         };
//         Ok(Self {
//             header: header,
//             data,
//         })
//     }
// }
