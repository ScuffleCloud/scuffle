use std::io::{
    Write, {self},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};
use scuffle_bytes_util::{BitReader, BitWriter, BytesCow, IoResultExt};

use crate::sps::SpsExtended;

/// The AVC (H.264) Decoder Configuration Record.
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AVCDecoderConfigurationRecord<'a> {
    /// The `configuration_version` is set to 1 (as a u8) defined by the h264 spec until further notice.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub configuration_version: u8,

    /// The `profile_indication` (aka AVCProfileIndication) contains the `profile_idc` u8 from SPS.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub profile_indication: u8,

    /// The `profile_compatibility` is a u8, similar to the `profile_idc` and `level_idc` bytes from SPS.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub profile_compatibility: u8,

    /// The `level_indication` (aka AVCLevelIndication) contains the `level_idc` u8 from SPS.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub level_indication: u8,

    /// The `length_size_minus_one` is the u8 length of the NALUnitLength minus one.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub length_size_minus_one: u8,

    /// The `sps` is a vec of SPS Bytes.
    ///
    /// Note that these should be ordered by ascending SPS ID.
    ///
    /// Refer to the [`crate::Sps`] struct in the SPS docs for more info.
    pub sps: Vec<BytesCow<'a>>,

    /// The `pps` is a vec of PPS Bytes.
    ///
    /// These contain syntax elements that can apply layer repesentation(s).
    ///
    /// Note that these should be ordered by ascending PPS ID.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub pps: Vec<BytesCow<'a>>,

    /// An optional `AvccExtendedConfig`.
    ///
    /// Refer to the AvccExtendedConfig for more info.
    pub extended_config: Option<AvccExtendedConfig>,
}

/// The AVC (H.264) Extended Configuration.
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvccExtendedConfig {
    /// The `chroma_format_idc` as a u8.
    ///
    /// Also labelled as `chroma_format`, this contains the `chroma_format_idc` from
    /// ISO/IEC 14496-10.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub chroma_format_idc: u8,

    /// The `bit_depth_luma_minus8` is the bit depth of samples in the Luma arrays as a u8.
    ///
    /// The value of this ranges from \[0, 4\].
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub bit_depth_luma_minus8: u8,

    /// The `bit_depth_chroma_minus8` is the bit depth of the samples in the Chroma arrays as a u8.
    ///
    /// The value of this ranges from \[0, 4\].
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub bit_depth_chroma_minus8: u8,

    /// The `sequence_parameter_set_ext` is a vec of SpsExtended Bytes.
    ///
    /// Refer to the [`crate::SpsExtended`] struct in the SPS docs for more info.
    pub sequence_parameter_set_ext: Vec<SpsExtended>,
}

impl<'a> Deserialize<'a> for AVCDecoderConfigurationRecord<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let configuration_version = reader.as_std().read_u8()?;
        let profile_indication = reader.as_std().read_u8()?;
        let profile_compatibility = reader.as_std().read_u8()?;
        let level_indication = reader.as_std().read_u8()?;
        let length_size_minus_one = reader.as_std().read_u8()? & 0b00000011;
        let num_of_sequence_parameter_sets = reader.as_std().read_u8()? & 0b00011111;

        let mut sps = Vec::with_capacity(num_of_sequence_parameter_sets as usize);
        for _ in 0..num_of_sequence_parameter_sets {
            let sps_length = reader.as_std().read_u16::<BigEndian>()?;
            let sps_data = reader.try_read(sps_length as usize)?;
            sps.push(sps_data);
        }

        let num_of_picture_parameter_sets = reader.as_std().read_u8()?;
        let mut pps = Vec::with_capacity(num_of_picture_parameter_sets as usize);
        for _ in 0..num_of_picture_parameter_sets {
            let pps_length = reader.as_std().read_u16::<BigEndian>()?;
            let pps_data = reader.try_read(pps_length as usize)?;
            pps.push(pps_data);
        }

        // It turns out that sometimes the extended config is not present, even though
        // the avc_profile_indication is not 66, 77 or 88. We need to be lenient here on
        // decoding.
        let extended_config = match profile_indication {
            66 | 77 | 88 => None,
            _ => {
                let chroma_format_idc = reader.as_std().read_u8().eof_to_none()?;
                if let Some(chroma_format_idc) = chroma_format_idc {
                    let chroma_format_idc = chroma_format_idc & 0b00000011; // 2 bits (6 bits reserved)
                    let bit_depth_luma_minus8 = reader.as_std().read_u8()? & 0b00000111; // 3 bits (5 bits reserved)
                    let bit_depth_chroma_minus8 = reader.as_std().read_u8()? & 0b00000111; // 3 bits (5 bits reserved)
                    let number_of_sequence_parameter_set_ext = reader.as_std().read_u8()?; // 8 bits

                    let mut sequence_parameter_set_ext = Vec::with_capacity(number_of_sequence_parameter_set_ext as usize);
                    for _ in 0..number_of_sequence_parameter_set_ext {
                        let sps_ext_length = reader.as_std().read_u16::<BigEndian>()?;
                        let sps_ext_data = reader.try_read(sps_ext_length as usize)?;

                        let mut bit_reader = BitReader::new_from_slice(sps_ext_data);
                        let sps_ext_parsed = SpsExtended::parse(&mut bit_reader)?;
                        sequence_parameter_set_ext.push(sps_ext_parsed);
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
}

impl Serialize for AVCDecoderConfigurationRecord<'_> {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
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
            bit_writer.write_all(sps.as_bytes())?;
        }

        bit_writer.write_bits(self.pps.len() as u64, 8)?;
        for pps in &self.pps {
            bit_writer.write_u16::<BigEndian>(pps.len() as u16)?;
            bit_writer.write_all(pps.as_bytes())?;
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
                bit_writer.write_u16::<BigEndian>(sps_ext.bytesize() as u16)?;
                // SpsExtended::build() does not automatically align the writer
                // due to the fact that it's used when building the Sps.
                // If SpsExtended::build() were to align the writer, it could
                // potentially cause a mismatch as it might introduce 0-padding in
                // the middle of the bytestream, as the bytestream should only be aligned
                // at the very end.
                // In this case however, we want to intentionally align the writer as
                // the sps is the only thing here.
                sps_ext.build(&mut bit_writer)?;
                bit_writer.align()?;
            }
        }

        bit_writer.finish()?;

        Ok(())
    }
}

