// Section 9 and 10 are not implemented
// TODO: Fixed-point numbers
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use std::fmt::Debug;

use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

pub mod boxes;
mod file;
mod header;
mod string_deserializer;

pub use file::*;
pub use header::*;
pub use isobmff_derive::IsoBox;
pub use string_deserializer::*;

#[doc(hidden)]
pub mod reexports {
    pub use scuffle_bytes_util;
}

pub trait IsoBox {
    const TYPE: BoxType;
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

impl<'a> Deserialize<'a> for UnknownBox<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        Self::deserialize_seed(&mut reader, header)
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

impl<'a> UnknownBox<'a> {
    pub fn deserialize_as<T, S>(self) -> std::io::Result<T>
    where
        T: DeserializeSeed<'a, S>,
        S: DeserializeSeed<'a, BoxHeader>,
    {
        let mut reader = scuffle_bytes_util::zero_copy::BytesBuf::from(self.data.into_bytes());
        let seed = S::deserialize_seed(&mut reader, self.header)?;
        T::deserialize_seed(&mut reader, seed)
    }

    pub fn deserialize_as_box<B>(self) -> std::io::Result<B>
    where
        B: IsoBox + DeserializeSeed<'a, B::Header>,
        B::Header: DeserializeSeed<'a, BoxHeader>,
    {
        if self.header.box_type != B::TYPE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Box type mismatch: expected {:?}, found {:?}", B::TYPE, self.header.box_type),
            ));
        }

        let mut reader = scuffle_bytes_util::zero_copy::BytesBuf::from(self.data.into_bytes());
        let seed = B::Header::deserialize_seed(&mut reader, self.header)?;
        B::deserialize_seed(&mut reader, seed)
    }
}
