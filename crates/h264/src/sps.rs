use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

#[derive(Debug, Clone, PartialEq)]
/// The Sequence Parameter Set.
/// ISO/IEC-14496-10-2022 - 7.3.2
pub struct Sps {
    /// Comprised of 6 `constraint_setn_flag`s where `n` ranges from [0, 5]
    /// (ex: `constraint_set0_flag`, `constraint_set1_flag`, etc.) followed
    /// by 2 reserved zero bits. Each flag is a singular unsigned bit.
    ///
    /// `constraint_set0_flag`: `1` if it abides by the constraints in A.2.1, `0` if unsure or otherwise.
    ///
    /// `constraint_set1_flag`: `1` if it abides by the constraints in A.2.2, `0` if unsure or otherwise.
    ///
    /// `constraint_set2_flag`: `1` if it abides by the constraints in A.2.3, `0` if unsure or otherwise.
    ///
    /// `constraint_set3_flag`:
    /// ```text
    ///     if (profile_idc == 66, 77, or 88) AND (level_idc == 11):
    ///         1 if it abides by the constraints in Annex A for level 1b
    ///         0 if it abides by the constraints in Annex A for level 1.1
    ///     elif profile_idc == 100 or 110:
    ///         1 if it abides by the constraints for the "High 10 Intra profile"
    ///         0 if unsure or otherwise
    ///     elif profile_idc == 122:
    ///         1 if it abides by the constraints in Annex A for the "High 4:2:2 Intra profile"
    ///         0 if unsure or otherwise
    ///     elif profile_idc == 44:
    ///         1 by default
    ///         0 is not possible.
    ///     elif profile_idc == 244:
    ///         1 if it abides by the constraints in Annex A for the "High 4:4:4 Intra profile"
    ///         0 if unsure or otherwise
    ///     else:
    ///         1 is reserved for future use
    ///         0 otherwise
    /// ```
    /// `constraint_set4_flag`:
    /// ```text
    ///     if (profile_idc == 77, 88, 100, or 110):
    ///         1 if frame_mbs_only_flag == 1
    ///         0 if unsure or otherwise
    ///     elif (profile_idc == 118, 128, or 134):
    ///         1 if it abides by the constraints in G.6.1.1
    ///         0 if unsure or otherwise
    ///     else:
    ///         1 is reserved for future use
    ///         0 otherwise
    /// ```
    /// `constraint_set5_flag`:
    /// ```text
    ///     if (profile_idc == 77, 88, or 100):
    ///         1 if there are no B slice types
    ///         0 if unsure or otherwise
    ///     elif profile_idc == 118:
    ///         1 if it abides by the constraints in G.6.1.2
    ///         0 if unsure or otherwise
    ///     else:
    ///         1 is reserved for future use
    ///         0 otherwise
    /// ```
    /// The last two bits in the u8 are set to be 0. They are reserved for future use.
    pub profile_idc: u8,
    /// The level_idc as a u8.
    pub level_idc: u8,
    /// An optional `SpsExtended`. Refer to the SpsExtended struct for more info.
    pub ext: Option<SpsExtended>,
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
        // Returns an error if there aren't enough bytes.
        if data.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Insufficient data: SPS must be at least 4 bytes long",
            ));
        }

        let mut vec = Vec::with_capacity(data.len());

        // We need to remove the emulation prevention byte
        // This is BARELY documented in the spec, but it's there.
        // ISO/IEC-14496-10-2022 - 3.1.48
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
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Forbidden zero bit is set"));
        }

        bit_reader.seek_bits(2)?; // nal_ref_idc

        let nal_unit_type = bit_reader.read_bits(5)?;
        if nal_unit_type != 7 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "NAL unit type is not SPS"));
        }

        let profile_idc = bit_reader.read_u8()?;
        bit_reader.seek_bits(
            1 // constraint_set0_flag
            + 1 // constraint_set1_flag
            + 1 // constraint_set2_flag
            + 1 // constraint_set3_flag
            + 4, // reserved_zero_4bits
        )?;

        let level_idc = bit_reader.read_u8()?;
        bit_reader.read_exp_golomb()?; // seq_parameter_set_id

        let sps_ext = match profile_idc {
            100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135 => {
                Some(SpsExtended::parse(&mut bit_reader)?)
            }
            _ => None,
        };

        bit_reader.read_exp_golomb()?; // log2_max_frame_num_minus4
        let pic_order_cnt_type = bit_reader.read_exp_golomb()?;
        if pic_order_cnt_type == 0 {
            bit_reader.read_exp_golomb()?; // log2_max_pic_order_cnt_lsb_minus4
        } else if pic_order_cnt_type == 1 {
            bit_reader.seek_bits(1)?; // delta_pic_order_always_zero_flag
            bit_reader.read_signed_exp_golomb()?; // offset_for_non_ref_pic
            bit_reader.read_signed_exp_golomb()?; // offset_for_top_to_bottom_field
            let num_ref_frames_in_pic_order_cnt_cycle = bit_reader.read_exp_golomb()?;
            for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                bit_reader.read_signed_exp_golomb()?; // offset_for_ref_frame
            }
        }

        bit_reader.read_exp_golomb()?; // max_num_ref_frames
        bit_reader.read_bit()?; // gaps_in_frame_num_value_allowed_flag
        let pic_width_in_mbs_minus1 = bit_reader.read_exp_golomb()?; // pic_width_in_mbs_minus1
        let pic_height_in_map_units_minus1 = bit_reader.read_exp_golomb()?; // pic_height_in_map_units_minus1
        let frame_mbs_only_flag = bit_reader.read_bit()?;
        if !frame_mbs_only_flag {
            bit_reader.seek_bits(1)?; // mb_adaptive_frame_field_flag
        }

        bit_reader.seek_bits(1)?; // direct_8x8_inference_flag

        let mut frame_crop_left_offset = 0;
        let mut frame_crop_right_offset = 0;
        let mut frame_crop_top_offset = 0;
        let mut frame_crop_bottom_offset = 0;

        if bit_reader.read_bit()? {
            // frame_cropping_flag
            frame_crop_left_offset = bit_reader.read_exp_golomb()?; // frame_crop_left_offset
            frame_crop_right_offset = bit_reader.read_exp_golomb()?; // frame_crop_right_offset
            frame_crop_top_offset = bit_reader.read_exp_golomb()?; // frame_crop_top_offset
            frame_crop_bottom_offset = bit_reader.read_exp_golomb()?; // frame_crop_bottom_offset
        }

        let width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2;
        let height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
            - frame_crop_bottom_offset * 2
            - frame_crop_top_offset * 2;

        let mut frame_rate = 0.0;

        let vui_parameters_present_flag = bit_reader.read_bit()?;

        let mut color_config = None;

        if vui_parameters_present_flag {
            // We do want to read the VUI parameters to get the frame rate.

            // aspect_ratio_info_present_flag
            if bit_reader.read_bit()? {
                let aspect_ratio_idc = bit_reader.read_u8()?;
                if aspect_ratio_idc == 255 {
                    bit_reader.seek_bits(16)?; // sar_width
                    bit_reader.seek_bits(16)?; // sar_height
                }
            }

            // overscan_info_present_flag
            if bit_reader.read_bit()? {
                bit_reader.seek_bits(1)?; // overscan_appropriate_flag
            }

            // video_signal_type_present_flag
            if bit_reader.read_bit()? {
                bit_reader.seek_bits(3)?; // video_format
                let full_range = bit_reader.read_bit()?; // video_full_range_flag

                let color_primaries;
                let transfer_characteristics;
                let matrix_coefficients;

                if bit_reader.read_bit()? {
                    // colour_description_present_flag
                    color_primaries = bit_reader.read_u8()?; // colour_primaries
                    transfer_characteristics = bit_reader.read_u8()?; // transfer_characteristics
                    matrix_coefficients = bit_reader.read_u8()?; // matrix_coefficients
                } else {
                    color_primaries = 2; // UNSPECIFIED
                    transfer_characteristics = 2; // UNSPECIFIED
                    matrix_coefficients = 2; // UNSPECIFIED
                }

                color_config = Some(ColorConfig {
                    full_range,
                    color_primaries,
                    transfer_characteristics,
                    matrix_coefficients,
                });
            }

            // chroma_loc_info_present_flag
            if bit_reader.read_bit()? {
                bit_reader.read_exp_golomb()?; // chroma_sample_loc_type_top_field
                bit_reader.read_exp_golomb()?; // chroma_sample_loc_type_bottom_field
            }

            // timing_info_present_flag
            if bit_reader.read_bit()? {
                let num_units_in_tick = bit_reader.read_u32::<BigEndian>()?;
                let time_scale = bit_reader.read_u32::<BigEndian>()?;

                if num_units_in_tick == 0 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "num_units_in_tick cannot be zero"));
                }

                frame_rate = time_scale as f64 / (2.0 * num_units_in_tick as f64);
            }
        }

        Ok(Sps {
            profile_idc,
            level_idc,
            ext: sps_ext,
            width,
            height,
            frame_rate,
            color_config,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// The Sequence Parameter Set extension.
/// ISO/IEC-14496-10-2022 - 7.3.2
pub struct SpsExtended {
    /// The `chroma_format_idc` as a u64.
    pub chroma_format_idc: u64, // ue(v)
    /// The `bit_depth_luma_minus8` as a u64.
    pub bit_depth_luma_minus8: u64, // ue(v)
    /// The `bit_depth_chroma_minus8` as a u64.
    pub bit_depth_chroma_minus8: u64, // ue(v)
}

impl SpsExtended {
    /// Parses an extended SPS from a bitstream.
    /// Returns an `SpsExtended` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let chroma_format_idc = reader.read_exp_golomb()?;
        if chroma_format_idc == 3 {
            reader.read_bit()?;
        }

        let bit_depth_luma_minus8 = reader.read_exp_golomb()?;
        let bit_depth_chroma_minus8 = reader.read_exp_golomb()?;
        reader.read_bit()?; // qpprime_y_zero_transform_bypass_flag

        if reader.read_bit()? {
            // seq_scaling_matrix_present_flag
            // We need to read the scaling matrices here, but we don't need them
            // for decoding, so we just skip them.
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for i in 0..count {
                if reader.read_bit()? {
                    let size = if i < 6 { 16 } else { 64 };
                    let mut next_scale = 8;
                    for _ in 0..size {
                        let delta_scale = reader.read_signed_exp_golomb()?;
                        next_scale = (next_scale + delta_scale + 256) % 256;
                        if next_scale == 0 {
                            break;
                        }
                    }
                }
            }
        }

        Ok(SpsExtended {
            chroma_format_idc,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use bytes::Bytes;
    use scuffle_bytes_util::BitReader;

    use crate::sps::{ColorConfig, Sps, SpsExtended};

    #[test]
    fn test_parse_sps_insufficient_bytes_() {
        let sps = Bytes::from(vec![0xFF]);
        let result = Sps::parse(sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Insufficient data: SPS must be at least 4 bytes long");
    }

    #[test]
    fn test_parse_sps_set_forbidden_bit() {
        let sps = Bytes::from(vec![
            0xFF, // forbidden bit is set
            0xFF, // dummy data
            0xFF, 0xFF,
        ]);
        let result = Sps::parse(sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Forbidden zero bit is set");
    }

    #[test]
    fn test_parse_sps_invalid_nal() {
        let data = Bytes::from(vec![
            // NAL Header: forbidden_zero_bit (0) + nal_ref_idc (11) + nal_unit_type (5 = non-SPS)
            0x65, // 01100101 -> nal_unit_type = 5 (not 7, so invalid)
            0xFF, // dummy data
            0xFF, 0xFF,
        ]);
        let result = Sps::parse(data);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "NAL unit type is not SPS");
    }

    #[test]
    fn test_parse_sps_4k_60fps() {
        let sps = Bytes::from(vec![
            // NAL Header: forbidden_zero_bit (0), nal_ref_idc (11), nal_unit_type (7 = SPS)
            0x67, // Profile IDC (High Profile = 100)
            0x64, // Constraint flags and reserved bits
            0x00, // Level IDC (51)
            0x33, // Sequence Parameter Set ID, log2_max_frame_num_minus4, pic_order_cnt_type
            0xAC, 0xCA, 0x50, 0x0F, // Reserved bits and emulation prevention
            0x00, 0x10, 0xFB, 0x01, 0x10, // Frame dimensions: width = 3840, height = 2160
            0x00, 0x00, 0x03, 0x00, 0x10, 0x00, 0x00, 0x07, 0x88, 0xF1, 0x83, 0x19, 0x60,
        ]);

        let sps = Sps::parse(sps).unwrap();

        assert_eq!(sps.profile_idc, 100);
        assert_eq!(sps.level_idc, 51);
        assert_eq!(
            sps.ext,
            Some(SpsExtended {
                chroma_format_idc: 1,
                bit_depth_luma_minus8: 0,
                bit_depth_chroma_minus8: 0,
            })
        );
        assert_eq!(sps.width, 3840);
        assert_eq!(sps.height, 2160);
        assert_eq!(sps.frame_rate, 60.0);
        assert_eq!(sps.color_config, None);
    }

    #[test]
    fn test_parse_sps_480p_0fps() {
        let sps = Bytes::from(vec![
            // NAL Header: nal_unit_type (7 = SPS)
            0x67, // Profile IDC (Baseline = 66)
            0x42, // Constraint flags and reserved bits
            0xC0, // Level IDC (31)
            0x1F, // Sequence Parameter Set ID, log2_max_frame_num_minus4, pic_order_cnt_type
            0x8C, 0x8D, 0x40, 0x50, // Frame dimensions: width = 640, height = 480
            0x1E, 0x90, 0x0F, 0x08, 0x84, 0x6A,
        ]);

        let sps = Sps::parse(sps).unwrap();

        assert_eq!(sps.profile_idc, 66);
        assert_eq!(sps.level_idc, 31);
        assert_eq!(sps.ext, None);
        assert_eq!(sps.width, 640);
        assert_eq!(sps.height, 480);
        assert_eq!(sps.frame_rate, 0.0);
        assert_eq!(sps.color_config, None);
    }

    #[test]
    fn test_parse_sps_1080p_60fps_with_color_config() {
        let sps = Bytes::from(vec![
            // NAL Header: nal_unit_type (7 = SPS)
            0x67, // Profile IDC (High Profile = 100)
            0x64, // Constraint flags and reserved bits
            0x00, // Level IDC (42)
            0x2A, // Sequence Parameter Set ID, log2_max_frame_num_minus4, pic_order_cnt_type
            0xAC, 0xB2, 0x00, 0xF0, // Color configuration present
            0x04, 0x4F, 0xCB, 0x80, 0xB5, 0x01, 0x01, 0x01, 0x40,
            // Emulation prevention bytes removal and frame rate
            0x00, 0x00, 0x03, 0x00, 0x40, 0x00, 0x00, 0x1E, 0x23, 0xC6, 0x0C, 0x92,
        ]);

        let sps = Sps::parse(sps).unwrap();

        assert_eq!(sps.profile_idc, 100);
        assert_eq!(sps.level_idc, 42);
        assert_eq!(
            sps.ext,
            Some(SpsExtended {
                chroma_format_idc: 1,
                bit_depth_luma_minus8: 0,
                bit_depth_chroma_minus8: 0,
            })
        );
        assert_eq!(sps.width, 1920);
        assert_eq!(sps.height, 1080);
        assert_eq!(sps.frame_rate, 60.0);
        assert_eq!(
            sps.color_config,
            Some(ColorConfig {
                full_range: false,
                matrix_coefficients: 1,
                color_primaries: 1,
                transfer_characteristics: 1,
            })
        );
    }

    #[test]
    fn test_parse_sps_pic_order_cnt_type_set() {
        let sps = bytes::Bytes::from(vec![
            // NAL header, profile (66), constraint flags + reserved bits, level idc (31)
            0x67, 0x42, 0xC0, 0x1F, 0xD3, 0x58, // sps_id (0), log2_max_frame_num_minus4 (0)
            0x14, // pic_order_cnt_type (1)
            0x07, // delta_pic_order_always_zero_flag (0) and offset_for_non_ref_pic (0)
            // offset_for_top_to_bottom_field (0) and num_ref_frames... (1) and offset_for_ref_frame (0)
            0xB0,
            // max_num_ref_frames (0) and gaps_in_frame_num_value_allowed_flag (0) and begins pic_width_in_mbs_minus1 (39)
            0x1E, 0x90, // pic_width_in_mbs_minus1 encoding (39, so width = 40 * 16 = 640)
            0x0F, // pic_height_in_map_units_minus1 = 29 (so height = 30 * 16 = 480)
            0x08, // frame_mbs_only_flag = 1
            0x84, // direct_8x8_inference_flag = 1; frame_cropping_flag = 0
            0x6A, // vui_parameters_present_flag = 0; end of SPS data
        ]);

        let sps = crate::sps::Sps::parse(sps).unwrap();

        assert_eq!(sps.profile_idc, 66);
        assert_eq!(sps.level_idc, 31);
        assert_eq!(sps.ext, None);
        assert_eq!(sps.width, 640);
        assert_eq!(sps.height, 480);
        assert_eq!(sps.frame_rate, 0.0);
        assert_eq!(sps.color_config, None);
    }

    #[test]
    fn test_parse_sps_vui_and_interlaced() {
        let sps = bytes::Bytes::from(vec![
            // NAL header, profile idc = 66, constraint flags and reserved bits, level idc = 31
            0x67, 0x42, 0x00, 0x1F, 0xF8, // first bits of pic_width_in_mbs_minus1
            0x14, // next 8 bits of pic_width_in_mbs_minus1
            // remainder of pic_width_in_mbs_minus1 + first 7 bits of pic_height_in_map_units_minus1
            0x07,
            // last bits of pic_height_in_map_units_minus1 + flags (frame_mbs_only_flag, etc.) + VUI start bits
            0x8B, 0xFF, // aspect_ratio_idc = 255
            0x01, 0x23, // sar_width high byte (0x0123)
            0x04, 0x56, // sar_height high byte (0x0456)
            0xA0, // overscan and video signal type flags
            0xE0, // chroma loc info and timing flag (plus padding)
        ]);
        let result = Sps::parse(sps).unwrap();

        assert_eq!(result.width, 640);
        assert_eq!(result.height, 960);
        assert_eq!(result.frame_rate, 0.0);
        assert_eq!(
            result.color_config,
            Some(crate::sps::ColorConfig {
                full_range: false,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
            })
        );
    }

    #[test]
    fn test_parse_sps_ext_chroma_format_3() {
        let sps = Bytes::from_static(&[
            0x67, 0x64, 0x00, 0x1F, // NAL/profile/constraints/level
            0x91, 0x9E, 0xF0, // chroma_format_idc=3
        ]);

        let result = Sps::parse(sps).expect("Failed to parse SPS");
        assert_eq!(result.profile_idc, 100);

        let ext = result.ext.expect("Expected SpsExtended, got None");
        assert_eq!(ext.chroma_format_idc, 3);

        assert_eq!(ext.bit_depth_luma_minus8, 0);
        assert_eq!(ext.bit_depth_chroma_minus8, 0);
    }

    #[test]
    fn test_parse_sps_ext_scaling_matrix() {
        let data = Bytes::from(vec![0x23, 0x7F, 0xFF, 0xE0, 0x00]);
        let mut reader = BitReader::new_from_slice(data);
        let ext = SpsExtended::parse(&mut reader).unwrap();

        assert_eq!(ext.chroma_format_idc, 3);
        assert_eq!(ext.bit_depth_luma_minus8, 0);
        assert_eq!(ext.bit_depth_chroma_minus8, 0);
    }

    #[test]
    fn test_parse_sps_ext_break() {
        let data = Bytes::from(vec![0x5B, 0x08, 0x80]);
        let mut reader = BitReader::new_from_slice(data);
        let ext = SpsExtended::parse(&mut reader).unwrap();

        assert_eq!(ext.chroma_format_idc, 1);
        assert_eq!(ext.bit_depth_luma_minus8, 0);
        assert_eq!(ext.bit_depth_chroma_minus8, 0);
    }
}
