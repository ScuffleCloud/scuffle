#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

pub mod boxes;
mod header;

pub use header::*;
pub use isobmff_derive::IsoBox;
use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::DeserializeSeed;

pub trait IsoBox {
    const TYPE: [u8; 4];
    type Header;
}

pub struct UnknownBox<'a> {
    pub header: BoxHeader,
    pub data: BytesCow<'a>,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for UnknownBox<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            header: seed,
            data: reader.try_read_to_end()?,
        })
    }
}
