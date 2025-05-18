//! Zero-copy reader types.

use std::io::{self, Read};

use crate::BytesCow;

/// A trait for zero-copy readers.
pub trait ZeroCopyReader<'a> {
    /// Attempts to read a specified number of bytes from the reader without copying.
    ///
    /// This function does not guarantee that no copying will occur.
    /// Some implementations can't avoid copying.
    fn try_read(&mut self, size: usize) -> Result<BytesCow<'a>, io::Error>;

    /// Returns a standard [`io::Read`] interface for the reader.
    fn as_std(&mut self) -> impl io::Read;

    /// Limits the number of bytes that can be read from the reader.
    fn take(self, limit: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, limit)
    }

    /// Reads all remaining bytes from the reader.
    fn try_read_to_end(&mut self) -> Result<BytesCow<'a>, io::Error> {
        let mut buf = Vec::new();
        self.as_std().read_to_end(&mut buf)?;
        Ok(BytesCow::from_vec(buf))
    }
}

impl<'a, T: ZeroCopyReader<'a>> ZeroCopyReader<'a> for &mut T {
    fn try_read(&mut self, size: usize) -> Result<BytesCow<'a>, io::Error> {
        (*self).try_read(size)
    }

    fn as_std(&mut self) -> impl io::Read {
        (*self).as_std()
    }
}

/// A zero-copy reader that wraps a [`bytes::Buf`].
pub struct BytesBuf<B>(B);

impl<B: bytes::Buf> From<B> for BytesBuf<B> {
    fn from(buf: B) -> Self {
        Self(buf)
    }
}

impl<'a, B: bytes::Buf> ZeroCopyReader<'a> for BytesBuf<B> {
    fn try_read(&mut self, size: usize) -> Result<BytesCow<'a>, io::Error> {
        if self.0.remaining() < size {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data"));
        }

        Ok(BytesCow::from_bytes(self.0.copy_to_bytes(size)))
    }

    fn as_std(&mut self) -> impl io::Read {
        bytes::Buf::reader(&mut self.0)
    }
}

/// A zero-copy reader that wraps a [`std::io::Read`].
///
/// This implementation is not zero-copy and will always copy the data into a new buffer.
/// It is not possible to implement zero-copy reading for [`std::io::Read`]
/// because it does not provide a way to access the underlying buffer directly.
pub struct IoRead<R>(R);

impl<R: io::Read> From<R> for IoRead<R> {
    fn from(reader: R) -> Self {
        Self(reader)
    }
}

impl<'a, R: io::Read> ZeroCopyReader<'a> for IoRead<R> {
    fn try_read(&mut self, size: usize) -> Result<BytesCow<'a>, io::Error> {
        let mut buf = vec![0; size];
        self.0.read_exact(&mut buf)?;
        Ok(BytesCow::from_vec(buf))
    }

    fn as_std(&mut self) -> impl io::Read {
        &mut self.0
    }
}

/// A zero-copy reader that wraps a byte slice (`&[u8]`).
pub struct Slice<'a>(io::Cursor<&'a [u8]>);

impl<'a> From<&'a [u8]> for Slice<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self(io::Cursor::new(slice))
    }
}

impl<'a> ZeroCopyReader<'a> for Slice<'a> {
    fn try_read(&mut self, size: usize) -> Result<BytesCow<'a>, io::Error> {
        let start = self.0.position() as usize;
        let end = start + size;

        if end > self.0.get_ref().len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data"));
        }

        let slice = &self.0.get_ref()[start..end];
        self.0.set_position(end as u64);
        Ok(BytesCow::from_slice(slice))
    }

    fn as_std(&mut self) -> impl io::Read {
        &mut self.0
    }
}

/// Zero-copy reader that limits the number of bytes read.
///
/// This is equivalent to [`std::io::Take`], but it implements [`ZeroCopyReader`].
pub struct Take<R> {
    inner: R,
    limit: usize,
}

struct TakeAsStd<'a, R> {
    inner: R,
    limit: &'a mut usize,
}

impl<R: io::Read> io::Read for TakeAsStd<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if *self.limit == 0 {
            return Ok(0);
        }

        let size = std::cmp::min(buf.len(), *self.limit);
        let n = self.inner.read(&mut buf[..size])?;
        *self.limit -= n;
        Ok(n)
    }
}

impl<R> Take<R> {
    /// Creates a new [`Take`] reader that limits the number of bytes read.
    pub fn new(inner: R, limit: usize) -> Self {
        Self { inner, limit }
    }

    /// Returns the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<'a, R> ZeroCopyReader<'a> for Take<R>
where
    R: ZeroCopyReader<'a>,
{
    fn as_std(&mut self) -> impl io::Read {
        TakeAsStd {
            inner: self.inner.as_std(),
            limit: &mut self.limit,
        }
    }

    fn try_read(&mut self, size: usize) -> Result<BytesCow<'a>, io::Error> {
        let size = std::cmp::min(size, self.limit);
        let result = self.inner.try_read(size)?;
        self.limit -= result.len();
        Ok(result)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::{Slice, ZeroCopyReader};

    #[test]
    fn take_and_read_to_end() {
        let data = b"Hello, world!";
        let mut reader = Slice::from(&data[..]).take(5);
        assert_eq!(reader.try_read_to_end().unwrap(), b"Hello");
    }
}
