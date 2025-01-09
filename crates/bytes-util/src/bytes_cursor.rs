use std::io;

use bytes::Bytes;

/// A helper trait to implement zero copy reads on a `Cursor<Bytes>` type.
pub trait BytesCursor {
    /// Returns the remaining bytes in the cursor.
    fn remaining(&self) -> usize;

    /// Extracts the remaining bytes from the cursor returning.
    ///
    /// This does not do a copy of the bytes, and is O(1) time.
    ///
    /// This is the same as `BytesCursor::extract_bytes(self.remaining())`.
    fn extract_remaining(&mut self) -> Bytes;

    /// Extracts a bytes from the cursor.
    ///
    /// This does not do a copy of the bytes, and is O(1) time.
    /// Returns an error if the size is greater than the remaining bytes.
    fn extract_bytes(&mut self, size: usize) -> io::Result<Bytes>;
}

impl BytesCursor for io::Cursor<Bytes> {
    fn remaining(&self) -> usize {
        // We have to use a saturating sub here because the position can be
        // greater than the length of the bytes.
        self.get_ref().len().saturating_sub(self.position() as usize)
    }

    fn extract_remaining(&mut self) -> Bytes {
        // We don't really care if we fail here since the desired behavior is
        // to return all bytes remaining in the cursor. If we fail its because
        // there are not enough bytes left in the cursor to read.
        self.extract_bytes(self.remaining()).unwrap_or_default()
    }

    fn extract_bytes(&mut self, size: usize) -> io::Result<Bytes> {
        // If the size is zero we can just return an empty bytes slice.
        if size == 0 {
            return Ok(Bytes::new());
        }

        // If the size is greater than the remaining bytes we can just return an
        // error.
        if size > self.remaining() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "not enough bytes"));
        }

        let position = self.position() as usize;

        // We slice bytes here which is a O(1) operation as it only modifies a few
        // reference counters and does not copy the memory.
        let slice = self.get_ref().slice(position..position + size);

        // We advance the cursor because we have now "read" the bytes.
        self.set_position((position + size) as u64);

        Ok(slice)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_cursor_extract_remaining() {
        let mut cursor = io::Cursor::new(Bytes::from_static(&[1, 2, 3, 4, 5]));
        let remaining = cursor.extract_remaining();
        assert_eq!(remaining, Bytes::from_static(&[1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_bytes_cursor_extract_bytes() {
        let mut cursor = io::Cursor::new(Bytes::from_static(&[1, 2, 3, 4, 5]));
        let bytes = cursor.extract_bytes(3).unwrap();
        assert_eq!(bytes, Bytes::from_static(&[1, 2, 3]));
        assert_eq!(cursor.remaining(), 2);

        let bytes = cursor.extract_bytes(2).unwrap();
        assert_eq!(bytes, Bytes::from_static(&[4, 5]));
        assert_eq!(cursor.remaining(), 0);

        let bytes = cursor.extract_bytes(1).unwrap_err();
        assert_eq!(bytes.kind(), io::ErrorKind::UnexpectedEof);

        let bytes = cursor.extract_bytes(0).unwrap();
        assert_eq!(bytes, Bytes::from_static(&[]));
        assert_eq!(cursor.remaining(), 0);

        let bytes = cursor.extract_remaining();
        assert_eq!(bytes, Bytes::from_static(&[]));
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn seek_out_of_bounds() {
        let mut cursor = io::Cursor::new(Bytes::from_static(&[1, 2, 3, 4, 5]));
        cursor.set_position(10);
        assert_eq!(cursor.remaining(), 0);

        let bytes = cursor.extract_remaining();
        assert_eq!(bytes, Bytes::from_static(&[]));

        let bytes = cursor.extract_bytes(1);
        assert_eq!(bytes.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);

        let bytes = cursor.extract_bytes(0);
        assert_eq!(bytes.unwrap(), Bytes::from_static(&[]));
    }
}