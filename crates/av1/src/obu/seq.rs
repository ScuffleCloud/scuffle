//! Sequence Header

use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use scuffle_bytes_util::BitReader;

use super::ObuHeader;
use crate::obu::utils::read_uvlc;

/// Sequence Header OBU
///
/// AV1-Spec-2 - 5.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceHeaderObu {
    /// The OBU header that precedes the sequence header
    pub header: ObuHeader,
    /// `seq_profile`
    ///
    /// 3 bits
    pub seq_profile: u8,
    /// `still_picture`
    ///
    /// 1 bit
    pub still_picture: bool,
    /// `reduced_still_picture_header`
    ///
    /// 1 bit
    pub reduced_still_picture_header: bool,
    /// `timing_info` if `reduced_still_picture_header` is 0 and `timing_info_present_flag` is 1
    pub timing_info: Option<TimingInfo>,
    /// `decoder_model_info` if
    /// - `reduced_still_picture_header` is 0
    /// - `timing_info_present_flag` is 1
    /// - `decoder_model_info_present_flag` is 1
    pub decoder_model_info: Option<DecoderModelInfo>,
    /// All operating points
    pub operating_points: Vec<OperatingPoint>,
    /// `max_frame_width_minus_1 + 1`
    pub max_frame_width: u64,
    /// `max_frame_height_minus_1 + 1`
    pub max_frame_height: u64,
    /// The [`FrameIds`] if `reduced_still_picture_header` is 0 and `frame_id_numbers_present_flag` is 1
    pub frame_ids: Option<FrameIds>,
    /// `use_128x128_superblock`
    ///
    /// 1 bit
    pub use_128x128_superblock: bool,
    /// `enable_filter_intra`
    ///
    /// 1 bit
    pub enable_filter_intra: bool,
    /// `enable_intra_edge_filter`
    ///
    /// 1 bit
    pub enable_intra_edge_filter: bool,
    /// `enable_interintra_compound`
    ///
    /// 1 bit
    pub enable_interintra_compound: bool,
    /// `enable_masked_compound`
    ///
    /// 1 bit
    pub enable_masked_compound: bool,
    /// `enable_warped_motion`
    ///
    /// 1 bit
    pub enable_warped_motion: bool,
    /// `enable_dual_filter`
    ///
    /// 1 bit
    pub enable_dual_filter: bool,
    /// `enable_order_hint`
    ///
    /// 1 bit
    pub enable_order_hint: bool,
    /// `enable_jnt_comp`
    ///
    /// 1 bit
    pub enable_jnt_comp: bool,
    /// `enable_ref_frame_mvs`
    ///
    /// 1 bit
    pub enable_ref_frame_mvs: bool,
    /// `seq_force_screen_content_tools`
    pub seq_force_screen_content_tools: u8,
    /// `seq_force_integer_mv`
    pub seq_force_integer_mv: u8,
    /// `OrderHintBits`
    ///
    /// 3 bits
    pub order_hint_bits: u8,
    /// `enable_superres`
    ///
    /// 1 bit
    pub enable_superres: bool,
    /// `enable_cdef`
    ///
    /// 1 bit
    pub enable_cdef: bool,
    /// `enable_restoration`
    ///
    /// 1 bit
    pub enable_restoration: bool,
    /// `color_config()`
    pub color_config: ColorConfig,
    /// `film_grain_params_present`
    pub film_grain_params_present: bool,
}

/// Frame IDs
///
/// Can be part of the [`SequenceHeaderObu`].
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct FrameIds {
    /// `delta_frame_id_length_minus_2 + 2`
    ///
    /// 4 bits
    pub delta_frame_id_length: u8,
    /// `additional_frame_id_length_minus_1 + 1`
    ///
    /// 3 bits
    pub additional_frame_id_length: u8,
}

/// Operating Point
///
/// Part of the [`SequenceHeaderObu`].
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct OperatingPoint {
    /// `operating_point_idc`
    ///
    /// 12 bits
    pub idc: u16,
    /// `seq_level_idx`
    ///
    /// 5 bits
    pub seq_level_idx: u8,
    /// `seq_tier`
    ///
    /// 1 bit
    pub seq_tier: bool,
    /// `operating_parameters_info` if `decoder_model_info_present_flag` is 1
    pub operating_parameters_info: Option<OperatingParametersInfo>,
    /// `initial_display_delay_minus_1 + 1` if `initial_display_delay_present_flag` is 1 and `initial_display_delay_present_for_this_op` is 1
    ///
    /// 4 bits
    pub initial_display_delay: Option<u8>,
}

/// Timing info
///
/// AV1-Spec-2 - 5.5.3
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct TimingInfo {
    /// `num_units_in_display_tick`
    ///
    /// 32 bits
    pub num_units_in_display_tick: u32,
    /// `time_scale`
    ///
    /// 32 bits
    pub time_scale: u32,
    /// `num_ticks_per_picture_minus_1 + 1` if `equal_picture_interval` is 1
    ///
    /// uvlc()
    pub num_ticks_per_picture: Option<u64>,
}

impl TimingInfo {
    /// Parses the timing info from the given reader.
    pub fn parse(bit_reader: &mut BitReader<impl io::Read>) -> io::Result<Self> {
        let num_units_in_display_tick = bit_reader.read_u32::<BigEndian>()?;
        let time_scale = bit_reader.read_u32::<BigEndian>()?;
        let num_ticks_per_picture = if bit_reader.read_bit()? {
            Some(read_uvlc(bit_reader)? + 1)
        } else {
            None
        };
        Ok(Self {
            num_units_in_display_tick,
            time_scale,
            num_ticks_per_picture,
        })
    }
}

