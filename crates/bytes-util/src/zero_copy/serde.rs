//! Types for binary deserialization and serialization.

use std::io;
use std::ops::Deref;

use byteorder::ReadBytesExt;

use super::ZeroCopyReader;
use crate::{BytesCow, StringCow};

/// A trait that should be implemented by types that can contain other deserializable types.
pub trait Container {
    /// The type of items in the container.
    type Item;
    /// Adds an item to the container.
    fn add(&mut self, item: Self::Item);
}

impl<T> Container for Vec<T> {
    type Item = T;

    fn add(&mut self, item: Self::Item) {
        self.push(item);
    }
}

// TODO: This doesn't really make sense but it works
impl<T> Container for Option<T> {
    type Item = T;

    fn add(&mut self, item: Self::Item) {
        *self = Some(item);
    }
}

/// A trait for deserializing types from a zero-copy reader.
pub trait Deserialize<'a>: Sized {
    /// Deserialize a value from the given zero-copy reader.
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>;
}

/// A trait for deserializing types from a zero-copy reader with a seed.
pub trait DeserializeSeed<'a, S>: Sized {
    /// Deserialize a value from the given zero-copy reader using the provided seed.
    fn deserialize_seed<R>(reader: R, seed: S) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>;
}

/// A trait for serializing types to a writer.
pub trait Serialize {
    /// Serialize the value into the given writer.
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write;
}

/// A trait for serializing types to a writer with a seed.
pub trait SerializeSeed<S> {
    /// Serialize the value into the given writer using the provided seed.
    fn serialize_seed<W>(&self, writer: W, seed: S) -> io::Result<()>
    where
        W: std::io::Write;
}

macro_rules! impl_serialize {
    ($($t:ty),+) => {
        $(
            impl Serialize for $t {
                fn serialize<W>(&self, mut writer: W) -> io::Result<()>
                where
                    W: std::io::Write,
                {
                    writer.write_all(&self.to_be_bytes())
                }
            }
        )+
    };
}

impl_serialize!(f32, f64, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

impl Serialize for char {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        (*self as u8).serialize(writer)
    }
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

impl<T, const LEN: usize> Serialize for [T; LEN]
where
    T: Serialize,
{
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        for t in self {
            t.serialize(&mut writer)?;
        }

        Ok(())
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

impl Serialize for BytesCow<'_> {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(self.as_bytes())
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

impl Serialize for StringCow<'_> {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(self.as_str().as_bytes())
    }
}

/// A 24-bit signed integer in big-endian byte order.
#[derive(Debug, Clone, Copy)]
pub struct I24Be(pub i32);

impl<'a> Deserialize<'a> for I24Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i24::<byteorder::BigEndian>().map(I24Be)
    }
}

impl Serialize for I24Be {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.0.to_be_bytes()[1..])
    }
}

impl From<I24Be> for i32 {
    fn from(value: I24Be) -> Self {
        value.0
    }
}

impl Deref for I24Be {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A 48-bit signed integer in big-endian byte order.
#[derive(Debug, Clone, Copy)]
pub struct I48Be(pub i64);

impl<'a> Deserialize<'a> for I48Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i48::<byteorder::BigEndian>().map(I48Be)
    }
}

impl Serialize for I48Be {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.0.to_be_bytes()[2..])
    }
}

impl From<I48Be> for i64 {
    fn from(value: I48Be) -> Self {
        value.0
    }
}

impl Deref for I48Be {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A 24-bit unsigned integer in big-endian byte order.
#[derive(Debug, Clone, Copy)]
pub struct U24Be(pub u32);

impl<'a> Deserialize<'a> for U24Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u24::<byteorder::BigEndian>().map(U24Be)
    }
}

impl Serialize for U24Be {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.0.to_be_bytes()[1..])
    }
}

impl From<U24Be> for u32 {
    fn from(value: U24Be) -> Self {
        value.0
    }
}

impl Deref for U24Be {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A 48-bit unsigned integer in big-endian byte order.
#[derive(Debug, Clone, Copy)]
pub struct U48Be(pub u64);

impl<'a> Deserialize<'a> for U48Be {
    fn deserialize<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u48::<byteorder::BigEndian>().map(U48Be)
    }
}

impl Serialize for U48Be {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.0.to_be_bytes()[2..])
    }
}

impl From<U48Be> for u64 {
    fn from(value: U48Be) -> Self {
        value.0
    }
}

impl Deref for U48Be {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
