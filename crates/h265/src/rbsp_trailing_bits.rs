use std::io;

use scuffle_bytes_util::BitReader;

/// Described by ISO/IEC 23008-2 - 7.4.2.1
pub(crate) fn rbsp_trailing_bits<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<()> {
    let rbsp_stop_one_bit = bit_reader.read_bit()?;
    if !rbsp_stop_one_bit {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "rbsp_stop_one_bit must be 1"));
    }

    // Skip to the end of the current byte (all rbsp_alignment_zero_bits)
    bit_reader.align()?;

    Ok(())
}
