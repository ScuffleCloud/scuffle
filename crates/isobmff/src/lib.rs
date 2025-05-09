// compressed boxes (8.19) are not supported

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use std::fmt::Debug;

use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::DeserializeSeed;

pub mod boxes;
mod file;
mod header;
mod string_deserializer;

pub use file::*;
pub use header::*;
pub use isobmff_derive::IsoBox;
pub use string_deserializer::*;

pub trait IsoBox {
    const TYPE: [u8; 4];
    type Header;
}

pub struct UnknownBox<'a> {
    pub header: BoxHeader,
    pub data: BytesCow<'a>,
}

impl Debug for UnknownBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnknownBox")
            .field("header", &self.header)
            .field("data.len", &self.data.len())
            .finish()
    }
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
