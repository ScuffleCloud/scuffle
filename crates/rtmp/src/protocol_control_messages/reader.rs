//! Reading protocol control messages.

use std::io::{self, Cursor};

use byteorder::{BigEndian, ReadBytesExt};

use super::{ProtocolControlMessageSetChunkSize, ProtocolControlMessageWindowAcknowledgementSize};

impl ProtocolControlMessageSetChunkSize {
    /// Reads a [`ProtocolControlMessageSetChunkSize`] from the given data.
    pub fn read(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        let chunk_size = cursor.read_u32::<BigEndian>()?;

        Ok(Self { chunk_size })
    }
}

impl ProtocolControlMessageWindowAcknowledgementSize {
    /// Reads a [`ProtocolControlMessageWindowAcknowledgementSize`] from the given data.
    pub fn read(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        let acknowledgement_window_size = cursor.read_u32::<BigEndian>()?;

        Ok(Self {
            acknowledgement_window_size,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn read_set_chunk_size() {
        let data = vec![0x00, 0x00, 0x00, 0x01];
        let chunk_size = ProtocolControlMessageSetChunkSize::read(&data).unwrap();
        assert_eq!(chunk_size.chunk_size, 1);
    }

    #[test]
    fn read_window_acknowledgement_size() {
        let data = vec![0x00, 0x00, 0x00, 0x01];
        let window_acknowledgement_size = ProtocolControlMessageWindowAcknowledgementSize::read(&data).unwrap();
        assert_eq!(window_acknowledgement_size.acknowledgement_window_size, 1);
    }
}
