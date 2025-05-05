use std::io;

use scuffle_bytes_util::BitReader;

/// Read a little-endian variable-length integer.
/// AV1-Spec-2 - 4.10.5
pub(crate) fn read_leb128<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<u64> {
    let mut result = 0;
    for i in 0..8 {
        let byte = reader.read_bits(8)?;
        result |= (byte & 0x7f) << (i * 7);
        if byte & 0x80 == 0 {
            break;
        }
    }
    Ok(result)
}

/// Read a variable-length unsigned integer.
/// AV1-Spec-2 - 4.10.3
pub(crate) fn read_uvlc<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<u64> {
    let mut leading_zeros = 0;
    while !reader.read_bit()? {
        leading_zeros += 1;
    }

    if leading_zeros >= 32 {
        return Ok((1 << 32) - 1);
    }

    let value = reader.read_bits(leading_zeros)?;
    Ok(value + (1 << leading_zeros) - 1)
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_read_leb128() {
        let mut cursor = std::io::Cursor::new([0b11010101, 0b00101010]);
        let mut reader = BitReader::new(&mut cursor);
        assert_eq!(read_leb128(&mut reader).unwrap(), 0b1010101010101);

        let mut cursor = std::io::Cursor::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        let mut reader = BitReader::new(&mut cursor);

        // 8 bits less because we lose 1 bit from each byte with 8 bytes this means the
        // max we can read is 2^56 - 1
        assert_eq!(read_leb128(&mut reader).unwrap(), (1 << 56) - 1);
    }

    #[test]
    fn test_read_uvlc() {
        let mut cursor = std::io::Cursor::new([0x01, 0xff]);
        let mut reader = BitReader::new(&mut cursor);
        assert_eq!(read_uvlc(&mut reader).unwrap(), 0xfe);

        let mut cursor = std::io::Cursor::new([0x00, 0x00, 0x00, 0x00, 0x01]);
        let mut reader = BitReader::new(&mut cursor);
        assert_eq!(read_uvlc(&mut reader).unwrap(), (1 << 32) - 1);
    }
}
