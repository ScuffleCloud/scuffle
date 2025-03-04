use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;

use super::define::ProtocolControlMessageSetChunkSize;
use super::errors::ProtocolControlMessageError;
use super::{
    ProtocolControlMessageSetPeerBandwidth, ProtocolControlMessageSetPeerBandwidthLimitType,
    ProtocolControlMessageWindowAcknowledgementSize,
};

impl ProtocolControlMessageSetChunkSize {
    pub fn read(data: &[u8]) -> Result<Self, ProtocolControlMessageError> {
        let mut cursor = Cursor::new(data);
        let chunk_size = cursor.read_u32::<BigEndian>()?;
        Ok(Self { chunk_size })
    }
}

impl ProtocolControlMessageWindowAcknowledgementSize {
    pub fn read(data: &[u8]) -> Result<Self, ProtocolControlMessageError> {
        let mut cursor = Cursor::new(data);
        let acknowledgement_window_size = cursor.read_u32::<BigEndian>()?;
        Ok(Self {
            acknowledgement_window_size,
        })
    }
}

impl ProtocolControlMessageSetPeerBandwidth {
    pub fn read(data: &[u8]) -> Result<Self, ProtocolControlMessageError> {
        let mut cursor = Cursor::new(data);
        let acknowledgement_window_size = cursor.read_u32::<BigEndian>()?;
        let limit_type = cursor.read_u8()?;
        let limit_type = ProtocolControlMessageSetPeerBandwidthLimitType::from_u8(limit_type)
            .ok_or(ProtocolControlMessageError::InvalidLimitType(limit_type))?;

        Ok(Self {
            acknowledgement_window_size,
            limit_type,
        })
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
