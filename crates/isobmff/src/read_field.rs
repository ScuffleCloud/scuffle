use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::zero_copy::ZeroCopyReader;

use crate::{BoxHeader, IsoBox};

pub trait ReadField<'a>: Sized {
    fn read_field<R: ZeroCopyReader<'a>>(reader: R) -> io::Result<Self>;
    fn size_hint() -> Option<usize> {
        Some(std::mem::size_of::<Self>())
    }
}

impl<'a> ReadField<'a> for f32 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_f32::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for f64 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_f64::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for i8 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i8()
    }
}

impl<'a> ReadField<'a> for i16 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i16::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for i32 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i32::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for i64 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i64::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for i128 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i128::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for u8 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u8()
    }
}

impl<'a> ReadField<'a> for u16 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u16::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for u32 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u32::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for u64 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u64::<byteorder::BigEndian>()
    }
}

impl<'a> ReadField<'a> for u128 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u128::<byteorder::BigEndian>()
    }
}

impl<'a, const LEN: usize> ReadField<'a> for [u8; LEN] {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        Ok(reader.try_read(LEN)?.as_bytes().try_into().unwrap())
    }
}

#[allow(non_camel_case_types)]
pub struct i24(i32);

impl<'a> ReadField<'a> for i24 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i24::<byteorder::BigEndian>().map(i24)
    }

    fn size_hint() -> Option<usize> {
        Some(3)
    }
}

impl From<i24> for i32 {
    fn from(value: i24) -> Self {
        value.0
    }
}

#[allow(non_camel_case_types)]
pub struct i48(i64);

impl<'a> ReadField<'a> for i48 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_i48::<byteorder::BigEndian>().map(i48)
    }

    fn size_hint() -> Option<usize> {
        Some(6)
    }
}

impl From<i48> for i64 {
    fn from(value: i48) -> Self {
        value.0
    }
}

#[allow(non_camel_case_types)]
pub struct u24(u32);

impl<'a> ReadField<'a> for u24 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u24::<byteorder::BigEndian>().map(u24)
    }

    fn size_hint() -> Option<usize> {
        Some(3)
    }
}

impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        value.0
    }
}

#[allow(non_camel_case_types)]
pub struct u48(u64);

impl<'a> ReadField<'a> for u48 {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        reader.as_std().read_u48::<byteorder::BigEndian>().map(u48)
    }

    fn size_hint() -> Option<usize> {
        Some(6)
    }
}

impl From<u48> for u64 {
    fn from(value: u48) -> Self {
        value.0
    }
}

impl<'a, B: IsoBox<'a>> ReadField<'a> for B {
    fn read_field<R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        let box_header = BoxHeader::demux(&mut reader)?;

        if let Some(size) = box_header.payload_size() {
            Self::demux(box_header, reader.take(size))
        } else {
            Self::demux(box_header, reader)
        }
    }

    fn size_hint() -> Option<usize> {
        None
    }
}

pub trait ReadRemaining<'a>: Sized {
    fn read_remaining<R: ZeroCopyReader<'a>>(reader: R, size_hint: Option<usize>) -> io::Result<Self>;
}

// Could be optimized with specialization for Vec<u8>
impl<'a, T: ReadField<'a>> ReadRemaining<'a> for Vec<T> {
    fn read_remaining<R: ZeroCopyReader<'a>>(mut reader: R, size_hint: Option<usize>) -> io::Result<Self> {
        let mut remaining = if let (Some(total_size), Some(field_size)) = (size_hint, T::size_hint()) {
            Vec::with_capacity(total_size / field_size)
        } else {
            Vec::new()
        };

        loop {
            match T::read_field(&mut reader) {
                Ok(value) => remaining.push(value),
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }

        remaining.shrink_to_fit();

        Ok(remaining)
    }
}
