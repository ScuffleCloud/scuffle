use std::io;

/// A reader that reads individual bits from a stream
#[derive(Debug)]
#[must_use]
pub struct BitReader<T> {
    data: T,
    bit_pos: u8,
    current_byte: u8,
}

impl<T> BitReader<T> {
    /// Create a new BitReader from a reader
    pub const fn new(data: T) -> Self {
        Self {
            data,
            bit_pos: 0,
            current_byte: 0,
        }
    }
}

impl<T: io::Read> BitReader<T> {
    /// Reads a single bit
    pub fn read_bit(&mut self) -> io::Result<bool> {
        if self.is_aligned() {
            self.update_byte()?;
        }

        let bit = (self.current_byte >> (7 - self.bit_pos)) & 1;

        self.bit_pos = (self.bit_pos + 1) % 8;

        Ok(bit == 1)
    }

    fn update_byte(&mut self) -> io::Result<()> {
        let mut buf = [0];
        self.data.read_exact(&mut buf)?;
        self.current_byte = buf[0];
        Ok(())
    }

    /// Reads multiple bits
    pub fn read_bits(&mut self, count: u8) -> io::Result<u64> {
        let count = count.min(64);

        let mut bits = 0;
        for _ in 0..count {
            let bit = self.read_bit()?;
            bits <<= 1;
            bits |= if bit { 1 } else { 0 };
        }

        Ok(bits)
    }

    /// Aligns the reader to the next byte boundary
    #[inline(always)]
    pub fn align(&mut self) -> io::Result<()> {
        // This has the effect of making the next read_bit call read the next byte
        // and is equivalent to calling read_bits(8 - self.bit_pos)
        self.bit_pos = 0;
        Ok(())
    }
}

impl<T> BitReader<T> {
    /// Returns the underlying reader
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Returns a reference to the underlying reader
    #[inline(always)]
    #[must_use]
    pub const fn get_ref(&self) -> &T {
        &self.data
    }

    /// Returns the current bit position (0-7)
    #[inline(always)]
    #[must_use]
    pub const fn bit_pos(&self) -> u8 {
        self.bit_pos
    }

    /// Checks if the reader is aligned to the byte boundary
    #[inline(always)]
    #[must_use]
    pub const fn is_aligned(&self) -> bool {
        self.bit_pos == 0
    }
}

impl<T: io::Read> io::Read for BitReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we are aligned this will be essentially the same as just reading directly
        // from the underlying reader.
        if self.is_aligned() {
            return self.data.read(buf);
        }

        // However if we are not aligned we need to shift all the bits into the correct
        // position. Think of it like this
        //
        // 011|0110 0000000 11111111
        //    ^---- This is the next bit to read (0) i show it with a | to make it clear
        // the resulting read should be [01100000, 00001111]
        // Byte 1: first 4 bits are from the first byte and 4 bits from the second byte
        // Byte 2: first 4 bits are from the second byte and the first 4 bits from the
        // third byte

        for byte in buf.iter_mut() {
            *byte = 0;
            for _ in 0..8 {
                let bit = self.read_bit()?;
                *byte <<= 1;
                *byte |= bit as u8;
            }
        }

        Ok(buf.len())
    }
}

impl<B: AsRef<[u8]>> BitReader<std::io::Cursor<B>> {
    /// Creates a new BitReader from a slice
    pub const fn new_from_slice(data: B) -> Self {
        Self::new(std::io::Cursor::new(data))
    }
}

impl<W: io::Seek + io::Read> BitReader<W> {
    /// Returns the current stream position in bits
    pub fn bit_stream_position(&mut self) -> io::Result<u64> {
        let pos = self.data.stream_position()?;
        Ok(pos * 8 + if self.is_aligned() { 8 } else { self.bit_pos as u64 } - 8)
    }

