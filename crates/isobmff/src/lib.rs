#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use std::io;

use scuffle_bytes_util::zero_copy::ZeroCopyReader;

pub mod boxes;
mod header;
pub mod read_field;

pub use header::*;
pub use isobmff_derive::IsoBox;

pub trait IsoBox<'a>: Sized {
    const TYPE: [u8; 4];

    fn demux<R: ZeroCopyReader<'a>>(header: BoxHeader, payload_reader: R) -> io::Result<Self>;
}
