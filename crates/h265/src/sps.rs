use std::io;

use byteorder::ReadBytesExt;
use bytes::Bytes;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

#[derive(Debug, Clone, PartialEq)]
/// The Sequence Parameter Set.
/// ISO/IEC-14496-10-2022 - 7.3.2
pub struct Sps {
    /// The width as a u64.
    pub width: u64,
    /// The height as a u64.
    pub height: u64,
    /// The framerate as a f64.
    pub frame_rate: f64,
    /// An optional `ColorConfig`. Refer to the ColorConfig struct for more info.
    pub color_config: Option<ColorConfig>,
}

#[derive(Debug, Clone, PartialEq)]
/// The color config for SPS.
pub struct ColorConfig {
    /// The `video_full_range_flag` as a bool.
    pub full_range: bool,
    /// The `colour_primaries` bits as a u8.
    pub color_primaries: u8,
    /// The `transfer_characteristics` bits as a u8.
    pub transfer_characteristics: u8,
    /// The `matrix_coefficients` bits as a u8.
    pub matrix_coefficients: u8,
}

impl Sps {
    /// Parses an SPS from the input bytes.
    /// Returns an `Sps` struct.
    pub fn parse(data: Bytes) -> io::Result<Self> {
        let mut vec = Vec::with_capacity(data.len());

        // ISO/IEC-23008-2-2022 - 7.3.1.1
        let mut i = 0;
        while i < data.len() {
            if i + 2 < data.len() && data[i] == 0x00 && data[i + 1] == 0x00 && data[i + 2] == 0x03 {
                vec.push(0x00);
                vec.push(0x00);
                i += 3; // Skip the emulation prevention byte.
            } else {
                vec.push(data[i]);
                i += 1;
            }
        }

        let mut bit_reader = BitReader::new_from_slice(vec);

        let forbidden_zero_bit = bit_reader.read_bit()?;
        if forbidden_zero_bit {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "forbidden_zero_bit is not zero"));
        }

        let nalu_type = bit_reader.read_bits(6)?;
        if nalu_type != 33 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "nalu_type is not 33", // SPS
            ));
        }

        bit_reader.seek_bits(
            6 // nuh_layer_id
            + 3 // nuh_temporal_id_plus1
            + 4, // sps_video_parameter_set_id
        )?;

        let sps_max_sub_layers_minus1 = bit_reader.read_bits(3)?;
        bit_reader.seek_bits(1)?; // sps_temporal_id_nesting_flag
        {
            bit_reader.seek_bits(
                2 // general_profile_space
                + 1 // general_tier_flag
                + 5 // general_profile_idc
                + 32 // general_profile_compatibility_flag
                + 1 // general_progressive_source_flag
                + 1 // general_interlaced_source_flag
                + 1 // general_non_packed_constraint_flag
                + 1 // general_frame_only_constraint_flag
                + 43 // general_reserved_zero_43bits
                + 1 // general_reserved_zero_bit
                + 8, // general_level_idc
            )?;

            let mut sub_layer_level_present_flags = vec![false; sps_max_sub_layers_minus1 as usize];
            for v in sub_layer_level_present_flags.iter_mut() {
                bit_reader.seek_bits(1)?; // sub_layer_profile_present_flag
                *v = bit_reader.read_bit()?; // sub_layer_level_present_flag
            }

            if sps_max_sub_layers_minus1 > 0 && sps_max_sub_layers_minus1 < 8 {
                bit_reader.seek_bits(2 * (8 - sps_max_sub_layers_minus1 as i64))?;
                // reserved_zero_2bits
            }

            for v in sub_layer_level_present_flags.drain(..) {
                bit_reader.seek_bits(
                    2 // sub_layer_profile_space
                    + 1 // sub_layer_tier_flag
                    + 5 // sub_layer_profile_idc
                    + 32 // sub_layer_profile_compatibility_flag[32]
                    + 1 // sub_layer_progressive_source_flag
                    + 1 // sub_layer_interlaced_source_flag
                    + 1 // sub_layer_non_packed_constraint_flag
                    + 1 // sub_layer_frame_only_constraint_flag
                    + 43 // sub_layer_reserved_zero_44bits
                    + 1, // sub_layer_reserved_zero_bit
                )?;
                if v {
                    bit_reader.seek_bits(8)?; // sub_layer_level_idc
                }
            }
        }

        bit_reader.read_exp_golomb()?; // sps_seq_parameter_set_id
        let chroma_format_idc = bit_reader.read_exp_golomb()?;
        if chroma_format_idc == 3 {
            bit_reader.read_bit()?;
        }
        let pic_width_in_luma_samples = bit_reader.read_exp_golomb()?;
        let pic_height_in_luma_samples = bit_reader.read_exp_golomb()?;
        let conformance_window_flag = bit_reader.read_bit()?;

        let conf_win_left_offset;
        let conf_win_right_offset;
        let conf_win_top_offset;
        let conf_win_bottom_offset;

        if conformance_window_flag {
            conf_win_left_offset = bit_reader.read_exp_golomb()?;
            conf_win_right_offset = bit_reader.read_exp_golomb()?;
            conf_win_top_offset = bit_reader.read_exp_golomb()?;
            conf_win_bottom_offset = bit_reader.read_exp_golomb()?;
        } else {
            conf_win_left_offset = 0;
            conf_win_right_offset = 0;
            conf_win_top_offset = 0;
            conf_win_bottom_offset = 0;
        }

        let (sub_width_c, sub_height_c) = match chroma_format_idc {
            0 => (1, 1),
            1 => (2, 2),
            2 => (2, 1),
            3 => (1, 1),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "chroma_format_idc is not 0-3")),
        };

        let width = pic_width_in_luma_samples - sub_width_c * (conf_win_left_offset + conf_win_right_offset);
        let height = pic_height_in_luma_samples - sub_height_c * (conf_win_top_offset + conf_win_bottom_offset);

        bit_reader.read_exp_golomb()?; // bit_depth_luma_minus8
        bit_reader.read_exp_golomb()?; // bit_depth_chroma_minus8
        bit_reader.read_exp_golomb()?; // log2_max_pic_order_cnt_lsb_minus4
        let sps_sub_layer_ordering_info_present_flag = bit_reader.read_bit()?;

        if sps_sub_layer_ordering_info_present_flag {
            for _ in 0..=sps_max_sub_layers_minus1 {
                bit_reader.read_exp_golomb()?; // sps_max_dec_pic_buffering_minus1
                bit_reader.read_exp_golomb()?; // sps_max_num_reorder_pics
                bit_reader.read_exp_golomb()?; // sps_max_latency_increase_plus1
            }
        };

        bit_reader.read_exp_golomb()?; // log2_min_luma_coding_block_size_minus3
        bit_reader.read_exp_golomb()?; // log2_diff_max_min_luma_coding_block_size
        bit_reader.read_exp_golomb()?; // log2_min_transform_block_size_minus2
        bit_reader.read_exp_golomb()?; // log2_diff_max_min_transform_block_size
        bit_reader.read_exp_golomb()?; // max_transform_hierarchy_depth_inter
        bit_reader.read_exp_golomb()?; // max_transform_hierarchy_depth_intra

        let scaling_list_enabled_flag = bit_reader.read_bit()?;
        if scaling_list_enabled_flag {
            let sps_scaling_list_data_present_flag = bit_reader.read_bit()?;
            if sps_scaling_list_data_present_flag {
                for size_id in 0..4 {
                    let mut matrix_id = 0;
                    while matrix_id < 6 {
                        let scaling_list_pred_mode_flag = bit_reader.read_bit()?;
                        if !scaling_list_pred_mode_flag {
                            bit_reader.read_exp_golomb()?; // scaling_list_pred_matrix_id_delta
                        } else {
                            let coef_num = 64.min(1 << (4 + (size_id << 1)));
                            let mut next_coef = 8;
                            if size_id > 1 {
                                let scaling_list_dc_coef_minus8 = bit_reader.read_signed_exp_golomb()?;
                                next_coef = 8 + scaling_list_dc_coef_minus8;
                            }
                            for _ in 0..coef_num {
                                let scaling_list_delta_coef = bit_reader.read_signed_exp_golomb()?;
                                next_coef = (next_coef + scaling_list_delta_coef + 256) % 256;
                            }
                        }
                        matrix_id += if size_id == 3 { 3 } else { 1 };
                    }
                }
            }
        }

        bit_reader.seek_bits(1)?; // amp_enabled_flag
        bit_reader.seek_bits(1)?; // sample_adaptive_offset_enabled_flag

        if bit_reader.read_bit()? {
            // pcm_enabled_flag
            bit_reader.seek_bits(4)?; // pcm_sample_bit_depth_luma_minus1
            bit_reader.seek_bits(4)?; // pcm_sample_bit_depth_chroma_minus1
            bit_reader.read_exp_golomb()?; // log2_min_pcm_luma_coding_block_size_minus3
            bit_reader.read_exp_golomb()?; // log2_diff_max_min_pcm_luma_coding_block_size
            bit_reader.seek_bits(1)?; // pcm_loop_filter_disabled_flag
        }

        let num_short_term_ref_pic_sets = bit_reader.read_exp_golomb()?;
        let mut num_delta_pocs = vec![0; num_short_term_ref_pic_sets as usize];
        for st_rps_idx in 0..num_short_term_ref_pic_sets {
            if st_rps_idx != 0 && bit_reader.read_bit()? {
                bit_reader.seek_bits(1)?;
                bit_reader.read_exp_golomb()?; // delta_rps_sign

                num_delta_pocs[st_rps_idx as usize] = 0;

                for _ in 0..num_delta_pocs[(st_rps_idx - 1) as usize] {
                    let used_by_curr_pic_flag = bit_reader.read_bit()?;
                    let use_delta_flag = if !used_by_curr_pic_flag {
                        bit_reader.read_bit()? // use_delta_flag
                    } else {
                        false
                    };

                    if used_by_curr_pic_flag || use_delta_flag {
                        num_delta_pocs[st_rps_idx as usize] += 1;
                    }
                }
            } else {
                let num_negative_pics = bit_reader.read_exp_golomb()?;
                let num_positive_pics = bit_reader.read_exp_golomb()?;

                num_delta_pocs[st_rps_idx as usize] = num_negative_pics + num_positive_pics;
                for _ in 0..num_negative_pics {
                    bit_reader.read_exp_golomb()?; // delta_poc_s0_minus1
                    bit_reader.seek_bits(1)?; // used_by_curr_pic_s0_flag
                }
                for _ in 0..num_positive_pics {
                    bit_reader.read_exp_golomb()?; // delta_poc_s1_minus1
                    bit_reader.seek_bits(1)?; // used_by_curr_pic_s1_flag
                }
            }
        }

        let long_term_ref_pics_present_flag = bit_reader.read_bit()?;
        if long_term_ref_pics_present_flag {
            let num_long_term_ref_pics_sps = bit_reader.read_exp_golomb()?;
            for _ in 0..num_long_term_ref_pics_sps {
                bit_reader.read_exp_golomb()?; // lt_ref_pic_poc_lsb_sps
                bit_reader.seek_bits(1)?; // used_by_curr_pic_lt_sps_flag
            }
        }

        bit_reader.seek_bits(1)?; // sps_temporal_mvp_enabled_flag
        bit_reader.seek_bits(1)?; // strong_intra_smoothing_enabled_flag
        let vui_parameters_present_flag = bit_reader.read_bit()?;

        let mut color_config = None;

        let mut frame_rate = 0.0;
        if vui_parameters_present_flag {
            let aspect_ratio_info_present_flag = bit_reader.read_bit()?;
            if aspect_ratio_info_present_flag {
                let aspect_ratio_idc = bit_reader.read_bits(8)?;
                if aspect_ratio_idc == 255 {
                    bit_reader.seek_bits(16)?; // sar_width
                    bit_reader.seek_bits(16)?; // sar_height
                }
            }

            let overscan_info_present_flag = bit_reader.read_bit()?;
            if overscan_info_present_flag {
                bit_reader.seek_bits(1)?; // overscan_appropriate_flag
            }

            let video_signal_type_present_flag = bit_reader.read_bit()?;
            if video_signal_type_present_flag {
                bit_reader.seek_bits(3)?; // video_format
                let full_range = bit_reader.read_bit()?; // video_full_range_flag
                let color_primaries;
                let transfer_characteristics;
                let matrix_coefficients;

                let colour_description_present_flag = bit_reader.read_bit()?;
                if colour_description_present_flag {
                    color_primaries = bit_reader.read_u8()?; // colour_primaries
                    transfer_characteristics = bit_reader.read_u8()?; // transfer_characteristics
                    matrix_coefficients = bit_reader.read_u8()?; // matrix_coeffs
                } else {
                    color_primaries = 2; // Unspecified
                    transfer_characteristics = 2; // Unspecified
                    matrix_coefficients = 2; // Unspecified
                }

                color_config = Some(ColorConfig {
                    full_range,
                    color_primaries,
                    transfer_characteristics,
                    matrix_coefficients,
                });
            }

            let chroma_loc_info_present_flag = bit_reader.read_bit()?;
            if chroma_loc_info_present_flag {
                bit_reader.read_exp_golomb()?; // chroma_sample_loc_type_top_field
                bit_reader.read_exp_golomb()?; // chroma_sample_loc_type_bottom_field
            }

            bit_reader.seek_bits(1)?;
            bit_reader.seek_bits(1)?;
            bit_reader.seek_bits(1)?;
            let default_display_window_flag = bit_reader.read_bit()?;

            if default_display_window_flag {
                bit_reader.read_exp_golomb()?; // def_disp_win_left_offset
                bit_reader.read_exp_golomb()?; // def_disp_win_right_offset
                bit_reader.read_exp_golomb()?; // def_disp_win_top_offset
                bit_reader.read_exp_golomb()?; // def_disp_win_bottom_offset
            }

            let vui_timing_info_present_flag = bit_reader.read_bit()?;
            if vui_timing_info_present_flag {
                let num_units_in_tick = bit_reader.read_bits(32)?; // vui_num_units_in_tick
                let time_scale = bit_reader.read_bits(32)?; // vui_time_scale

                if num_units_in_tick == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "vui_num_units_in_tick cannot be zero",
                    ));
                }

                frame_rate = time_scale as f64 / num_units_in_tick as f64;
            }
        }

        Ok(Sps {
            width,
            height,
            frame_rate,
            color_config,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use bytes::Bytes;

    use crate::{ColorConfig, Sps};

    #[test]
    fn test_sps_parse() {
        let data = b"B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\x80\x80\x82\0\0\x03\0\x02\0\0\x03\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08".to_vec();

        let sps = Sps::parse(Bytes::from(data.to_vec())).unwrap();
        assert_eq!(
            sps,
            Sps {
                color_config: Some(ColorConfig {
                    full_range: false,
                    color_primaries: 1,
                    matrix_coefficients: 1,
                    transfer_characteristics: 1,
                }),
                frame_rate: 144.0,
                width: 2560,
                height: 1440,
            }
        );
    }

    #[test]
    fn test_parse_sps_with_zero_vui_num_units_in_tick() {
        let sps = Bytes::from(b"B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\0\x80\x82\0\0\x03\0\0\0\0\0\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08".to_vec());
        let sps = Sps::parse(sps);

        match sps {
            Ok(_) => panic!("Expected error for vui_num_units_in_tick = 0, but got Ok"),
            Err(e) => assert_eq!(
                e.kind(),
                std::io::ErrorKind::InvalidData,
                "Expected InvalidData error, got {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_forbidden_zero_bit() {
        // 0x80 = 1000 0000: forbidden_zero_bit (first bit) is 1.
        let data = Bytes::from(vec![0x80]);
        let err = Sps::parse(data).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "forbidden_zero_bit is not zero");
    }

    #[test]
    fn test_invalid_nalu_type() {
        // 0x40 = 0100 0000:
        //   forbidden_zero_bit = 0;
        //   next 6 bits (100000) = 32 ≠ 33.
        let data = Bytes::from(vec![0x40]);
        let err = Sps::parse(data).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "nalu_type is not 33");
    }

    #[test]
    fn test_sub_layer_for_loop() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x40\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \xC0\x16\x88\x07\xC5\xDF\x84\x00"
            .to_vec();
        let data = bytes::Bytes::from(data);
        let result = Sps::parse(data).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            width: 720,
            height: 496,
            frame_rate: 0.0,
            color_config: None,
        }
        ");
    }

    #[test]
    fn test_sub_layer_loop_without_level_idc() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \xC0\x0F\x02\x00\x43\x97\x7E\x10"
            .to_vec();
        let data = bytes::Bytes::from(data);
        let result = Sps::parse(data).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            width: 1920,
            height: 1080,
            frame_rate: 0.0,
            color_config: None,
        }
        ");
    }

    #[test]
    fn test_chroma_format_idc_3() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x40\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x90\x00\xA0\x40\x2D\x2E\xFC\x20"
            .to_vec();
        let data = bytes::Bytes::from(data);
        let result = Sps::parse(data).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            width: 640,
            height: 360,
            frame_rate: 0.0,
            color_config: None,
        }
        ");
    }

    #[test]
    fn test_conformance_window_and_chroma_format_idc_2() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x40\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \xB0\x0A\x48\x0F\x5B\x6D\xF7\xF1\x20\
                    \x00\x80\x00\x00\x00\x01\x00\x00\x00\x0F\
                    \x00"
            .to_vec();
        let data = bytes::Bytes::from(data);
        let result = Sps::parse(data).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            width: 320,
            height: 240,
            frame_rate: 0.0,
            color_config: None,
        }
        ");
    }

    #[test]
    fn test_invalid_chroma_format_idc() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x40\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x97\x00"
            .to_vec();
        let data = bytes::Bytes::from(data);
        let err = Sps::parse(data).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "chroma_format_idc is not 0-3");
    }

    #[test]
    fn test_scaling_list_pred_mode_flag_false() {
        let data = b"\x42\x00\x00\
            \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
            \xA0\x03\xC0\x80\x10\xE5\xDF\xEA\xAA\xAA\xAA\xAA\xA2\x20\x10\x00\x00\x06\x40\x00\x05\xDC\x00"
            .to_vec();

        let data = bytes::Bytes::from(data);
        let result = Sps::parse(data).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            width: 1920,
            height: 1080,
            frame_rate: 240.0,
            color_config: None,
        }
        ");
    }

    #[test]
    fn test_nonzero_st_rps_idx() {
        let data = b"\x42\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                 \xA0\x32\x83\x37\x7E\x0D\x6A\xA0"
            .to_vec();

        let sps = Sps::parse(bytes::Bytes::from(data)).unwrap();

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            width: 100,
            height: 50,
            frame_rate: 0.0,
            color_config: None,
        }
        ");
    }
}
