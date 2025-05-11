#![allow(missing_docs)]

use std::io;

use byteorder::ReadBytesExt;

use super::ZeroCopyReader;
use crate::{BytesCow, StringCow};

pub trait Container {
    type Item;
    fn add(&mut self, item: Self::Item);
}

impl<T> Container for Vec<T> {
    type Item = T;

    fn add(&mut self, item: Self::Item) {
        self.push(item);
    }
}

impl<T> Container for Option<T> {
    type Item = T;

    fn add(&mut self, item: Self::Item) {
        *self = Some(item);
    }
}

pub trait Deserialize<'a>: Sized {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>;
}

pub trait DeserializeSeed<'a, S>: Sized {
    fn deserialize_seed<R>(reader: R, seed: S) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>;
}

impl<'a> Deserialize<'a> for f32 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_f32::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for f64 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_f64::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for i8 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i8()
    }
}

impl<'a> Deserialize<'a> for i16 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i16::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for i32 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i32::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for i64 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i64::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for i128 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i128::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for u8 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u8()
    }
}

impl<'a> Deserialize<'a> for char {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u8().map(|b| b as char)
    }
}

impl<'a> Deserialize<'a> for u16 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u16::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for u32 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u32::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for u64 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u64::<byteorder::BigEndian>()
    }
}

impl<'a> Deserialize<'a> for u128 {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u128::<byteorder::BigEndian>()
    }
}

impl<'a, T, const LEN: usize> Deserialize<'a> for [T; LEN]
where
    T: Deserialize<'a> + Default + Copy,
{
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        let mut buf = [T::default(); LEN];

        for t in &mut buf {
            *t = T::deserialize(&mut reader)?;
        }

        Ok(buf)
    }
}

impl<'a> Deserialize<'a> for BytesCow<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        reader.try_read_to_end()
    }
}

impl<'a> Deserialize<'a> for StringCow<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let bytes = reader.try_read_to_end()?;
        Self::try_from(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))
    }
}

/// A 24-bit signed integer in big-endian byte order.
pub struct I24Be(i32);

impl<'a> Deserialize<'a> for I24Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i24::<byteorder::BigEndian>().map(I24Be)
    }
}

impl From<I24Be> for i32 {
    fn from(value: I24Be) -> Self {
        value.0
    }
}

/// A 48-bit signed integer in big-endian byte order.
pub struct I48Be(i64);

impl<'a> Deserialize<'a> for I48Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i48::<byteorder::BigEndian>().map(I48Be)
    }
}

impl From<I48Be> for i64 {
    fn from(value: I48Be) -> Self {
        value.0
    }
}

/// A 24-bit unsigned integer in big-endian byte order.
pub struct U24Be(u32);

impl<'a> Deserialize<'a> for U24Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u24::<byteorder::BigEndian>().map(U24Be)
    }
}

impl From<U24Be> for u32 {
    fn from(value: U24Be) -> Self {
        value.0
    }
}

/// A 48-bit unsigned integer in big-endian byte order.
pub struct U48Be(u64);

impl<'a> Deserialize<'a> for U48Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u48::<byteorder::BigEndian>().map(U48Be)
    }
}

impl From<U48Be> for u64 {
    fn from(value: U48Be) -> Self {
        value.0
    }
}
