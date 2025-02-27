use std::io::{
    Write, {self},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Buf, Bytes};
use scuffle_bytes_util::{BitReader, BitWriter, BytesCursorExt};

use crate::{Sps, SpsExtended};

/// The AVC (H.264) Decoder Configuration Record.
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
#[derive(Debug, Clone, PartialEq)]
pub struct AVCDecoderConfigurationRecord {
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
    pub sps: Vec<Sps>,

    /// The `pps` is a vec of PPS Bytes.
    ///
    /// These contain syntax elements that can apply layer repesentation(s).
    ///
    /// Note that these should be ordered by ascending PPS ID.
    ///
    /// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
    pub pps: Vec<Bytes>,

    /// An optional `AvccExtendedConfig`.
    ///
    /// Refer to the AvccExtendedConfig for more info.
    pub extended_config: Option<AvccExtendedConfig>,
}

/// The AVC (H.264) Extended Configuration.
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
#[derive(Debug, Clone, PartialEq)]
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

impl AVCDecoderConfigurationRecord {
    /// Parses an AVCDecoderConfigurationRecord from a byte stream.
    /// Returns a parsed AVCDecoderConfigurationRecord.
    pub fn parse(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
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
            let sps_parsed = Sps::parse(&sps_data)?;
            sps.push(sps_parsed);
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
            + sps.size()
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
                    + sps_ext.bytesize() // sps_ext
                }).sum::<u64>()
            }
            None => 0,
        }
    }

    /// Builds the AVCDecoderConfigurationRecord into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
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
            bit_writer.write_u16::<BigEndian>(sps.size() as u16)?;
            // Sps::build() automatically aligns the writer.
            sps.build(&mut bit_writer)?;
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

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io::{self, Write};

    use bytes::Bytes;
    use scuffle_bytes_util::BitWriter;

    use crate::config::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
    use crate::{Sps, SpsExtended};

    #[test]
    fn test_config_parse() {
        let sample_sps = b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0";
        let reduced_sps = Sps::reduce(sample_sps).unwrap();
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
        writer.write_bits(reduced_sps.len() as u64, 16).unwrap();
        // sps
        // this was from the old test
        writer.write_all(&reduced_sps).unwrap();

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

        let result = AVCDecoderConfigurationRecord::parse(&mut io::Cursor::new(data.into())).unwrap();

        let sps = &result.sps[0];

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            emu_bytes: 2,
            forbidden_zero_bit: false,
            nal_ref_idc: 3,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 100,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 31,
            seq_parameter_set_id: 0,
            ext: Some(
                SpsExtended {
                    chroma_format_idc: 1,
                    separate_color_plane_flag: false,
                    bit_depth_luma_minus8: 0,
                    bit_depth_chroma_minus8: 0,
                    qpprime_y_zero_transform_bypass_flag: false,
                    scaling_matrix: [],
                },
            ),
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 0,
            log2_max_pic_order_cnt_lsb_minus4: Some(
                2,
            ),
            pic_order_cnt_type1: None,
            max_num_ref_frames: 4,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 29,
            pic_height_in_map_units_minus1: 53,
            mb_adaptive_frame_field_flag: None,
            direct_8x8_inference_flag: true,
            frame_crop_info: Some(
                FrameCropInfo {
                    frame_crop_left_offset: 0,
                    frame_crop_right_offset: 0,
                    frame_crop_top_offset: 0,
                    frame_crop_bottom_offset: 6,
                },
            ),
            sample_aspect_ratio: None,
            overscan_appropriate_flag: None,
            color_config: Some(
                ColorConfig {
                    video_format: VideoFormat::Unspecified,
                    video_full_range_flag: false,
                    color_primaries: 1,
                    transfer_characteristics: 1,
                    matrix_coefficients: 1,
                },
            ),
            chroma_sample_loc: None,
            timing_info: Some(
                TimingInfo {
                    num_units_in_tick: 1,
                    time_scale: 60,
                },
            ),
        }
        ");
    }

    #[test]
    fn test_config_build() {
        // these may not be the same size due to the natural reduction from the SPS parsing.
        // in specific, the sps size function may return a lower size than the original bitstring.
        // reduction will occur from rebuilding the sps and from rebuilding the sps_ext.
        let data = Bytes::from(b"\x01d\0\x1f\xff\xe1\0\x19\x67\x64\x00\x1F\xAC\xD9\x41\xE0\x6D\xF9\xE6\xA0\x20\x20\x28\x00\x00\x03\x00\x08\x00\x00\x03\x01\xE0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0".to_vec());

        let config = AVCDecoderConfigurationRecord::parse(&mut io::Cursor::new(data.clone())).unwrap();

        assert_eq!(config.size(), data.len() as u64);

        let mut buf = Vec::new();
        config.build(&mut buf).unwrap();

        assert_eq!(buf, data.to_vec());
    }

    #[test]
    fn test_no_ext_cfg_for_profiles_66_77_88() {
        let data = Bytes::from(b"\x01B\x00\x1F\xFF\xE1\x00\x1Dgd\x00\x1F\xAC\xD9A\xE0m\xF9\xE6\xA0  (\x00\x00\x03\x00\x08\x00\x00\x03\x01\xE0x\xC1\x8C\xB0\x01\x00\x06h\xEB\xE3\xCB\"\xC0\xFD\xF8\xF8\x00".to_vec());
        let config = AVCDecoderConfigurationRecord::parse(&mut io::Cursor::new(data)).unwrap();

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
            sps: vec![
                Sps::parse(b"\x67\x64\x00\x1F\xAC\xD9\x41\xE0\x6D\xF9\xE6\xA0\x20\x20\x28\x00\x00\x00\x08\x00\x00\x01\xE0")
                    .unwrap(),
            ],
            pps: vec![Bytes::from_static(b"ppsdata")],
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
                Sps {
                    emu_bytes: 0,
                    forbidden_zero_bit: false,
                    nal_ref_idc: 3,
                    nal_unit_type: NALUnitType::SPS,
                    profile_idc: 100,
                    constraint_set0_flag: false,
                    constraint_set1_flag: false,
                    constraint_set2_flag: false,
                    constraint_set3_flag: false,
                    constraint_set4_flag: false,
                    constraint_set5_flag: false,
                    level_idc: 31,
                    seq_parameter_set_id: 0,
                    ext: Some(
                        SpsExtended {
                            chroma_format_idc: 1,
                            separate_color_plane_flag: false,
                            bit_depth_luma_minus8: 0,
                            bit_depth_chroma_minus8: 0,
                            qpprime_y_zero_transform_bypass_flag: false,
                            scaling_matrix: [],
                        },
                    ),
                    log2_max_frame_num_minus4: 0,
                    pic_order_cnt_type: 0,
                    log2_max_pic_order_cnt_lsb_minus4: Some(
                        2,
                    ),
                    pic_order_cnt_type1: None,
                    max_num_ref_frames: 4,
                    gaps_in_frame_num_value_allowed_flag: false,
                    pic_width_in_mbs_minus1: 29,
                    pic_height_in_map_units_minus1: 53,
                    mb_adaptive_frame_field_flag: None,
                    direct_8x8_inference_flag: true,
                    frame_crop_info: Some(
                        FrameCropInfo {
                            frame_crop_left_offset: 0,
                            frame_crop_right_offset: 0,
                            frame_crop_top_offset: 0,
                            frame_crop_bottom_offset: 6,
                        },
                    ),
                    sample_aspect_ratio: None,
                    overscan_appropriate_flag: None,
                    color_config: Some(
                        ColorConfig {
                            video_format: VideoFormat::Unspecified,
                            video_full_range_flag: false,
                            color_primaries: 1,
                            transfer_characteristics: 1,
                            matrix_coefficients: 1,
                        },
                    ),
                    chroma_sample_loc: None,
                    timing_info: Some(
                        TimingInfo {
                            num_units_in_tick: 1,
                            time_scale: 60,
                        },
                    ),
                },
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
            sps: vec![
                Sps::parse(b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0").unwrap(),
            ],
            pps: vec![Bytes::from_static(b"ppsdata")],
            extended_config: Some(extended_config),
        };

        let mut buf = Vec::new();
        config.build(&mut buf).unwrap();

        let parsed = AVCDecoderConfigurationRecord::parse(&mut io::Cursor::new(buf.into())).unwrap();
        assert_eq!(parsed.extended_config.unwrap().sequence_parameter_set_ext.len(), 1);
        insta::assert_debug_snapshot!(config, @r#"
        AVCDecoderConfigurationRecord {
            configuration_version: 1,
            profile_indication: 100,
            profile_compatibility: 0,
            level_indication: 31,
            length_size_minus_one: 3,
            sps: [
                Sps {
                    emu_bytes: 2,
                    forbidden_zero_bit: false,
                    nal_ref_idc: 3,
                    nal_unit_type: NALUnitType::SPS,
                    profile_idc: 100,
                    constraint_set0_flag: false,
                    constraint_set1_flag: false,
                    constraint_set2_flag: false,
                    constraint_set3_flag: false,
                    constraint_set4_flag: false,
                    constraint_set5_flag: false,
                    level_idc: 31,
                    seq_parameter_set_id: 0,
                    ext: Some(
                        SpsExtended {
                            chroma_format_idc: 1,
                            separate_color_plane_flag: false,
                            bit_depth_luma_minus8: 0,
                            bit_depth_chroma_minus8: 0,
                            qpprime_y_zero_transform_bypass_flag: false,
                            scaling_matrix: [],
                        },
                    ),
                    log2_max_frame_num_minus4: 0,
                    pic_order_cnt_type: 0,
                    log2_max_pic_order_cnt_lsb_minus4: Some(
                        2,
                    ),
                    pic_order_cnt_type1: None,
                    max_num_ref_frames: 4,
                    gaps_in_frame_num_value_allowed_flag: false,
                    pic_width_in_mbs_minus1: 29,
                    pic_height_in_map_units_minus1: 53,
                    mb_adaptive_frame_field_flag: None,
                    direct_8x8_inference_flag: true,
                    frame_crop_info: Some(
                        FrameCropInfo {
                            frame_crop_left_offset: 0,
                            frame_crop_right_offset: 0,
                            frame_crop_top_offset: 0,
                            frame_crop_bottom_offset: 6,
                        },
                    ),
                    sample_aspect_ratio: None,
                    overscan_appropriate_flag: None,
                    color_config: Some(
                        ColorConfig {
                            video_format: VideoFormat::Unspecified,
                            video_full_range_flag: false,
                            color_primaries: 1,
                            transfer_characteristics: 1,
                            matrix_coefficients: 1,
                        },
                    ),
                    chroma_sample_loc: None,
                    timing_info: Some(
                        TimingInfo {
                            num_units_in_tick: 1,
                            time_scale: 60,
                        },
                    ),
                },
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