    /// Seeks a number of bits forward or backward
    /// Returns the new stream position in bits
    pub fn seek_bits(&mut self, count: i64) -> io::Result<u64> {
        // We dont need to do any work here.
        if count == 0 {
            return self.bit_stream_position();
        }

        let count = self.bit_pos as i64 + count;

        // Otherwise we need to do some work to move the bit position to the desired
        // position

        // the number of bits we should move by
        let bit_move = count % 8;
        // the number of bytes we should move by
        let mut byte_move = count / 8;

        // if we are not aligned we need to move back 1 byte (since we have partially
        // read the current byte)
        if !self.is_aligned() {
            byte_move -= 1;
        }

        // if we are seeking back we need to move back 1 byte (since we are going to
        // move forward a byte)
        if bit_move < 0 {
            byte_move -= 1;
        }

        let mut pos = self.data.seek(io::SeekFrom::Current(byte_move))? * 8;

        // This works for both positive and negative bit_move
        // If bit_move is -3 then we want to move 3 bits (8 + (-3)) % 8 = 5
        // If bit_move is 3 then we want to move 3 bits (8 + 3) % 8 = 3
        // Modulo arithmetic is cool!
        self.bit_pos = ((8 + bit_move) % 8) as u8;

        // If we are not unaligned we need to update the byte because we have a partial
        // read, but the byte has not been read yet (its only read when the bit
        // position is 0 on the next call to read_bit)
        if !self.is_aligned() {
            self.update_byte()?;
            pos += self.bit_pos as u64;
        }

        Ok(pos)
    }
}

