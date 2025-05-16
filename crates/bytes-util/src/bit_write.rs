use std::io;

/// A writer that allows you to write bits to a stream
#[derive(Debug)]
#[must_use]
pub struct BitWriter<W> {
    bit_pos: u8,
    current_byte: u8,
    writer: W,
}

impl<W: Default> Default for BitWriter<W> {
    fn default() -> Self {
        Self::new(W::default())
    }
}

impl<W: io::Write> BitWriter<W> {
    /// Writes a single bit to the stream
    pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        if bit {
            self.current_byte |= 1 << (7 - self.bit_pos);
        } else {
            self.current_byte &= !(1 << (7 - self.bit_pos));
        }

        self.bit_pos += 1;

        if self.bit_pos == 8 {
            self.writer.write_all(&[self.current_byte])?;
            self.current_byte = 0;
            self.bit_pos = 0;
        }

        Ok(())
    }

    /// Writes a number of bits to the stream (the most significant bit is
    /// written first).
    ///
    /// If `count` is less than 64 the lower bits are written.
    pub fn write_bits(&mut self, bits: u64, count: u8) -> io::Result<()> {
        let count = count.min(64);

        if count != 64 && bits > (1 << count as u64) - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bits too large to write"));
        }

        for i in 0..count {
            let bit = (bits >> (count - i - 1)) & 1 == 1;
            self.write_bit(bit)?;
        }

        Ok(())
    }

    /// Flushes the buffer and returns the underlying writer
    /// This will also align the writer to the byte boundary
    pub fn finish(mut self) -> io::Result<W> {
        self.align()?;
        Ok(self.writer)
    }

    /// Aligns the writer to the byte boundary
    pub fn align(&mut self) -> io::Result<()> {
        if !self.is_aligned() {
            self.write_bits(0, 8 - self.bit_pos())?;
        }

        Ok(())
    }
}

impl<W> BitWriter<W> {
    /// Creates a new BitWriter from a writer
    pub const fn new(writer: W) -> Self {
        Self {
            bit_pos: 0,
            current_byte: 0,
            writer,
        }
    }

    /// Returns the current bit position (0-7)
    #[inline(always)]
    #[must_use]
    pub const fn bit_pos(&self) -> u8 {
        self.bit_pos % 8
    }

    /// Checks if the writer is aligned to the byte boundary
    #[inline(always)]
    #[must_use]
    pub const fn is_aligned(&self) -> bool {
        self.bit_pos % 8 == 0
    }

    /// Returns a reference to the underlying writer
    #[inline(always)]
    #[must_use]
    pub const fn get_ref(&self) -> &W {
        &self.writer
    }
}

impl<W: io::Write> io::Write for BitWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.is_aligned() {
            return self.writer.write(buf);
        }

        for byte in buf {
            self.write_bits(*byte as u64, 8)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use io::Write;

    use super::*;

    #[test]
    fn test_bit_writer() {
        let mut bit_writer = BitWriter::<Vec<u8>>::default();

        bit_writer.write_bits(0b11111111, 8).unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());

        bit_writer.write_bits(0b0000, 4).unwrap();
        assert_eq!(bit_writer.bit_pos(), 4);
        assert!(!bit_writer.is_aligned());
        bit_writer.align().unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());

        bit_writer.write_bits(0b1010, 4).unwrap();
        assert_eq!(bit_writer.bit_pos(), 4);
        assert!(!bit_writer.is_aligned());

        bit_writer.write_bits(0b101010101010, 12).unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());

        bit_writer.write_bit(true).unwrap();
        assert_eq!(bit_writer.bit_pos(), 1);
        assert!(!bit_writer.is_aligned());

        let err = bit_writer.write_bits(0b10000, 4).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "bits too large to write");

        assert_eq!(
            bit_writer.finish().unwrap(),
            vec![0b11111111, 0b00000000, 0b10101010, 0b10101010, 0b10000000]
        );
    }

    #[test]
    fn test_flush_buffer() {
        let mut bit_writer = BitWriter::<Vec<u8>>::default();

        bit_writer.write_bits(0b11111111, 8).unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());
        assert_eq!(bit_writer.get_ref(), &[0b11111111], "underlying writer should have one byte");

        bit_writer.write_bits(0b0000, 4).unwrap();
        assert_eq!(bit_writer.bit_pos(), 4);
        assert!(!bit_writer.is_aligned());
        assert_eq!(bit_writer.get_ref(), &[0b11111111], "underlying writer should have one bytes");

        bit_writer.write_bits(0b1010, 4).unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());
        assert_eq!(
            bit_writer.get_ref(),
            &[0b11111111, 0b00001010],
            "underlying writer should have two bytes"
        );
    }

    #[test]
    fn test_io_write() {
        let mut inner = Vec::new();
        let mut bit_writer = BitWriter::new(&mut inner);

        bit_writer.write_bits(0b11111111, 8).unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());
        // We should have buffered the write
        assert_eq!(bit_writer.get_ref().as_slice(), &[255]);

        bit_writer.write_all(&[1, 2, 3]).unwrap();
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());
        // since we did an io::Write on an aligned bit_writer
        // we should have written directly to the underlying
        // writer
        assert_eq!(bit_writer.get_ref().as_slice(), &[255, 1, 2, 3]);

        bit_writer.write_bit(true).unwrap();

        bit_writer.write_bits(0b1010, 4).unwrap();

        bit_writer
            .write_all(&[0b11111111, 0b00000000, 0b11111111, 0b00000000])
            .unwrap();

        // Since the writer was not aligned we should have buffered the writes
        assert_eq!(
            bit_writer.get_ref().as_slice(),
            &[255, 1, 2, 3, 0b11010111, 0b11111000, 0b00000111, 0b11111000]
        );

        bit_writer.finish().unwrap();

        assert_eq!(
            inner,
            vec![255, 1, 2, 3, 0b11010111, 0b11111000, 0b00000111, 0b11111000, 0b00000000]
        );
    }

    #[test]
    fn test_flush() {
        let mut inner = Vec::new();
        let mut bit_writer = BitWriter::new(&mut inner);

        bit_writer.write_bits(0b10100000, 8).unwrap();

        bit_writer.flush().unwrap();

        assert_eq!(bit_writer.get_ref().as_slice(), &[0b10100000]);
        assert_eq!(bit_writer.bit_pos(), 0);
        assert!(bit_writer.is_aligned());
    }
}
