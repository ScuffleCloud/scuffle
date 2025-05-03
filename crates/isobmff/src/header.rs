use std::fmt::Debug;
use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use scuffle_bytes_util::zero_copy::ZeroCopyReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    FourCc([u8; 4]),
    Uuid(uuid::Uuid),
}

impl Debug for BoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoxType::FourCc(fourcc) => f.debug_tuple("FourCc").field(&String::from_utf8_lossy(fourcc)).finish(),
            BoxType::Uuid(uuid) => f.debug_tuple("Uuid").field(uuid).finish(),
        }
    }
}

impl BoxType {
    pub fn is_four_cc(&self, four_cc: &[u8; 4]) -> bool {
        match self {
            BoxType::FourCc(box_four_cc) => box_four_cc == four_cc,
            BoxType::Uuid(_) => false,
        }
    }
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

pub trait BoxHeaderProperties {
    fn size(&self) -> usize;
    fn box_size(&self) -> BoxSize;
    fn payload_size(&self) -> Option<usize> {
        let header_size = self.size();
        Some(self.box_size().size()?.saturating_sub(header_size))
    }
    fn box_type(&self) -> BoxType;
}

#[derive(Debug, Clone)]
pub struct BoxHeader {
    pub size: BoxSize,
    pub box_type: BoxType,
}

impl BoxHeaderProperties for BoxHeader {
    fn size(&self) -> usize {
        let mut size = 4 + 4; // size + type

        if let BoxSize::Long(_) = self.size {
            size += 8; // large size
        }

        if let BoxType::Uuid(_) = self.box_type {
            size += 16; // usertype
        }

        size
    }

    fn box_size(&self) -> BoxSize {
        self.size
    }

    fn box_type(&self) -> BoxType {
        self.box_type
    }
}

impl<'a> scuffle_bytes_util::zero_copy::Deserialize<'a> for BoxHeader {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
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
}

impl<'a> scuffle_bytes_util::zero_copy::DeserializeSeed<'a, BoxHeader> for BoxHeader {
    fn deserialize_seed<R>(_reader: R, seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(seed)
    }
}

#[derive(Debug, Clone)]
pub struct FullBoxHeader {
    pub header: BoxHeader,
    pub version: u8,
    /// only lower 24 bits are used
    pub flags: u32,
}

impl BoxHeaderProperties for FullBoxHeader {
    fn size(&self) -> usize {
        self.header.size()
            + 1 // version
            + 3 // flags
    }

    fn box_size(&self) -> BoxSize {
        self.header.box_size()
    }

    fn box_type(&self) -> BoxType {
        self.header.box_type()
    }
}

impl<'a> scuffle_bytes_util::zero_copy::DeserializeSeed<'a, BoxHeader> for FullBoxHeader {
    fn deserialize_seed<R>(mut reader: R, seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let version = reader.as_std().read_u8()?;
        let flags = reader.as_std().read_u24::<BigEndian>()?;

        Ok(Self {
            header: seed,
            version,
            flags,
        })
    }
}
