use std::io::{
    Write, {self},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Buf, Bytes};
use scuffle_bytes_util::{BitWriter, BytesCursorExt};

#[derive(Debug, Clone, PartialEq)]
/// The AVC (H.264) Decoder Configuration Record.
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
pub struct AVCDecoderConfigurationRecord {
    /// By default, this is set to 1. TODO: I couldn't find more info about this in the docs; ctrl+f couldn't find any more instances.
    pub configuration_version: u8,
    /// The `profile_indication` (aka AVCProfileIndication) contains the `profile_idc` u8 from SPS.
    pub profile_indication: u8,
    /// The `profile_compatibility` is a u8, similar to the `profile_idc` and `level_idc` bytes from SPS.
    pub profile_compatibility: u8,
    /// The `level_indication` (aka AVCLevelIndication) contains the `level_idc` u8 from SPS.
    pub level_indication: u8,
    /// The `length_size_minus_one` is the u8 length of the NALUnitLength minus one.
    pub length_size_minus_one: u8,
    /// The `sps` is a vec of SPS, each of which is a u64.
    /// Refer to the SPS struct in the SPS docs for more info.
    pub sps: Vec<Bytes>,
    /// The `pps` is a vec of PPS, each of which is a u64.
    /// These contain syntax elements that can apply layer repesentation(s).
    /// Note that they are supposed to be ordered by ascending PPS ID.
    pub pps: Vec<Bytes>,
    /// An optional `AvccExtendedConfig`. Refer to the AvccExtendedConfig for more info.
    pub extended_config: Option<AvccExtendedConfig>,
}

#[derive(Debug, Clone, PartialEq)]
/// The AVC (H.264) Extended Configuration.
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
pub struct AvccExtendedConfig {
    /// The `chroma_format_idc` as a u8.
    pub chroma_format_idc: u8,
    /// The `bit_depth_luma_minus8` as a u8.
    pub bit_depth_luma_minus8: u8,
    /// The `bit_depth_chroma_minus8` as a u8.
    pub bit_depth_chroma_minus8: u8,
    /// The `sequence_parameter_set_ext` is a vec of SpsExtended, each of which is a u64.
    /// Refer to the SpsExtended struct in the SPS docs for more info.
    pub sequence_parameter_set_ext: Vec<Bytes>,
}

impl AVCDecoderConfigurationRecord {
    /// Demuxes an AVCDecoderConfigurationRecord from a byte stream.
    /// Returns a demuxed AVCDecoderConfigurationRecord.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let configuration_version = reader.read_u8()?;
        let profile_indication = reader.read_u8()?;
        let profile_compatibility = reader.read_u8()?;
        let level_indication = reader.read_u8()?;
        let length_size_minus_one = reader.read_u8()? & 0b00000011;
        let num_of_sequence_parameter_sets = reader.read_u8()? & 0b00011111;

        let mut sps = Vec::with_capacity(num_of_sequence_parameter_sets as usize);
        for _ in 0..num_of_sequence_parameter_sets {
            let sps_length = reader.read_u16::<BigEndian>()?;
            let sps_data = reader.extract_bytes(sps_length as usize)?;
            sps.push(sps_data);
        }

        let num_of_picture_parameter_sets = reader.read_u8()?;
        let mut pps = Vec::with_capacity(num_of_picture_parameter_sets as usize);
        for _ in 0..num_of_picture_parameter_sets {
            let pps_length = reader.read_u16::<BigEndian>()?;
            let pps_data = reader.extract_bytes(pps_length as usize)?;
            pps.push(pps_data);
        }

        // It turns out that sometimes the extended config is not present, even though
        // the avc_profile_indication is not 66, 77 or 88. We need to be lenient here on
        // decoding.
        let extended_config = match profile_indication {
            66 | 77 | 88 => None,
            _ => {
                if reader.has_remaining() {
                    let chroma_format_idc = reader.read_u8()? & 0b00000011; // 2 bits (6 bits reserved)
                    let bit_depth_luma_minus8 = reader.read_u8()? & 0b00000111; // 3 bits (5 bits reserved)
                    let bit_depth_chroma_minus8 = reader.read_u8()? & 0b00000111; // 3 bits (5 bits reserved)
                    let number_of_sequence_parameter_set_ext = reader.read_u8()?; // 8 bits

                    let mut sequence_parameter_set_ext = Vec::with_capacity(number_of_sequence_parameter_set_ext as usize);
                    for _ in 0..number_of_sequence_parameter_set_ext {
                        let sps_ext_length = reader.read_u16::<BigEndian>()?;
                        let sps_ext_data = reader.extract_bytes(sps_ext_length as usize)?;
                        sequence_parameter_set_ext.push(sps_ext_data);
                    }

                    Some(AvccExtendedConfig {
                        chroma_format_idc,
                        bit_depth_luma_minus8,
                        bit_depth_chroma_minus8,
                        sequence_parameter_set_ext,
                    })
                } else {
                    // No extended config present even though avc_profile_indication is not 66, 77
                    // or 88
                    None
                }
            }
        };