/// Decoder model info
///
/// AV1-Spec-2 - 5.5.4
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct DecoderModelInfo {
    /// `buffer_delay_length_minus_1 + 1`
    ///
    /// 5 bits
    pub buffer_delay_length: u8,
    /// `num_units_in_decoding_tick`
    ///
    /// 32 bits
    pub num_units_in_decoding_tick: u32,
    /// `buffer_removal_time_length_minus_1 + 1`
    ///
    /// 5 bits
    pub buffer_removal_time_length: u8,
    /// `frame_presentation_time_length_minus_1 + 1`
    ///
    /// 5 bits
    pub frame_presentation_time_length: u8,
}

impl DecoderModelInfo {
    /// Parses the decoder model info from the given reader.
    pub fn parse(bit_reader: &mut BitReader<impl io::Read>) -> io::Result<Self> {
        let buffer_delay_length = bit_reader.read_bits(5)? as u8 + 1;
        let num_units_in_decoding_tick = bit_reader.read_u32::<BigEndian>()?;
        let buffer_removal_time_length = bit_reader.read_bits(5)? as u8 + 1;
        let frame_presentation_time_length = bit_reader.read_bits(5)? as u8 + 1;
        Ok(Self {
            buffer_delay_length,
            num_units_in_decoding_tick,
            buffer_removal_time_length,
            frame_presentation_time_length,
        })
    }
}

/// Operating parameters info
///
///  AV1-Spec-2 - 5.5.5
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct OperatingParametersInfo {
    /// `decoder_buffer_delay`
    pub decoder_buffer_delay: u64,
    /// `encoder_buffer_delay`
    pub encoder_buffer_delay: u64,
    /// `low_delay_mode_flag`
    ///
    /// 1 bit
    pub low_delay_mode_flag: bool,
}

impl OperatingParametersInfo {
    /// Parses the operating parameters info from the given reader.
    pub fn parse(delay_bit_length: u8, bit_reader: &mut BitReader<impl io::Read>) -> io::Result<Self> {
        let decoder_buffer_delay = bit_reader.read_bits(delay_bit_length)?;
        let encoder_buffer_delay = bit_reader.read_bits(delay_bit_length)?;
        let low_delay_mode_flag = bit_reader.read_bit()?;

        Ok(Self {
            decoder_buffer_delay,
            encoder_buffer_delay,
            low_delay_mode_flag,
        })
    }
}