#[cfg(feature = "isobmff")]
impl isobmff::IsoSized for AVCDecoderConfigurationRecord<'_> {
    /// Returns the total byte size of the AVCDecoderConfigurationRecord.
    fn size(&self) -> usize {
        1 // configuration_version
        + 1 // avc_profile_indication
        + 1 // profile_compatibility
        + 1 // avc_level_indication
        + 1 // length_size_minus_one
        + 1 // num_of_sequence_parameter_sets (5 bits reserved, 3 bits)
        + self.sps.iter().map(|sps| {
            2 // sps_length
            + sps.len()
        }).sum::<usize>() // sps
        + 1 // num_of_picture_parameter_sets
        + self.pps.iter().map(|pps| {
            2 // pps_length
            + pps.len()
        }).sum::<usize>() // pps
        + match &self.extended_config {
            Some(config) => {
                1 // chroma_format_idc (6 bits reserved, 2 bits)
                + 1 // bit_depth_luma_minus8 (5 bits reserved, 3 bits)
                + 1 // bit_depth_chroma_minus8 (5 bits reserved, 3 bits)
                + 1 // number_of_sequence_parameter_set_ext
                + config.sequence_parameter_set_ext.iter().map(|sps_ext| {
                    2 // sps_ext_length
                    + sps_ext.bytesize() as usize // sps_ext
                }).sum::<usize>()
            }
            None => 0,
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io::Write;

    use byteorder::{BigEndian, WriteBytesExt};
    use isobmff::IsoSized;
    use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};
    use scuffle_bytes_util::{BitWriter, BytesCow};

    use crate::config::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
    use crate::sps::SpsExtended;

    #[test]
    fn test_config_parse() {
        let sample_sps = b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0";
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        // configuration_version
        writer.write_bits(1, 8).unwrap();
        // profile_indication
        writer.write_bits(100, 8).unwrap();
        // profile_compatibility
        writer.write_bits(0, 8).unwrap();
        // level_indication
        writer.write_bits(31, 8).unwrap();
        // length_size_minus_one
        writer.write_bits(3, 8).unwrap();

        // num_of_sequence_parameter_sets
        writer.write_bits(1, 8).unwrap();
        // sps_length
        writer.write_u16::<BigEndian>(sample_sps.len() as u16).unwrap();
        // sps
        // this was from the old test
        writer.write_all(sample_sps).unwrap();

        // num_of_picture_parameter_sets
        writer.write_bits(1, 8).unwrap();
        // pps_length
        writer.write_bits(6, 16).unwrap();
        writer.write_all(b"h\xeb\xe3\xcb\"\xc0\x00\x00").unwrap();

        // chroma_format_idc
        writer.write_bits(1, 8).unwrap();
        // bit_depth_luma_minus8
        writer.write_bits(0, 8).unwrap();
        // bit_depth_chroma_minus8
        writer.write_bits(0, 8).unwrap();
        // number_of_sequence_parameter_set_ext
        writer.write_bits(0, 8).unwrap();
        writer.finish().unwrap();

        let result =
            AVCDecoderConfigurationRecord::deserialize(scuffle_bytes_util::zero_copy::Slice::from(&data[..])).unwrap();

        let sps = &result.sps[0];

        assert_eq!(sps.as_bytes(), *sample_sps);
    }

    #[test]
    fn test_config_build() {
        // these may not be the same size due to the natural reduction from the SPS parsing.
        // in specific, the sps size function may return a lower size than the original bitstring.
        // reduction will occur from rebuilding the sps and from rebuilding the sps_ext.
        let data = b"\x01d\0\x1f\xff\xe1\0\x19\x67\x64\x00\x1F\xAC\xD9\x41\xE0\x6D\xF9\xE6\xA0\x20\x20\x28\x00\x00\x03\x00\x08\x00\x00\x03\x01\xE0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0";

        let config =
            AVCDecoderConfigurationRecord::deserialize(scuffle_bytes_util::zero_copy::Slice::from(&data[..])).unwrap();

        assert_eq!(config.size(), data.len());

        let mut buf = Vec::new();
        config.serialize(&mut buf).unwrap();

        assert_eq!(buf, data.to_vec());
    }

    #[test]
    fn test_no_ext_cfg_for_profiles_66_77_88() {
        let data = b"\x01B\x00\x1F\xFF\xE1\x00\x1Dgd\x00\x1F\xAC\xD9A\xE0m\xF9\xE6\xA0  (\x00\x00\x03\x00\x08\x00\x00\x03\x01\xE0x\xC1\x8C\xB0\x01\x00\x06h\xEB\xE3\xCB\"\xC0\xFD\xF8\xF8\x00";
        let config =
            AVCDecoderConfigurationRecord::deserialize(scuffle_bytes_util::zero_copy::Slice::from(&data[..])).unwrap();

        assert_eq!(config.extended_config, None);
    }

    #[test]
    fn test_size_calculation_with_sequence_parameter_set_ext() {
        let extended_config = AvccExtendedConfig {
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            sequence_parameter_set_ext: vec![SpsExtended {
                chroma_format_idc: 1,
                separate_color_plane_flag: false,
                bit_depth_luma_minus8: 2,
                bit_depth_chroma_minus8: 3,
                qpprime_y_zero_transform_bypass_flag: false,
                scaling_matrix: vec![],
            }],
        };
        let config = AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: vec![BytesCow::from_static(
                b"\x67\x64\x00\x1F\xAC\xD9\x41\xE0\x6D\xF9\xE6\xA0\x20\x20\x28\x00\x00\x00\x08\x00\x00\x01\xE0",
            )],
            pps: vec![BytesCow::from_static(b"ppsdata")],
            extended_config: Some(extended_config),
        };

        assert_eq!(config.size(), 49);
        insta::assert_debug_snapshot!(config, @r#"
        AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: [
                b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\0\x08\0\0\x01\xe0",
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
                        SpsExtended {
                            chroma_format_idc: 1,
                            separate_color_plane_flag: false,
                            bit_depth_luma_minus8: 2,
                            bit_depth_chroma_minus8: 3,
                            qpprime_y_zero_transform_bypass_flag: false,
                            scaling_matrix: [],
                        },
                    ],
                },
            ),
        }
        "#);
    }

    #[test]
    fn test_build_with_sequence_parameter_set_ext() {
        let extended_config = AvccExtendedConfig {
            chroma_format_idc: 1,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            sequence_parameter_set_ext: vec![SpsExtended {
                chroma_format_idc: 1,
                separate_color_plane_flag: false,
                bit_depth_luma_minus8: 2,
                bit_depth_chroma_minus8: 3,
                qpprime_y_zero_transform_bypass_flag: false,
                scaling_matrix: vec![],
            }],
        };
        let config = AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: vec![BytesCow::from_static(
                b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0",
            )],
            pps: vec![BytesCow::from_static(b"ppsdata")],
            extended_config: Some(extended_config),
        };

        let mut buf = Vec::new();
        config.serialize(&mut buf).unwrap();

        let parsed =
            AVCDecoderConfigurationRecord::deserialize(scuffle_bytes_util::zero_copy::Slice::from(&buf[..])).unwrap();
        assert_eq!(parsed.extended_config.unwrap().sequence_parameter_set_ext.len(), 1);
        insta::assert_debug_snapshot!(config, @r#"
        AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: [
                b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0",
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
                        SpsExtended {
                            chroma_format_idc: 1,
                            separate_color_plane_flag: false,
                            bit_depth_luma_minus8: 2,
                            bit_depth_chroma_minus8: 3,
                            qpprime_y_zero_transform_bypass_flag: false,
                            scaling_matrix: [],
                        },
                    ],
                },
            ),
        }
        "#);
    }
}
