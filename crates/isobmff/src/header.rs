use std::fmt::Debug;
use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be, ZeroCopyReader};

use crate::IsoSized;

/// Represents the size of a box.
///
/// Use [`Self::size`] to get the size as a number of bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSize {
    /// 32-bit encoded size. Can't be 0 or 1.
    Short(u32),
    /// 64-bit encoded size.
    Long(u64),
    /// The box this size beongs to goes to the end of the file.
    ToEnd,
}

impl BoxSize {
    /// Returns the size as a number of bytes.
    ///
    /// Returns [`None`] if this is a [`BoxSize::ToEnd`].
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

/// Represents the box type.
///
/// Can either be a FourCC value or a user-defined UUID.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    /// A FourCC value, which is a 4-byte identifier. Cannot be "uuid".
    FourCc([u8; 4]),
    /// A user extended identifier, which is a 16-byte UUID.
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
    /// Returns `true` if this is a FourCC value that matches the given FourCC.
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

/// Represents the header of any box.
///
/// Every ISOBMFF box starts with this header, even boxes inheriting from `FullBox`.
/// Please use [`FullBoxHeader`] in combination with this to represent full boxes,
/// as [`FullBoxHeader`] only contains the `version` and `flags` field.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoxHeader {
    /// An integer that specifies the number of bytes in this box, including all its fields and contained
    /// boxes.
    pub size: BoxSize,
    /// Identifies the box type.
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
    /// Uses the size stored in this header to calculate the size of the payload.
    ///
    /// Returns `None` if the box size is [`BoxSize::ToEnd`], as the payload size cannot be determined.
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

/// Contains the `version` and `flags` fields.
///
/// **Attention**: This does NOT represent the `FullBoxHeader` defined by ISO/IEC 14496-12 - 4.
/// This struct only contains the additional fields (compared to [`BoxHeader`]), which are `version` and `flags`.
/// That means that every box, even boxes inheriting from `FullBox`, should contain the [`BoxHeader`] as well.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct FullBoxHeader {
    /// Is an integer that specifies the version of this format of the box.
    pub version: u8,
    /// A map of flags.
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
