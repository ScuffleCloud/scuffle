use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use scuffle_bytes_util::zero_copy::ZeroCopyReader;

pub enum BoxSize {
    Short(u32),
    Long(u64),
    ToEnd,
}

impl BoxSize {
    pub fn size(&self) -> Option<usize> {
        match self {
            BoxSize::Short(size) => Some(*size as usize),
            BoxSize::Long(size) => Some(*size as usize),
            BoxSize::ToEnd => None,
        }
    }
}

pub enum BoxType {
    FourCc([u8; 4]),
    Uuid(uuid::Uuid),
}

impl From<BoxType> for uuid::Uuid {
    fn from(box_type: BoxType) -> Self {
        match box_type {
            BoxType::FourCc(fourcc) => {
                #[rustfmt::skip]
                let bytes = [
                    fourcc[0], fourcc[1], fourcc[2], fourcc[3],
                    0x00, 0x11, 0x00, 0x10, 0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71,
                ];
                uuid::Uuid::from_bytes(bytes)
            }
            BoxType::Uuid(uuid) => uuid,
        }
    }
}

pub struct BoxHeader {
    pub size: BoxSize,
    pub box_type: BoxType,
}

impl BoxHeader {
    pub fn demux<'a, R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        let size = reader.as_std().read_u32::<BigEndian>()?;
        let box_type = reader.as_std().read_u32::<BigEndian>()?.to_be_bytes();

        let size = match size {
            0 => BoxSize::ToEnd,
            1 => {
                let size = reader.as_std().read_u64::<BigEndian>()?;
                BoxSize::Long(size)
            }
            _ => BoxSize::Short(size),
        };

        let box_type = if box_type == *b"uuid" {
            let uuid = reader.as_std().read_u128::<BigEndian>()?;
            let uuid = uuid::Uuid::from_u128(uuid);
            BoxType::Uuid(uuid)
        } else {
            BoxType::FourCc(box_type)
        };

        Ok(Self { size, box_type })
    }

    pub fn size(&self) -> usize {
        let mut size = 4 + 4; // size + type

        if let BoxSize::Long(_) = self.size {
            size += 8; // large size
        }

        if let BoxType::Uuid(_) = self.box_type {
            size += 16; // usertype
        }

        size
    }

    pub fn payload_size(&self) -> Option<usize> {
        let header_size = self.size();
        Some(header_size - self.size.size()?)
    }
}

pub struct FullBoxHeader {
    pub version: u8,
    /// only lower 24 bits are used
    pub flags: u32,
}

impl FullBoxHeader {
    pub fn demux<'a, R: ZeroCopyReader<'a>>(mut reader: R) -> io::Result<Self> {
        let version = reader.as_std().read_u8()?;
        let flags = reader.as_std().read_u24::<BigEndian>()?;
        Ok(Self { version, flags })
    }
}
