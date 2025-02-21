use std::io;

use scuffle_bytes_util::BitReader;
use utils::read_leb128;

pub mod seq;
mod utils;

/// OBU Header
/// AV1-Spec-2 - 5.3.2
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ObuHeader {
    /// `obu_type`
    pub obu_type: ObuType,
    /// `obu_size` if `obu_has_size_field` is 1
    pub size: Option<u64>,
    /// `obu_extension_header()` if `obu_extension_flag` is 1
    pub extension_header: Option<ObuExtensionHeader>,
}

/// Obu Header Extension
/// AV1-Spec-2 - 5.3.3
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ObuExtensionHeader {
    /// `temporal_id`
    pub temporal_id: u8,
    /// `spatial_id`
    pub spatial_id: u8,
}

impl ObuHeader {
    /// Parses an OBU header from the given `cursor`.
    pub fn parse(cursor: &mut impl io::Read) -> io::Result<Self> {
        let mut bit_reader = BitReader::new(cursor);
        let forbidden_bit = bit_reader.read_bit()?;
        if forbidden_bit {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "obu_forbidden_bit is not 0"));
        }

        let obu_type = bit_reader.read_bits(4)?;
        let extension_flag = bit_reader.read_bit()?;
        let has_size_field = bit_reader.read_bit()?;

        bit_reader.read_bit()?; // reserved_1bit

        let extension_header = if extension_flag {
            let temporal_id = bit_reader.read_bits(3)?;
            let spatial_id = bit_reader.read_bits(2)?;
            bit_reader.read_bits(3)?; // reserved_3bits
            Some(ObuExtensionHeader {
                temporal_id: temporal_id as u8,
                spatial_id: spatial_id as u8,
            })
        } else {
            None
        };

        let size = if has_size_field {
            // obu_size
            Some(read_leb128(&mut bit_reader)?)
        } else {
            None
        };

        if !bit_reader.is_aligned() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bit reader is not aligned"));
        }

        Ok(ObuHeader {
            obu_type: ObuType::from(obu_type as u8),
            size,
            extension_header,
        })
    }
}

/// OBU Type
/// AV1-Spec-2 - 6.2.2
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ObuType {
    /// `OBU_SEQUENCE_HEADER`
    SequenceHeader,
    /// `OBU_TEMPORAL_DELIMITER`
    TemporalDelimiter,
    /// `OBU_FRAME_HEADER`
    FrameHeader,
    /// `OBU_TILE_GROUP`
    TileGroup,
    /// `OBU_METADATA`
    Metadata,
    /// `OBU_FRAME`
    Frame,
    /// `OBU_REDUNDANT_FRAME_HEADER`
    RedundantFrameHeader,
    /// `OBU_TILE_LIST`
    TileList,
    /// `OBU_PADDING`
    Padding,
    /// Reserved
    Reserved(u8),
}

impl From<u8> for ObuType {
    fn from(value: u8) -> Self {
        match value {
            1 => ObuType::SequenceHeader,
            2 => ObuType::TemporalDelimiter,
            3 => ObuType::FrameHeader,
            4 => ObuType::TileGroup,
            5 => ObuType::Metadata,
            6 => ObuType::Frame,
            7 => ObuType::RedundantFrameHeader,
            8 => ObuType::TileList,
            15 => ObuType::Padding,
            _ => ObuType::Reserved(value),
        }
    }
}

impl From<ObuType> for u8 {
    fn from(value: ObuType) -> Self {
        match value {
            ObuType::SequenceHeader => 1,
            ObuType::TemporalDelimiter => 2,
            ObuType::FrameHeader => 3,
            ObuType::TileGroup => 4,
            ObuType::Metadata => 5,
            ObuType::Frame => 6,
            ObuType::RedundantFrameHeader => 7,
            ObuType::TileList => 8,
            ObuType::Padding => 15,
            ObuType::Reserved(value) => value,
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use bytes::Buf;

    use super::*;

    #[test]
    fn test_obu_header_parse() {
        let mut cursor = std::io::Cursor::new(b"\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@");
        let header = ObuHeader::parse(&mut cursor).unwrap();
        insta::assert_debug_snapshot!(header, @r"
        ObuHeader {
            obu_type: SequenceHeader,
            size: Some(
                15,
            ),
            extension_header: None,
        }
        ");

        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.remaining(), 15);
    }

    #[test]
    fn test_obu_header_parse_no_size_field() {
        let mut cursor = std::io::Cursor::new(b"\x00");
        let header = ObuHeader::parse(&mut cursor).unwrap();
        insta::assert_debug_snapshot!(header, @r"
        ObuHeader {
            obu_type: Reserved(
                0,
            ),
            size: None,
            extension_header: None,
        }
        ");

        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn test_obu_header_parse_extension_header() {
        let mut cursor = std::io::Cursor::new([0b00000100, 0b11010000]);
        let header = ObuHeader::parse(&mut cursor).unwrap();
        insta::assert_debug_snapshot!(header, @r"
        ObuHeader {
            obu_type: Reserved(
                0,
            ),
            size: None,
            extension_header: Some(
                ObuExtensionHeader {
                    temporal_id: 6,
                    spatial_id: 2,
                },
            ),
        }
        ");

        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn test_obu_header_forbidden_bit_set() {
        let err = ObuHeader::parse(&mut std::io::Cursor::new(
            b"\xff\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@",
        ))
        .unwrap_err();
        insta::assert_debug_snapshot!(err, @r#"
        Custom {
            kind: InvalidData,
            error: "obu_forbidden_bit is not 0",
        }
        "#);
    }

    #[test]
    fn test_obu_to_from_u8() {
        let case = [
            (ObuType::SequenceHeader, 1),
            (ObuType::TemporalDelimiter, 2),
            (ObuType::FrameHeader, 3),
            (ObuType::TileGroup, 4),
            (ObuType::Metadata, 5),
            (ObuType::Frame, 6),
            (ObuType::RedundantFrameHeader, 7),
            (ObuType::TileList, 8),
            (ObuType::Padding, 15),
            (ObuType::Reserved(0), 0),
            (ObuType::Reserved(100), 100),
        ];

        for (obu_type, value) in case {
            assert_eq!(u8::from(obu_type), value);
            assert_eq!(ObuType::from(value), obu_type);
        }
    }
}
