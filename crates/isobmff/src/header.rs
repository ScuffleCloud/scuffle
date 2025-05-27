use std::fmt::Debug;
use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be, ZeroCopyReader};

use crate::IsoSized;

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

impl From<usize> for BoxSize {
    fn from(value: usize) -> Self {
        // If the size does not fit in a u32 we use a long size.
        // 0 and 1 are reserved for special cases, so we have to use long size for them too.
        if value > u32::MAX as usize || value == 0 || value == 1 {
            BoxSize::Long(value as u64)
        } else {
            BoxSize::Short(value as u32)
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoxHeader {
    pub size: BoxSize,
    pub box_type: BoxType,
}

impl IsoSized for BoxHeader {
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
}

impl BoxHeader {
    pub fn payload_size(&self) -> Option<usize> {
        let header_size = self.size();
        Some(self.size.size()?.saturating_sub(header_size))
    }
}

impl<'a> Deserialize<'a> for BoxHeader {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let size = u32::deserialize(&mut reader)?;
        let box_type = u32::deserialize(&mut reader)?.to_be_bytes();

        let size = match size {
            0 => BoxSize::ToEnd,
            1 => {
                let size = u64::deserialize(&mut reader)?;
                BoxSize::Long(size)
            }
            _ => BoxSize::Short(size),
        };

        let box_type = if box_type == *b"uuid" {
            let uuid = u128::deserialize(&mut reader)?;
            let uuid = uuid::Uuid::from_u128(uuid);
            BoxType::Uuid(uuid)
        } else {
            BoxType::FourCc(box_type)
        };

        Ok(Self { size, box_type })
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for BoxHeader {
    fn deserialize_seed<R>(_reader: R, seed: BoxHeader) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        Ok(seed)
    }
}

impl Serialize for BoxHeader {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let size = match self.size {
            BoxSize::Short(size) => size,
            BoxSize::Long(_) => 1,
            BoxSize::ToEnd => 0,
        };
        size.serialize(&mut writer)?;

        let box_type = match &self.box_type {
            BoxType::FourCc(fourcc) => fourcc,
            BoxType::Uuid(_) => b"uuid",
        };
        box_type.serialize(&mut writer)?;

        if let BoxSize::Long(size) = self.size {
            size.serialize(&mut writer)?;
        }

        if let BoxType::Uuid(uuid) = &self.box_type {
            uuid.as_u128().to_be_bytes().serialize(&mut writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct FullBoxHeader {
    pub version: u8,
    pub flags: U24Be,
}

impl IsoSized for FullBoxHeader {
    fn size(&self) -> usize {
        1 // version
        + 3 // flags
    }
}

impl<'a> Deserialize<'a> for FullBoxHeader {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let version = u8::deserialize(&mut reader)?;
        let flags = U24Be::deserialize(&mut reader)?;

        Ok(Self { version, flags })
    }
}

impl Serialize for FullBoxHeader {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.version.serialize(&mut writer)?;
        self.flags.serialize(&mut writer)?;

        Ok(())
    }
}