        Ok(Self {
            configuration_version,
            profile_indication,
            profile_compatibility,
            level_indication,
            length_size_minus_one,
            sps,
            pps,
            extended_config,
        })
    }

    /// Returns the total byte size of the AVCDecoderConfigurationRecord.
    pub fn size(&self) -> u64 {
        1 // configuration_version
        + 1 // avc_profile_indication
        + 1 // profile_compatibility
        + 1 // avc_level_indication
        + 1 // length_size_minus_one
        + 1 // num_of_sequence_parameter_sets (5 bits reserved, 3 bits)
        + self.sps.iter().map(|sps| {
            2 // sps_length
            + sps.len() as u64
        }).sum::<u64>() // sps
        + 1 // num_of_picture_parameter_sets
        + self.pps.iter().map(|pps| {
            2 // pps_length
            + pps.len() as u64
        }).sum::<u64>() // pps
        + match &self.extended_config {
            Some(config) => {
                1 // chroma_format_idc (6 bits reserved, 2 bits)
                + 1 // bit_depth_luma_minus8 (5 bits reserved, 3 bits)
                + 1 // bit_depth_chroma_minus8 (5 bits reserved, 3 bits)
                + 1 // number_of_sequence_parameter_set_ext
                + config.sequence_parameter_set_ext.iter().map(|sps_ext| {
                    2 // sps_ext_length
                    + sps_ext.len() as u64
                }).sum::<u64>() // sps_ext
            }
            None => 0,
        }
    }

    /// Muxes the AVCDecoderConfigurationRecord into a byte stream.
    /// Returns a muxed byte stream.
    pub fn mux<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_u8(self.configuration_version)?;
        bit_writer.write_u8(self.profile_indication)?;
        bit_writer.write_u8(self.profile_compatibility)?;
        bit_writer.write_u8(self.level_indication)?;
        bit_writer.write_bits(0b111111, 6)?;
        bit_writer.write_bits(self.length_size_minus_one as u64, 2)?;
        bit_writer.write_bits(0b111, 3)?;

        bit_writer.write_bits(self.sps.len() as u64, 5)?;
        for sps in &self.sps {
            bit_writer.write_u16::<BigEndian>(sps.len() as u16)?;
            bit_writer.write_all(sps)?;
        }

        bit_writer.write_bits(self.pps.len() as u64, 8)?;
        for pps in &self.pps {
            bit_writer.write_u16::<BigEndian>(pps.len() as u16)?;
            bit_writer.write_all(pps)?;
        }

        if let Some(config) = &self.extended_config {
            bit_writer.write_bits(0b111111, 6)?;
            bit_writer.write_bits(config.chroma_format_idc as u64, 2)?;
            bit_writer.write_bits(0b11111, 5)?;
            bit_writer.write_bits(config.bit_depth_luma_minus8 as u64, 3)?;
            bit_writer.write_bits(0b11111, 5)?;
            bit_writer.write_bits(config.bit_depth_chroma_minus8 as u64, 3)?;

            bit_writer.write_bits(config.sequence_parameter_set_ext.len() as u64, 8)?;
            for sps_ext in &config.sequence_parameter_set_ext {
                bit_writer.write_u16::<BigEndian>(sps_ext.len() as u16)?;
                bit_writer.write_all(sps_ext)?;
            }
        }

        bit_writer.finish()?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use bytes::Bytes;

    use crate::config::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
    use crate::sps::{ColorConfig, Sps, SpsExtended};

    #[test]
    fn test_config_demux() {
        let data = Bytes::from(b"\x01d\0\x1f\xff\xe1\0\x1dgd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0".to_vec());

        let config = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data)).unwrap();

        assert_eq!(config.configuration_version, 1);
        assert_eq!(config.profile_indication, 100);
        assert_eq!(config.profile_compatibility, 0);
        assert_eq!(config.level_indication, 31);
        assert_eq!(config.length_size_minus_one, 3);
        assert_eq!(
            config.extended_config,
            Some(AvccExtendedConfig {
                bit_depth_chroma_minus8: 0,
                bit_depth_luma_minus8: 0,
                chroma_format_idc: 1,
                sequence_parameter_set_ext: vec![],
            })
        );

        assert_eq!(config.sps.len(), 1);
        assert_eq!(config.pps.len(), 1);

        let sps = &config.sps[0];
        let sps = Sps::parse(sps.clone()).unwrap();

        assert_eq!(sps.profile_idc, 100);
        assert_eq!(sps.level_idc, 31);
        assert_eq!(
            sps.ext,
            Some(SpsExtended {
                chroma_format_idc: 1,
                bit_depth_luma_minus8: 0,
                bit_depth_chroma_minus8: 0,
            })
        );

        assert_eq!(sps.width, 480);
        assert_eq!(sps.height, 852);
        assert_eq!(sps.frame_rate, 30.0);
        assert_eq!(
            sps.color_config,
            Some(ColorConfig {
                full_range: false,
                matrix_coefficients: 1,
                color_primaries: 1,
                transfer_characteristics: 1,
            })
        )
    }

    #[test]
    fn test_config_mux() {
        let data = Bytes::from(b"\x01d\0\x1f\xff\xe1\0\x1dgd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0".to_vec());

        let config = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data.clone())).unwrap();

        assert_eq!(config.size(), data.len() as u64);

        let mut buf = Vec::new();
        config.mux(&mut buf).unwrap();

        assert_eq!(buf, data.to_vec());
    }

    #[test]
    fn test_parse_sps_with_zero_num_units_in_tick() {
        let sps = Bytes::from(b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\0\0\0\x03\x01\xe0x\xc1\x8c\xb0 ".to_vec());
        let sps = Sps::parse(sps);

        match sps {
            Ok(_) => panic!("Expected error for num_units_in_tick = 0, but got Ok"),
            Err(e) => assert_eq!(
                e.kind(),
                std::io::ErrorKind::InvalidData,
                "Expected InvalidData error, got {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_no_ext_cfg_for_profiles_66_77_88() {
        let data = Bytes::from(b"\x01B\x00\x1F\xFF\xE1\x00\x1Dgd\x00\x1F\xAC\xD9A\xE0m\xF9\xE6\xA0  (\x00\x00\x03\x00\x08\x00\x00\x03\x01\xE0x\xC1\x8C\xB0\x01\x00\x06h\xEB\xE3\xCB\"\xC0\xFD\xF8\xF8\x00".to_vec());
        let config = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data)).unwrap();

        assert_eq!(config.extended_config, None);
    }

    #[test]
    fn test_size_calculation_with_sequence_parameter_set_ext() {
        let extended_config = AvccExtendedConfig {
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            sequence_parameter_set_ext: vec![Bytes::from_static(b"extra")],
        };
        let config = AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: vec![Bytes::from_static(b"spsdata")],
            pps: vec![Bytes::from_static(b"ppsdata")],
            extended_config: Some(extended_config),
        };

        assert_eq!(config.size(), 36);
        insta::assert_debug_snapshot!(config, @r#"
        AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: [
                b"spsdata",
            ],
            pps: [
                b"ppsdata",
            ],
            extended_config: Some(
                AvccExtendedConfig {
                    chroma_format_idc: 1,
                    bit_depth_luma_minus8: 0,
                    bit_depth_chroma_minus8: 0,
                    sequence_parameter_set_ext: [
                        b"extra",
                    ],
                },
            ),
        }
        "#);
    }

    #[test]
    fn test_mux_with_sequence_parameter_set_ext() {
        let extended_config = AvccExtendedConfig {
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            sequence_parameter_set_ext: vec![Bytes::from_static(b"extra")],
        };
        let config = AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: vec![Bytes::from_static(b"spsdata")],
            pps: vec![Bytes::from_static(b"ppsdata")],
            extended_config: Some(extended_config),
        };

        let mut buf = Vec::new();
        config.mux(&mut buf).unwrap();

        let demuxed = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(buf.into())).unwrap();
        assert_eq!(demuxed.extended_config.unwrap().sequence_parameter_set_ext.len(), 1);
        insta::assert_debug_snapshot!(config, @r#"
        AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: [
                b"spsdata",
            ],
            pps: [
                b"ppsdata",
            ],
            extended_config: Some(
                AvccExtendedConfig {
                    chroma_format_idc: 1,
                    bit_depth_luma_minus8: 0,
                    bit_depth_chroma_minus8: 0,
                    sequence_parameter_set_ext: [
                        b"extra",
                    ],
                },
            ),
        }
        "#);
    }
}
