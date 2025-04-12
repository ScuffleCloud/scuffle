use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use hrd_parameters::HrdParameters;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::{AspectRatioIdc, VideoFormat};

mod hrd_parameters;

#[derive(Debug, Clone, PartialEq)]
pub struct VuiParameters {
    pub aspect_ratio_info: Option<AspectRatioInfo>,
    pub overscan_appropriate_flag: Option<bool>,
    pub video_signal_type: Option<VideoSignalType>,
    pub chroma_loc_info: Option<ChromaLocInfo>,
    pub neutral_chroma_indication_flag: bool,
    pub field_seq_flag: bool,
    pub frame_field_info_present_flag: bool,
    pub default_display_window: Option<DefaultDisplayWindow>,
    pub vui_timing_info: Option<VuiTimingInfo>,
    pub bitstream_restriction: Option<BitStreamRestriction>,
}

impl VuiParameters {
    pub fn parse<R: io::Read>(bit_reader: &mut BitReader<R>, sps_max_sub_layers_minus1: u8) -> io::Result<Self> {
        let mut aspect_ratio_info = None;
        let mut overscan_appropriate_flag = None;
        let mut video_signal_type = None;
        let mut chroma_loc_info = None;
        let mut default_display_window = None;
        let mut vui_timing_info = None;
        let mut bitstream_restriction = None;

        let aspect_ratio_info_present_flag = bit_reader.read_bit()?;
        if aspect_ratio_info_present_flag {
            let aspect_ratio_idc = bit_reader.read_u8()?;
            if aspect_ratio_idc == AspectRatioIdc::ExtendedSar.0 {
                let sar_width = bit_reader.read_u16::<BigEndian>()?;
                let sar_height = bit_reader.read_u16::<BigEndian>()?;
                aspect_ratio_info = Some(AspectRatioInfo::ExtendedSar { sar_width, sar_height });
            } else {
                aspect_ratio_info = Some(AspectRatioInfo::Predefined(aspect_ratio_idc.into()));
            }
        }

        let overscan_info_present_flag = bit_reader.read_bit()?;
        if overscan_info_present_flag {
            overscan_appropriate_flag = Some(bit_reader.read_bit()?);
        }

        let video_signal_type_present_flag = bit_reader.read_bit()?;
        if video_signal_type_present_flag {
            let video_format = bit_reader.read_bits(3)? as u8;
            let video_full_range_flag = bit_reader.read_bit()?;
            let colour_description_present_flag = bit_reader.read_bit()?;

            if colour_description_present_flag {
                video_signal_type = Some(VideoSignalType {
                    video_format: VideoFormat::from(video_format),
                    video_full_range_flag,
                    color_primaries: bit_reader.read_u8()?,
                    transfer_characteristics: bit_reader.read_u8()?,
                    matrix_coeffs: bit_reader.read_u8()?,
                });
            } else {
                video_signal_type = Some(VideoSignalType {
                    video_format: VideoFormat::from(video_format),
                    video_full_range_flag,
                    color_primaries: 2,
                    transfer_characteristics: 2,
                    matrix_coeffs: 2,
                });
            }
        }

        let chroma_loc_info_present_flag = bit_reader.read_bit()?;
        if chroma_loc_info_present_flag {
            let chroma_sample_loc_type_top_field = bit_reader.read_exp_golomb()?;
            let chroma_sample_loc_type_bottom_field = bit_reader.read_exp_golomb()?;

            chroma_loc_info = Some(ChromaLocInfo {
                top_field: chroma_sample_loc_type_top_field,
                bottom_field: chroma_sample_loc_type_bottom_field,
            });
        }

        let neutral_chroma_indication_flag = bit_reader.read_bit()?;
        let field_seq_flag = bit_reader.read_bit()?;
        let frame_field_info_present_flag = bit_reader.read_bit()?;

        let default_display_window_flag = bit_reader.read_bit()?;
        if default_display_window_flag {
            default_display_window = Some(DefaultDisplayWindow {
                left_offset: bit_reader.read_exp_golomb()?,
                right_offset: bit_reader.read_exp_golomb()?,
                top_offset: bit_reader.read_exp_golomb()?,
                bottom_offset: bit_reader.read_exp_golomb()?,
            });
        }

        let vui_timing_info_present_flag = bit_reader.read_bit()?;
        if vui_timing_info_present_flag {
            let vui_num_units_in_tick = bit_reader.read_u32::<BigEndian>()?;
            let vui_time_scale = bit_reader.read_u32::<BigEndian>()?;

            let mut vui_num_ticks_poc_diff_one_minus1 = None;
            let vui_poc_proportional_to_timing_flag = bit_reader.read_bit()?;
            if vui_poc_proportional_to_timing_flag {
                vui_num_ticks_poc_diff_one_minus1 = Some(bit_reader.read_exp_golomb()?);
            }

            let mut vui_hrd_parameters = None;
            let vui_hrd_parameters_present_flag = bit_reader.read_bit()?;
            if vui_hrd_parameters_present_flag {
                vui_hrd_parameters = Some(HrdParameters::parse(bit_reader, true, sps_max_sub_layers_minus1)?);
            }

            vui_timing_info = Some(VuiTimingInfo {
                num_units_in_tick: vui_num_units_in_tick,
                time_scale: vui_time_scale,
                num_ticks_poc_diff_one_minus1: vui_num_ticks_poc_diff_one_minus1,
                hrd_parameters: vui_hrd_parameters,
            });
        }

        let bitstream_restriction_flag = bit_reader.read_bit()?;
        if bitstream_restriction_flag {
            bitstream_restriction = Some(BitStreamRestriction {
                tiles_fixed_structure_flag: bit_reader.read_bit()?,
                motion_vectors_over_pic_boundaries_flag: bit_reader.read_bit()?,
                restricted_ref_pic_lists_flag: bit_reader.read_bit()?,
                min_spatial_segmentation_idc: bit_reader.read_exp_golomb()?,
                max_bytes_per_pic_denom: bit_reader.read_exp_golomb()?,
                max_bits_per_min_cu_denom: bit_reader.read_exp_golomb()?,
                log2_max_mv_length_horizontal: bit_reader.read_exp_golomb()?,
                log2_max_mv_length_vertical: bit_reader.read_exp_golomb()?,
            });
        }

        Ok(Self {
            aspect_ratio_info,
            overscan_appropriate_flag,
            video_signal_type,
            chroma_loc_info,
            neutral_chroma_indication_flag,
            field_seq_flag,
            frame_field_info_present_flag,
            default_display_window,
            vui_timing_info,
            bitstream_restriction,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AspectRatioInfo {
    Predefined(AspectRatioIdc),
    ExtendedSar { sar_width: u16, sar_height: u16 },
}

/// The color config for SPS.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoSignalType {
    /// The `video_format` bits as a u8.
    pub video_format: VideoFormat,
    /// The `video_full_range_flag` as a bool.
    pub video_full_range_flag: bool,
    /// The `colour_primaries` bits as a u8.
    pub color_primaries: u8,
    /// The `transfer_characteristics` bits as a u8.
    pub transfer_characteristics: u8,
    /// The `matrix_coeffs` bits as a u8.
    pub matrix_coeffs: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaLocInfo {
    /// `chroma_sample_loc_type_top_field`
    pub top_field: u64,
    /// `chroma_sample_loc_type_bottom_field`
    pub bottom_field: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDisplayWindow {
    pub left_offset: u64,
    pub right_offset: u64,
    pub top_offset: u64,
    pub bottom_offset: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VuiTimingInfo {
    pub num_units_in_tick: u32,
    pub time_scale: u32,
    pub num_ticks_poc_diff_one_minus1: Option<u64>,
    pub hrd_parameters: Option<HrdParameters>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitStreamRestriction {
    pub tiles_fixed_structure_flag: bool,
    pub motion_vectors_over_pic_boundaries_flag: bool,
    pub restricted_ref_pic_lists_flag: bool,
    pub min_spatial_segmentation_idc: u64,
    pub max_bytes_per_pic_denom: u64,
    pub max_bits_per_min_cu_denom: u64,
    pub log2_max_mv_length_horizontal: u64,
    pub log2_max_mv_length_vertical: u64,
}
