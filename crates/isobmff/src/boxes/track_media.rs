//! Track media structure boxes defined in ISO/IEC 14496-12 - 8.4

use byteorder::ReadBytesExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::SampleTableBox;
use crate::string_deserializer::Utf8String;
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox};

/// Media box
///
/// ISO/IEC 14496-12 - 8.4.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mdia", crate_path = "crate")]
pub struct MediaBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub mdhd: MediaHeaderBox,
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    #[iso_box(nested_box)]
    pub minf: MediaInformationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub elng: Option<ExtendedLanguageBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Media header box
///
/// ISO/IEC 14496-12 - 8.4.2
#[derive(Debug)]
pub struct MediaHeaderBox {
    pub header: FullBoxHeader,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: [u8; 3],
    pub pre_defined: u16,
}

// Manual implementation because conditional fields are not supported in the macro

impl IsoBox for MediaHeaderBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"mdhd";
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for MediaHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let creation_time = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };
        let modification_time = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };
        let timescale = reader.as_std().read_u32::<byteorder::BigEndian>()?;
        let duration = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };

        // 0 xxxxx xxxxx xxxxx
        let language = reader.as_std().read_u16::<byteorder::BigEndian>()?;
        let language = [
            ((language >> 10) & 0b11111) as u8,
            ((language >> 5) & 0b11111) as u8,
            (language & 0b11111) as u8,
        ];
        let pre_defined = reader.as_std().read_u16::<byteorder::BigEndian>()?;

        Ok(Self {
            header: seed,
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
            pre_defined,
        })
    }
}

impl<'a> Deserialize<'a> for MediaHeaderBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl MediaHeaderBox {
    pub fn language(&self) -> [char; 3] {
        [
            (self.language[0] + 0x60) as char,
            (self.language[1] + 0x60) as char,
            (self.language[2] + 0x60) as char,
        ]
    }
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"hdlr", crate_path = "crate")]
pub struct HandlerBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub pre_defined: u32,
    pub handler_type: u32,
    _reserved: [u32; 3],
    pub name: Utf8String,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"minf", crate_path = "crate")]
pub struct MediaInformationBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub stbl: SampleTableBox<'a>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"nmhd", crate_path = "crate")]
pub struct NullMediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"elng", crate_path = "crate")]
pub struct ExtendedLanguageBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub extended_language: Utf8String,
}