/// Color config
///
/// AV1-Spec-2 - 5.5.2
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ColorConfig {
    /// `BitDepth`
    pub bit_depth: i32,
    /// `mono_chrome`
    ///
    /// 1 bit
    pub mono_chrome: bool,
    /// `NumPlanes`
    pub num_planes: u8,
    /// `color_primaries`
    ///
    /// 8 bits
    pub color_primaries: u8,
    /// `transfer_characteristics`
    ///
    /// 8 bits
    pub transfer_characteristics: u8,
    /// `matrix_coefficients`
    ///
    /// 8 bits
    pub matrix_coefficients: u8,
    /// `color_range`
    ///
    /// 1 bit
    pub full_color_range: bool,
    /// `subsampling_x`
    ///
    /// 1 bit
    pub subsampling_x: bool,
    /// `subsampling_y`
    ///
    /// 1 bit
    pub subsampling_y: bool,
    /// `chroma_sample_position`
    ///
    /// 2 bits
    pub chroma_sample_position: u8,
    /// `separate_uv_delta_q`
    ///
    /// 1 bit
    pub separate_uv_delta_q: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct ColorRangeAndSubsampling {
    color_range: bool,
    subsampling_x: bool,
    subsampling_y: bool,
}

impl ColorConfig {
    fn parse_color_range_and_subsampling(
        bit_reader: &mut BitReader<impl io::Read>,
        seq_profile: u8,
        color_primaries: u8,
        transfer_characteristics: u8,
        matrix_coefficients: u8,
        bit_depth: i32,
    ) -> io::Result<ColorRangeAndSubsampling> {
        let color_range;
        let subsampling_x;
        let subsampling_y;

        const CP_BT_709: u8 = 1;
        const TC_SRGB: u8 = 13;
        const MC_IDENTITY: u8 = 0;

        if color_primaries == CP_BT_709 && transfer_characteristics == TC_SRGB && matrix_coefficients == MC_IDENTITY {
            color_range = true;
            subsampling_x = false;
            subsampling_y = false;
        } else {
            color_range = bit_reader.read_bit()?;
            if seq_profile == 0 {
                subsampling_x = true;
                subsampling_y = true;
            } else if seq_profile == 1 {
                subsampling_x = false;
                subsampling_y = false;
            } else if bit_depth == 12 {
                subsampling_x = bit_reader.read_bit()?;
                if subsampling_x {
                    subsampling_y = bit_reader.read_bit()?;
                } else {
                    subsampling_y = false;
                }
            } else {
                subsampling_x = true;
                subsampling_y = false;
            }
        }

        Ok(ColorRangeAndSubsampling {
            color_range,
            subsampling_x,
            subsampling_y,
        })
    }

    /// Parses the color config from the given reader.
    pub fn parse(seq_profile: u8, bit_reader: &mut BitReader<impl io::Read>) -> io::Result<Self> {
        let high_bitdepth = bit_reader.read_bit()?;
        let bit_depth = match (seq_profile, high_bitdepth) {
            (2, true) if bit_reader.read_bit()? => 12,
            (_, true) => 10,
            (_, false) => 8,
        };

        let mono_chrome = if seq_profile == 1 { false } else { bit_reader.read_bit()? };

        let color_primaries;
        let transfer_characteristics;
        let matrix_coefficients;

        let color_description_present_flag = bit_reader.read_bit()?;
        if color_description_present_flag {
            color_primaries = bit_reader.read_bits(8)? as u8;
            transfer_characteristics = bit_reader.read_bits(8)? as u8;
            matrix_coefficients = bit_reader.read_bits(8)? as u8;
        } else {
            color_primaries = 2; // CP_UNSPECIFIED
            transfer_characteristics = 2; // TC_UNSPECIFIED
            matrix_coefficients = 2; // MC_UNSPECIFIED
        }

        let num_planes = if mono_chrome { 1 } else { 3 };

        if mono_chrome {
            Ok(ColorConfig {
                bit_depth,
                color_primaries,
                transfer_characteristics,
                matrix_coefficients,
                full_color_range: bit_reader.read_bit()?,
                subsampling_x: true,
                subsampling_y: true,
                mono_chrome,
                separate_uv_delta_q: false,
                chroma_sample_position: 0, // CSP_UNKNOWN
                num_planes,
            })
        } else {
            let ColorRangeAndSubsampling {
                color_range,
                subsampling_x,
                subsampling_y,
            } = Self::parse_color_range_and_subsampling(
                bit_reader,
                seq_profile,
                color_primaries,
                transfer_characteristics,
                matrix_coefficients,
                bit_depth,
            )?;

            let chroma_sample_position = if subsampling_x && subsampling_y {
                bit_reader.read_bits(2)? as u8
            } else {
                0 // CSP_UNKNOWN
            };

            let separate_uv_delta_q = bit_reader.read_bit()?;
            Ok(ColorConfig {
                bit_depth,
                mono_chrome,
                color_primaries,
                transfer_characteristics,
                matrix_coefficients,
                full_color_range: color_range,
                subsampling_x,
                subsampling_y,
                chroma_sample_position,
                separate_uv_delta_q,
                num_planes,
            })
        }
    }
}

impl SequenceHeaderObu {
    /// Returns a reference to the header of the OBU.
    pub const fn header(&self) -> &ObuHeader {
        &self.header
    }

    /// Parses the sequence header from the given reader.
    ///
    /// The given header will be part of the returned struct and can be accessed through the [`SequenceHeaderObu::header`] function.
    pub fn parse(header: ObuHeader, reader: &mut impl io::Read) -> io::Result<Self> {
        let mut bit_reader = BitReader::new(reader);

        let seq_profile = bit_reader.read_bits(3)? as u8;
        let still_picture = bit_reader.read_bit()?;
        let reduced_still_picture_header = bit_reader.read_bit()?;

        if !still_picture && reduced_still_picture_header {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "reduced_still_picture_header is true but still_picture is false",
            ));
        }

        let mut timing_info = None;
        let mut decoder_model_info = None;
        let mut operating_points = Vec::new();

        if reduced_still_picture_header {
            operating_points.push(OperatingPoint {
                idc: 0,
                seq_level_idx: bit_reader.read_bits(5)? as u8,
                seq_tier: false,
                operating_parameters_info: None,
                initial_display_delay: None,
            });
        } else {
            let timing_info_present_flag = bit_reader.read_bit()?;
            if timing_info_present_flag {
                timing_info = Some(TimingInfo::parse(&mut bit_reader)?);

                let decoder_model_info_present_flag = bit_reader.read_bit()?;
                if decoder_model_info_present_flag {
                    decoder_model_info = Some(DecoderModelInfo::parse(&mut bit_reader)?);
                }
            }

            let initial_display_delay_present_flag = bit_reader.read_bit()?;
            let operating_points_cnt_minus_1 = bit_reader.read_bits(5)? as u8;
            for _ in 0..operating_points_cnt_minus_1 + 1 {
                let idc = bit_reader.read_bits(12)? as u16;
                let seq_level_idx = bit_reader.read_bits(5)? as u8;
                let seq_tier = if seq_level_idx > 7 { bit_reader.read_bit()? } else { false };
                let decoder_model_present_for_this_op = if let Some(decoder_model_info) = decoder_model_info {
                    bit_reader.read_bit()?.then_some(decoder_model_info.buffer_delay_length)
                } else {
                    None
                };

                let operating_parameters_info = if let Some(delay_bit_length) = decoder_model_present_for_this_op {
                    Some(OperatingParametersInfo::parse(delay_bit_length, &mut bit_reader)?)
                } else {
                    None
                };

                let initial_display_delay = if initial_display_delay_present_flag {
                    if bit_reader.read_bit()? {
                        // initial_display_delay_present_for_this_op
                        Some(bit_reader.read_bits(4)? as u8 + 1) // initial_display_delay_minus_1
                    } else {
                        None
                    }
                } else {
                    None
                };

                operating_points.push(OperatingPoint {
                    idc,
                    seq_level_idx,
                    seq_tier,
                    operating_parameters_info,
                    initial_display_delay,
                });
            }
        }

        let frame_width_bits = bit_reader.read_bits(4)? as u8 + 1;
        let frame_height_bits = bit_reader.read_bits(4)? as u8 + 1;

        let max_frame_width = bit_reader.read_bits(frame_width_bits)? + 1;
        let max_frame_height = bit_reader.read_bits(frame_height_bits)? + 1;

        let frame_id_numbers_present_flag = if reduced_still_picture_header {
            false
        } else {
            bit_reader.read_bit()?
        };
        let frame_ids = if frame_id_numbers_present_flag {
            let delta_frame_id_length = bit_reader.read_bits(4)? as u8 + 2;
            let additional_frame_id_length = bit_reader.read_bits(3)? as u8 + 1;
            Some(FrameIds {
                delta_frame_id_length,
                additional_frame_id_length,
            })
        } else {
            None
        };

        let use_128x128_superblock = bit_reader.read_bit()?;
        let enable_filter_intra = bit_reader.read_bit()?;
        let enable_intra_edge_filter = bit_reader.read_bit()?;

        let enable_interintra_compound;
        let enable_masked_compound;
        let enable_warped_motion;
        let enable_dual_filter;
        let enable_order_hint;
        let enable_jnt_comp;
        let enable_ref_frame_mvs;
        let order_hint_bits;
        let seq_force_integer_mv;

        let seq_force_screen_content_tools;

        if !reduced_still_picture_header {
            enable_interintra_compound = bit_reader.read_bit()?;
            enable_masked_compound = bit_reader.read_bit()?;
            enable_warped_motion = bit_reader.read_bit()?;
            enable_dual_filter = bit_reader.read_bit()?;
            enable_order_hint = bit_reader.read_bit()?;
            if enable_order_hint {
                enable_jnt_comp = bit_reader.read_bit()?;
                enable_ref_frame_mvs = bit_reader.read_bit()?;
            } else {
                enable_jnt_comp = false;
                enable_ref_frame_mvs = false;
            }
            if bit_reader.read_bit()? {
                // seq_choose_screen_content_tools
                seq_force_screen_content_tools = 2; // SELECT_SCREEN_CONTENT_TOOLS
            } else {
                seq_force_screen_content_tools = bit_reader.read_bits(1)? as u8;
            }

            // If seq_force_screen_content_tools is 0, then seq_force_integer_mv must be 2.
            // Or if the next bit is 0, then seq_force_integer_mv must be 2.
            if seq_force_screen_content_tools == 0 || bit_reader.read_bit()? {
                seq_force_integer_mv = 2; // SELECT_INTEGER_MV
            } else {
                seq_force_integer_mv = bit_reader.read_bits(1)? as u8;
            }

            if enable_order_hint {
                order_hint_bits = bit_reader.read_bits(3)? as u8 + 1;
            } else {
                order_hint_bits = 0;
            }
        } else {
            enable_interintra_compound = false;
            enable_masked_compound = false;
            enable_warped_motion = false;
            enable_dual_filter = false;
            enable_order_hint = false;
            enable_jnt_comp = false;
            enable_ref_frame_mvs = false;
            seq_force_screen_content_tools = 2; // SELECT_SCREEN_CONTENT_TOOLS
            seq_force_integer_mv = 2; // SELECT_INTEGER_MV
            order_hint_bits = 0;
        }

        let enable_superres = bit_reader.read_bit()?;
        let enable_cdef = bit_reader.read_bit()?;
        let enable_restoration = bit_reader.read_bit()?;

        let color_config = ColorConfig::parse(seq_profile, &mut bit_reader)?;

        let film_grain_params_present = bit_reader.read_bit()?;

        Ok(Self {
            header,
            seq_profile,
            still_picture,
            reduced_still_picture_header,
            operating_points,
            decoder_model_info,
            max_frame_width,
            max_frame_height,
            frame_ids,
            use_128x128_superblock,
            enable_filter_intra,
            enable_intra_edge_filter,
            enable_interintra_compound,
            enable_masked_compound,
            enable_warped_motion,
            enable_dual_filter,
            enable_order_hint,
            enable_jnt_comp,
            enable_ref_frame_mvs,
            seq_force_screen_content_tools,
            seq_force_integer_mv,
            order_hint_bits,
            enable_superres,
            enable_cdef,
            enable_restoration,
            timing_info,
            color_config,
            film_grain_params_present,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use byteorder::WriteBytesExt;
    use scuffle_bytes_util::BitWriter;

    use super::*;
    use crate::ObuType;

    #[test]
    fn test_seq_obu_parse() {
        let obu = b"\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@";

        let header = ObuHeader {
            obu_type: ObuType::SequenceHeader,
            size: None,
            extension_header: None,
        };

        let seq_header = SequenceHeaderObu::parse(header, &mut io::Cursor::new(obu)).unwrap();

        insta::assert_debug_snapshot!(seq_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 0,
            still_picture: false,
            reduced_still_picture_header: false,
            timing_info: None,
            decoder_model_info: None,
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 13,
                    seq_tier: false,
                    operating_parameters_info: None,
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 3840,
            max_frame_height: 2160,
            frame_ids: None,
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: true,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 0,
            seq_force_integer_mv: 2,
            order_hint_bits: 7,
            enable_superres: false,
            enable_cdef: true,
            enable_restoration: true,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: false,
                num_planes: 3,
                color_primaries: 1,
                transfer_characteristics: 1,
                matrix_coefficients: 1,
                full_color_range: false,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: false,
        }
        ");

        assert_eq!(seq_header.header(), &header);
    }

    #[test]
    fn test_seq_obu_parse_reduced_still_picture() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(true).unwrap(); // still_picture
        bits.write_bit(true).unwrap(); // reduced_still_picture_header
        bits.write_bits(11, 5).unwrap(); // seq_lvl_idx

        bits.write_bits(15, 4).unwrap();
        bits.write_bits(15, 4).unwrap();
        bits.write_bits(1919, 16).unwrap();
        bits.write_bits(1079, 16).unwrap();

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter
        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: true,
            reduced_still_picture_header: true,
            timing_info: None,
            decoder_model_info: None,
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 11,
                    seq_tier: false,
                    operating_parameters_info: None,
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: None,
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: false,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 2,
            seq_force_integer_mv: 2,
            order_hint_bits: 0,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_seq_obu_parse_timing_info_decoder_model_preset() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(false).unwrap(); // still_picture
        bits.write_bit(false).unwrap(); // reduced_still_picture_header
        bits.write_bit(true).unwrap(); // timing_info_present_flag

        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_display_tick
        bits.write_u32::<BigEndian>(1).unwrap(); // time_scale
        bits.write_bit(false).unwrap(); // num_ticks_per_picture

        bits.write_bit(true).unwrap(); // decoder_model_info_present_flag
        bits.write_bits(4, 5).unwrap(); // buffer_delay_length
        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_decoding_tick
        bits.write_bits(4, 5).unwrap(); // buffer_removal_time_length
        bits.write_bits(4, 5).unwrap(); // frame_presentation_time_length

        bits.write_bit(true).unwrap(); // initial_display_delay_present_flag
        bits.write_bits(0, 5).unwrap(); // operating_points_cnt_minus_1

        bits.write_bits(0, 12).unwrap(); // idc
        bits.write_bits(1, 5).unwrap(); // seq_lvl_idx
        bits.write_bit(true).unwrap(); // seq_tier

        bits.write_bits(0b1010, 5).unwrap(); // decoder_buffer_delay
        bits.write_bits(0b0101, 5).unwrap(); // encoder_buffer_delay
        bits.write_bit(false).unwrap(); // low_delay_mode_flag

        bits.write_bit(true).unwrap(); // film_grain_params_present
        bits.write_bits(15, 4).unwrap(); // initial_display_delay_minus_1

        bits.write_bits(15, 4).unwrap(); // operating_points_cnt_minus_1
        bits.write_bits(15, 4).unwrap(); // operating_points_cnt_minus_1
        bits.write_bits(1919, 16).unwrap(); // operating_points_cnt_minus_1
        bits.write_bits(1079, 16).unwrap(); // operating_points_cnt_minus_1

        bits.write_bit(true).unwrap(); // frame_id_numbers_present_flag
        bits.write_bits(0b1101, 4).unwrap(); // delta_frame_id_length
        bits.write_bits(0b101, 3).unwrap(); // additional_frame_id_length

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter

        bits.write_bit(false).unwrap(); // enable_interintra_compound
        bits.write_bit(false).unwrap(); // enable_masked_compound
        bits.write_bit(false).unwrap(); // enable_warped_motion
        bits.write_bit(false).unwrap(); // enable_dual_filter
        bits.write_bit(true).unwrap(); // enable_order_hint
        bits.write_bit(false).unwrap(); // enable_jnt_comp
        bits.write_bit(false).unwrap(); // enable_ref_frame_mvs

        bits.write_bit(false).unwrap();
        bits.write_bit(true).unwrap();
        bits.write_bit(false).unwrap();
        bits.write_bit(false).unwrap();

        bits.write_bits(0b100, 3).unwrap();

        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: false,
            reduced_still_picture_header: false,
            timing_info: Some(
                TimingInfo {
                    num_units_in_display_tick: 1,
                    time_scale: 1,
                    num_ticks_per_picture: None,
                },
            ),
            decoder_model_info: Some(
                DecoderModelInfo {
                    buffer_delay_length: 5,
                    num_units_in_decoding_tick: 1,
                    buffer_removal_time_length: 5,
                    frame_presentation_time_length: 5,
                },
            ),
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 1,
                    seq_tier: false,
                    operating_parameters_info: Some(
                        OperatingParametersInfo {
                            decoder_buffer_delay: 10,
                            encoder_buffer_delay: 5,
                            low_delay_mode_flag: false,
                        },
                    ),
                    initial_display_delay: Some(
                        16,
                    ),
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: Some(
                FrameIds {
                    delta_frame_id_length: 15,
                    additional_frame_id_length: 6,
                },
            ),
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: true,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 1,
            seq_force_integer_mv: 0,
            order_hint_bits: 5,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_seq_obu_parse_num_ticks_per_picture() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(false).unwrap(); // still_picture
        bits.write_bit(false).unwrap(); // reduced_still_picture_header
        bits.write_bit(true).unwrap(); // timing_info_present_flag

        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_display_tick
        bits.write_u32::<BigEndian>(1).unwrap(); // time_scale
        bits.write_bit(true).unwrap(); // num_ticks_per_picture
        bits.write_bits(0b01, 1).unwrap(); // read_uvlc

        bits.write_bit(true).unwrap(); // decoder_model_info_present_flag
        bits.write_bits(4, 5).unwrap(); // buffer_delay_length
        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_decoding_tick
        bits.write_bits(4, 5).unwrap(); // buffer_removal_time_length
        bits.write_bits(4, 5).unwrap(); // frame_presentation_time_length

        bits.write_bit(true).unwrap(); // initial_display_delay_present_flag
        bits.write_bits(0, 5).unwrap(); // operating_points_cnt_minus_1

        bits.write_bits(0, 12).unwrap(); // idc
        bits.write_bits(1, 5).unwrap(); // seq_lvl_idx
        bits.write_bit(true).unwrap(); // seq_tier

        bits.write_bits(0b1010, 5).unwrap(); // decoder_buffer_delay
        bits.write_bits(0b0101, 5).unwrap(); // encoder_buffer_delay
        bits.write_bit(false).unwrap(); // low_delay_mode_flag

        bits.write_bit(true).unwrap(); // film_grain_params_present
        bits.write_bits(15, 4).unwrap(); // initial_display_delay_minus_1

        bits.write_bits(15, 4).unwrap(); // operating_points_cnt_minus_1
        bits.write_bits(15, 4).unwrap(); // operating_points_cnt_minus_1
        bits.write_bits(1919, 16).unwrap(); // operating_points_cnt_minus_1
        bits.write_bits(1079, 16).unwrap(); // operating_points_cnt_minus_1

        bits.write_bit(true).unwrap(); // frame_id_numbers_present_flag
        bits.write_bits(0b1101, 4).unwrap(); // delta_frame_id_length
        bits.write_bits(0b101, 3).unwrap(); // additional_frame_id_length

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter

        bits.write_bit(false).unwrap(); // enable_interintra_compound
        bits.write_bit(false).unwrap(); // enable_masked_compound
        bits.write_bit(false).unwrap(); // enable_warped_motion
        bits.write_bit(false).unwrap(); // enable_dual_filter
        bits.write_bit(true).unwrap(); // enable_order_hint
        bits.write_bit(false).unwrap(); // enable_jnt_comp
        bits.write_bit(false).unwrap(); // enable_ref_frame_mvs

        bits.write_bit(false).unwrap();
        bits.write_bit(true).unwrap();
        bits.write_bit(false).unwrap();
        bits.write_bit(false).unwrap();

        bits.write_bits(0b100, 3).unwrap();

        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: false,
            reduced_still_picture_header: false,
            timing_info: Some(
                TimingInfo {
                    num_units_in_display_tick: 1,
                    time_scale: 1,
                    num_ticks_per_picture: Some(
                        1,
                    ),
                },
            ),
            decoder_model_info: Some(
                DecoderModelInfo {
                    buffer_delay_length: 5,
                    num_units_in_decoding_tick: 1,
                    buffer_removal_time_length: 5,
                    frame_presentation_time_length: 5,
                },
            ),
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 1,
                    seq_tier: false,
                    operating_parameters_info: Some(
                        OperatingParametersInfo {
                            decoder_buffer_delay: 10,
                            encoder_buffer_delay: 5,
                            low_delay_mode_flag: false,
                        },
                    ),
                    initial_display_delay: Some(
                        16,
                    ),
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: Some(
                FrameIds {
                    delta_frame_id_length: 15,
                    additional_frame_id_length: 6,
                },
            ),
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: true,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 1,
            seq_force_integer_mv: 0,
            order_hint_bits: 5,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_seq_obu_parse_initial_display_delay_is_none() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(false).unwrap(); // still_picture
        bits.write_bit(false).unwrap(); // reduced_still_picture_header
        bits.write_bit(true).unwrap(); // timing_info_present_flag

        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_display_tick
        bits.write_u32::<BigEndian>(1).unwrap(); // time_scale
        bits.write_bit(false).unwrap(); // num_ticks_per_picture

        bits.write_bit(true).unwrap(); // decoder_model_info_present_flag
        bits.write_bits(4, 5).unwrap(); // buffer_delay_length
        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_decoding_tick
        bits.write_bits(4, 5).unwrap(); // buffer_removal_time_length
        bits.write_bits(4, 5).unwrap(); // frame_presentation_time_length

        bits.write_bit(true).unwrap(); // initial_display_delay_present_flag
        bits.write_bits(0, 5).unwrap(); // operating_points_cnt_minus_1

        bits.write_bits(0, 12).unwrap(); // idc
        bits.write_bits(1, 5).unwrap(); // seq_lvl_idx
        bits.write_bit(true).unwrap(); // seq_tier

        bits.write_bits(0b1010, 5).unwrap(); // decoder_buffer_delay
        bits.write_bits(0b0101, 5).unwrap(); // encoder_buffer_delay
        bits.write_bit(false).unwrap(); // low_delay_mode_flag

        bits.write_bit(false).unwrap(); // initial_display_delay_present_for_this_op

        bits.write_bits(11, 4).unwrap(); // frame_width_bits
        bits.write_bits(11, 4).unwrap(); // frame_height_bits
        bits.write_bits(1919, 12).unwrap(); // max_frame_width
        bits.write_bits(1079, 12).unwrap(); // max_frame_height

        bits.write_bit(true).unwrap(); // frame_id_numbers_present_flag
        bits.write_bits(0b1101, 4).unwrap(); // delta_frame_id_length
        bits.write_bits(0b101, 3).unwrap(); // additional_frame_id_length

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter

        bits.write_bit(false).unwrap(); // enable_interintra_compound
        bits.write_bit(false).unwrap(); // enable_masked_compound
        bits.write_bit(false).unwrap(); // enable_warped_motion
        bits.write_bit(false).unwrap(); // enable_dual_filter
        bits.write_bit(true).unwrap(); // enable_order_hint
        bits.write_bit(false).unwrap(); // enable_jnt_comp
        bits.write_bit(false).unwrap(); // enable_ref_frame_mvs

        bits.write_bit(false).unwrap();
        bits.write_bit(true).unwrap();
        bits.write_bit(false).unwrap();
        bits.write_bit(false).unwrap();

        bits.write_bits(0b100, 3).unwrap();

        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: false,
            reduced_still_picture_header: false,
            timing_info: Some(
                TimingInfo {
                    num_units_in_display_tick: 1,
                    time_scale: 1,
                    num_ticks_per_picture: None,
                },
            ),
            decoder_model_info: Some(
                DecoderModelInfo {
                    buffer_delay_length: 5,
                    num_units_in_decoding_tick: 1,
                    buffer_removal_time_length: 5,
                    frame_presentation_time_length: 5,
                },
            ),
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 1,
                    seq_tier: false,
                    operating_parameters_info: Some(
                        OperatingParametersInfo {
                            decoder_buffer_delay: 10,
                            encoder_buffer_delay: 5,
                            low_delay_mode_flag: false,
                        },
                    ),
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: Some(
                FrameIds {
                    delta_frame_id_length: 15,
                    additional_frame_id_length: 6,
                },
            ),
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: true,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 1,
            seq_force_integer_mv: 0,
            order_hint_bits: 5,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_seq_obu_parse_enable_order_hint_is_false() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(false).unwrap(); // still_picture
        bits.write_bit(false).unwrap(); // reduced_still_picture_header
        bits.write_bit(true).unwrap(); // timing_info_present_flag

        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_display_tick
        bits.write_u32::<BigEndian>(1).unwrap(); // time_scale
        bits.write_bit(false).unwrap(); // num_ticks_per_picture

        bits.write_bit(true).unwrap(); // decoder_model_info_present_flag
        bits.write_bits(4, 5).unwrap(); // buffer_delay_length
        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_decoding_tick
        bits.write_bits(4, 5).unwrap(); // buffer_removal_time_length
        bits.write_bits(4, 5).unwrap(); // frame_presentation_time_length

        bits.write_bit(true).unwrap(); // initial_display_delay_present_flag
        bits.write_bits(0, 5).unwrap(); // operating_points_cnt_minus_1

        bits.write_bits(0, 12).unwrap(); // idc
        bits.write_bits(1, 5).unwrap(); // seq_lvl_idx
        bits.write_bit(true).unwrap(); // seq_tier

        bits.write_bits(0b1010, 5).unwrap(); // decoder_buffer_delay
        bits.write_bits(0b0101, 5).unwrap(); // encoder_buffer_delay
        bits.write_bit(false).unwrap(); // low_delay_mode_flag

        bits.write_bit(false).unwrap(); // initial_display_delay_present_for_this_op

        bits.write_bits(11, 4).unwrap(); // frame_width_bits
        bits.write_bits(11, 4).unwrap(); // frame_height_bits
        bits.write_bits(1919, 12).unwrap(); // max_frame_width
        bits.write_bits(1079, 12).unwrap(); // max_frame_height

        bits.write_bit(true).unwrap(); // frame_id_numbers_present_flag
        bits.write_bits(0b1101, 4).unwrap(); // delta_frame_id_length
        bits.write_bits(0b101, 3).unwrap(); // additional_frame_id_length

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter

        bits.write_bit(false).unwrap(); // enable_interintra_compound
        bits.write_bit(false).unwrap(); // enable_masked_compound
        bits.write_bit(false).unwrap(); // enable_warped_motion
        bits.write_bit(false).unwrap(); // enable_dual_filter
        bits.write_bit(false).unwrap(); // enable_order_hint

        bits.write_bit(true).unwrap(); // seq_choose_screen_content_tools
        bits.write_bit(true).unwrap(); // sets seq_force_integer_mv to be 2

        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: false,
            reduced_still_picture_header: false,
            timing_info: Some(
                TimingInfo {
                    num_units_in_display_tick: 1,
                    time_scale: 1,
                    num_ticks_per_picture: None,
                },
            ),
            decoder_model_info: Some(
                DecoderModelInfo {
                    buffer_delay_length: 5,
                    num_units_in_decoding_tick: 1,
                    buffer_removal_time_length: 5,
                    frame_presentation_time_length: 5,
                },
            ),
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 1,
                    seq_tier: false,
                    operating_parameters_info: Some(
                        OperatingParametersInfo {
                            decoder_buffer_delay: 10,
                            encoder_buffer_delay: 5,
                            low_delay_mode_flag: false,
                        },
                    ),
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: Some(
                FrameIds {
                    delta_frame_id_length: 15,
                    additional_frame_id_length: 6,
                },
            ),
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: false,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 2,
            seq_force_integer_mv: 2,
            order_hint_bits: 0,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_seq_obu_parse_decoder_model_info_present_is_false() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(false).unwrap(); // still_picture
        bits.write_bit(false).unwrap(); // reduced_still_picture_header
        bits.write_bit(true).unwrap(); // timing_info_present_flag

        bits.write_u32::<BigEndian>(1).unwrap(); // num_units_in_display_tick
        bits.write_u32::<BigEndian>(1).unwrap(); // time_scale
        bits.write_bit(false).unwrap(); // num_ticks_per_picture

        bits.write_bit(false).unwrap(); // decoder_model_info_present_flag

        bits.write_bit(true).unwrap(); // initial_display_delay_present_flag
        bits.write_bits(0, 5).unwrap(); // operating_points_cnt_minus_1

        bits.write_bits(0, 12).unwrap(); // idc
        bits.write_bits(1, 5).unwrap(); // seq_lvl_idx

        bits.write_bit(false).unwrap(); // initial_display_delay_present_for_this_op

        bits.write_bits(11, 4).unwrap(); // frame_width_bits
        bits.write_bits(11, 4).unwrap(); // frame_height_bits
        bits.write_bits(1919, 12).unwrap(); // max_frame_width
        bits.write_bits(1079, 12).unwrap(); // max_frame_height

        bits.write_bit(true).unwrap(); // frame_id_numbers_present_flag
        bits.write_bits(0b1101, 4).unwrap(); // delta_frame_id_length
        bits.write_bits(0b101, 3).unwrap(); // additional_frame_id_length

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter

        bits.write_bit(false).unwrap(); // enable_interintra_compound
        bits.write_bit(false).unwrap(); // enable_masked_compound
        bits.write_bit(false).unwrap(); // enable_warped_motion
        bits.write_bit(false).unwrap(); // enable_dual_filter
        bits.write_bit(false).unwrap(); // enable_order_hint

        bits.write_bit(true).unwrap(); // seq_choose_screen_content_tools
        bits.write_bit(true).unwrap(); // sets seq_force_integer_mv to be 2

        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: false,
            reduced_still_picture_header: false,
            timing_info: Some(
                TimingInfo {
                    num_units_in_display_tick: 1,
                    time_scale: 1,
                    num_ticks_per_picture: None,
                },
            ),
            decoder_model_info: None,
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 1,
                    seq_tier: false,
                    operating_parameters_info: None,
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: Some(
                FrameIds {
                    delta_frame_id_length: 15,
                    additional_frame_id_length: 6,
                },
            ),
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: false,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 2,
            seq_force_integer_mv: 2,
            order_hint_bits: 0,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_seq_obu_parse_color_range_and_subsampling() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bit(false).unwrap(); // color_range
        bits.write_bit(false).unwrap(); // subsampling_x
        bits.write_bit(false).unwrap(); // subsampling_y

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(Vec::new())),
            0,
            1,
            13,
            0,
            8,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: false,
                subsampling_y: false,
            }
        );

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(&[0b10000000])),
            0,
            1,
            0,
            0,
            8,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: true,
                subsampling_y: true,
            }
        );

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(&[0b10000000])),
            1,
            1,
            0,
            0,
            8,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: false,
                subsampling_y: false,
            }
        );

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(&[0b11100000])),
            2,
            1,
            0,
            0,
            12,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: true,
                subsampling_y: true,
            }
        );

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(&[0b11000000])),
            2,
            1,
            0,
            0,
            12,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: true,
                subsampling_y: false,
            }
        );

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(&[0b10100000])),
            2,
            1,
            0,
            0,
            12,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: false,
                subsampling_y: false,
            }
        );

        let color_range_and_subsampling = ColorConfig::parse_color_range_and_subsampling(
            &mut BitReader::new(std::io::Cursor::new(&[0b11100000])),
            2,
            1,
            0,
            0,
            8,
        )
        .unwrap();

        assert_eq!(
            color_range_and_subsampling,
            ColorRangeAndSubsampling {
                color_range: true,
                subsampling_x: true,
                subsampling_y: false,
            }
        );
    }

    #[test]
    fn test_color_config_parse_bit_depth_12() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(true).unwrap(); // still_picture
        bits.write_bit(true).unwrap(); // reduced_still_picture_header
        bits.write_bits(11, 5).unwrap(); // seq_lvl_idx

        bits.write_bits(15, 4).unwrap();
        bits.write_bits(15, 4).unwrap();
        bits.write_bits(1919, 16).unwrap();
        bits.write_bits(1079, 16).unwrap();

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter
        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(true).unwrap(); // high_bitdepth
        bits.write_bit(true).unwrap(); // sets bitdepth to 12 instead of 10
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: true,
            reduced_still_picture_header: true,
            timing_info: None,
            decoder_model_info: None,
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 11,
                    seq_tier: false,
                    operating_parameters_info: None,
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: None,
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: false,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 2,
            seq_force_integer_mv: 2,
            order_hint_bits: 0,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 12,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_color_config_parse_bit_depth_10() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b010, 3).unwrap(); // seq_profile (2)
        bits.write_bit(true).unwrap(); // still_picture
        bits.write_bit(true).unwrap(); // reduced_still_picture_header
        bits.write_bits(11, 5).unwrap(); // seq_lvl_idx

        bits.write_bits(15, 4).unwrap();
        bits.write_bits(15, 4).unwrap();
        bits.write_bits(1919, 16).unwrap();
        bits.write_bits(1079, 16).unwrap();

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter
        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(true).unwrap(); // high_bitdepth
        bits.write_bit(false).unwrap(); // sets bitdepth to 10 instead of 12
        bits.write_bit(true).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // color_range
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 2,
            still_picture: true,
            reduced_still_picture_header: true,
            timing_info: None,
            decoder_model_info: None,
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 11,
                    seq_tier: false,
                    operating_parameters_info: None,
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: None,
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: false,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 2,
            seq_force_integer_mv: 2,
            order_hint_bits: 0,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 10,
                mono_chrome: true,
                num_planes: 1,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: true,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position: 0,
                separate_uv_delta_q: false,
            },
            film_grain_params_present: true,
        }
        ");
    }

    #[test]
    fn test_color_config_parse_csp_unknown() {
        let mut bits = BitWriter::new(Vec::new());

        bits.write_bits(0b001, 3).unwrap(); // seq_profile (1)
        bits.write_bit(true).unwrap(); // still_picture
        bits.write_bit(true).unwrap(); // reduced_still_picture_header
        bits.write_bits(11, 5).unwrap(); // seq_lvl_idx

        bits.write_bits(15, 4).unwrap();
        bits.write_bits(15, 4).unwrap();
        bits.write_bits(1919, 16).unwrap();
        bits.write_bits(1079, 16).unwrap();

        bits.write_bit(false).unwrap(); // use_128x128_superblock
        bits.write_bit(false).unwrap(); // enable_filter_intra
        bits.write_bit(false).unwrap(); // enable_intra_edge_filter
        bits.write_bit(false).unwrap(); // enable_superres
        bits.write_bit(false).unwrap(); // enable_cdef
        bits.write_bit(false).unwrap(); // enable_restoration

        bits.write_bit(false).unwrap(); // high_bitdepth
        bits.write_bit(false).unwrap(); // mono_chrome
        bits.write_bit(false).unwrap(); // color_description_present_flag
        bits.write_bit(true).unwrap(); // separate_uv_delta_q

        bits.write_bit(true).unwrap(); // film_grain_params_present

        let obu_header = SequenceHeaderObu::parse(
            ObuHeader {
                obu_type: ObuType::SequenceHeader,
                size: None,
                extension_header: None,
            },
            &mut io::Cursor::new(bits.finish().unwrap()),
        )
        .unwrap();

        insta::assert_debug_snapshot!(obu_header, @r"
        SequenceHeaderObu {
            header: ObuHeader {
                obu_type: SequenceHeader,
                size: None,
                extension_header: None,
            },
            seq_profile: 1,
            still_picture: true,
            reduced_still_picture_header: true,
            timing_info: None,
            decoder_model_info: None,
            operating_points: [
                OperatingPoint {
                    idc: 0,
                    seq_level_idx: 11,
                    seq_tier: false,
                    operating_parameters_info: None,
                    initial_display_delay: None,
                },
            ],
            max_frame_width: 1920,
            max_frame_height: 1080,
            frame_ids: None,
            use_128x128_superblock: false,
            enable_filter_intra: false,
            enable_intra_edge_filter: false,
            enable_interintra_compound: false,
            enable_masked_compound: false,
            enable_warped_motion: false,
            enable_dual_filter: false,
            enable_order_hint: false,
            enable_jnt_comp: false,
            enable_ref_frame_mvs: false,
            seq_force_screen_content_tools: 2,
            seq_force_integer_mv: 2,
            order_hint_bits: 0,
            enable_superres: false,
            enable_cdef: false,
            enable_restoration: false,
            color_config: ColorConfig {
                bit_depth: 8,
                mono_chrome: false,
                num_planes: 3,
                color_primaries: 2,
                transfer_characteristics: 2,
                matrix_coefficients: 2,
                full_color_range: false,
                subsampling_x: false,
                subsampling_y: false,
                chroma_sample_position: 0,
                separate_uv_delta_q: true,
            },
            film_grain_params_present: true,
        }
        ");
    }
}