impl<T: io::Seek + io::Read> io::Seek for BitReader<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            // Otherwise if we are doing a relative seek we likely do care about the bit position
            // So we call the seek_bits function to handle seeking the offset in bits
            io::SeekFrom::Current(offset) if !self.is_aligned() => {
                // This returns the new stream position in bytes rounded up to the nearest byte
                Ok(self.seek_bits(offset * 8)?.div_ceil(8))
            }
            // Otherwise we are seeking to a position relative to the start or end of the stream, we dont care about the bit
            // position Or the bit position is already 0 so we can just seek to the new position
            _ => {
                self.bit_pos = 0;
                self.data.seek(pos)
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use io::{Read, Seek};

    use super::*;

    #[test]
    fn test_bit_reader() {
        let binary = 0b10101010110011001111000101010101u32;

        let mut reader = BitReader::new_from_slice(binary.to_be_bytes());
        for i in 0..32 {
            assert_eq!(
                reader.read_bit().unwrap(),
                (binary & (1 << (31 - i))) != 0,
                "bit {i} is not correct"
            );
        }

        assert!(reader.read_bit().is_err(), "there shouldnt be any bits left");
    }

    #[test]
    fn test_bit_reader_read_bits() {
        let binary = 0b10101010110011001111000101010101u32;
        let mut reader = BitReader::new_from_slice(binary.to_be_bytes());
        let cases = [
            (3, 0b101),
            (4, 0b0101),
            (3, 0b011),
            (3, 0b001),
            (3, 0b100),
            (3, 0b111),
            (5, 0b10001),
            (1, 0b0),
            (7, 0b1010101),
        ];

        for (i, (count, expected)) in cases.into_iter().enumerate() {
            assert_eq!(
                reader.read_bits(count).ok(),
                Some(expected),
                "reading {count} bits ({i}) are not correct"
            );
        }

        assert!(reader.read_bit().is_err(), "there shouldnt be any bits left");
    }

    #[test]
    fn test_bit_reader_align() {
        let mut reader = BitReader::new_from_slice([0b10000000, 0b10000000, 0b10000000, 0b10000000, 0b10000000, 0b10000000]);

        for i in 0..6 {
            let pos = reader.data.stream_position().unwrap();
            assert_eq!(pos, i, "stream pos");
            assert_eq!(reader.bit_pos(), 0, "bit pos");
            assert!(reader.read_bit().unwrap(), "bit {i} is not correct");
            reader.align().unwrap();
            let pos = reader.data.stream_position().unwrap();
            assert_eq!(pos, i + 1, "stream pos");
            assert_eq!(reader.bit_pos(), 0, "bit pos");
        }

        assert!(reader.read_bit().is_err(), "there shouldnt be any bits left");
    }

    #[test]
    fn test_bit_reader_io_read() {
        let binary = 0b10101010110011001111000101010101u32;
        let mut reader = BitReader::new_from_slice(binary.to_be_bytes());

        // Aligned read (calls the underlying read directly (very fast))
        let mut buf = [0; 1];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [0b10101010]);

        // Unaligned read
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        let mut buf = [0; 1];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [0b10011001]);
    }

    #[test]
    fn test_bit_reader_seek() {
        let binary = 0b10101010110011001111000101010101u32;
        let mut reader = BitReader::new_from_slice(binary.to_be_bytes());

        assert_eq!(reader.seek_bits(5).unwrap(), 5);
        assert_eq!(reader.data.stream_position().unwrap(), 1);
        assert_eq!(reader.bit_pos(), 5);
        assert_eq!(reader.read_bits(1).unwrap(), 0b0);
        assert_eq!(reader.bit_pos(), 6);

        assert_eq!(reader.seek_bits(0).unwrap(), 6);

        assert_eq!(reader.seek_bits(10).unwrap(), 16);
        assert_eq!(reader.data.stream_position().unwrap(), 2);
        assert_eq!(reader.bit_pos(), 0);
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        assert_eq!(reader.bit_pos(), 1);
        assert_eq!(reader.data.stream_position().unwrap(), 3);

        assert_eq!(reader.seek_bits(-8).unwrap(), 9);
        assert_eq!(reader.data.stream_position().unwrap(), 2);
        assert_eq!(reader.bit_pos(), 1);
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        assert_eq!(reader.bit_pos(), 2);
        assert_eq!(reader.data.stream_position().unwrap(), 2);

        assert_eq!(reader.seek_bits(-2).unwrap(), 8);
        assert_eq!(reader.data.stream_position().unwrap(), 1);
        assert_eq!(reader.bit_pos(), 0);
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        assert_eq!(reader.bit_pos(), 1);
        assert_eq!(reader.data.stream_position().unwrap(), 2);
    }

    #[test]
    fn test_bit_reader_io_seek() {
        let binary = 0b10101010110011001111000101010101u32;
        let mut reader = BitReader::new_from_slice(binary.to_be_bytes());
        assert_eq!(reader.seek(io::SeekFrom::Start(1)).unwrap(), 1);
        assert_eq!(reader.bit_pos(), 0);
        assert_eq!(reader.data.stream_position().unwrap(), 1);
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        assert_eq!(reader.bit_pos(), 1);
        assert_eq!(reader.data.stream_position().unwrap(), 2);

        assert_eq!(reader.seek(io::SeekFrom::Current(1)).unwrap(), 3);
        assert_eq!(reader.bit_pos(), 1);
        assert_eq!(reader.data.stream_position().unwrap(), 3);
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        assert_eq!(reader.bit_pos(), 2);
        assert_eq!(reader.data.stream_position().unwrap(), 3);

        assert_eq!(reader.seek(io::SeekFrom::Current(-1)).unwrap(), 2);
        assert_eq!(reader.bit_pos(), 2);
        assert_eq!(reader.data.stream_position().unwrap(), 2);
        assert_eq!(reader.read_bits(1).unwrap(), 0b0);
        assert_eq!(reader.bit_pos(), 3);
        assert_eq!(reader.data.stream_position().unwrap(), 2);

        assert_eq!(reader.seek(io::SeekFrom::End(-1)).unwrap(), 3);
        assert_eq!(reader.bit_pos(), 0);
        assert_eq!(reader.data.stream_position().unwrap(), 3);
        assert_eq!(reader.read_bits(1).unwrap(), 0b0);
        assert_eq!(reader.bit_pos(), 1);
        assert_eq!(reader.data.stream_position().unwrap(), 4);
    }
}
