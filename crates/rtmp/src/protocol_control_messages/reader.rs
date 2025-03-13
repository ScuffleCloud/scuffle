use std::io::{self, Cursor};

use byteorder::{BigEndian, ReadBytesExt};

use super::define::ProtocolControlMessageSetChunkSize;

impl ProtocolControlMessageSetChunkSize {
    pub fn read(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        let chunk_size = cursor.read_u32::<BigEndian>()?;
        Ok(Self { chunk_size })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_reader_read_set_chunk_size() {
        let data = vec![0x00, 0x00, 0x00, 0x01];
        let chunk_size = ProtocolControlMessageSetChunkSize::read(&data).unwrap();
        assert_eq!(chunk_size.chunk_size, 1);
    }
}
