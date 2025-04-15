use std::io;
use std::num::NonZero;

use byteorder::{BigEndian, ReadBytesExt};
use hrd_parameters::HrdParameters;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use super::ConformanceWindow;
use crate::range_check::range_check;
use crate::{AspectRatioIdc, VideoFormat};

mod hrd_parameters;

#[derive(Debug, Clone, PartialEq)]
pub struct VuiParameters {
    pub aspect_ratio_info: AspectRatioInfo,
    pub overscan_appropriate_flag: Option<bool>,
    pub video_signal_type: VideoSignalType,
    pub chroma_loc_info: Option<ChromaLocInfo>,
    pub neutral_chroma_indication_flag: bool,
    pub field_seq_flag: bool,
    pub frame_field_info_present_flag: bool,
    pub default_display_window: DefaultDisplayWindow,
    pub vui_timing_info: Option<VuiTimingInfo>,
    pub bitstream_restriction: BitStreamRestriction,
}

impl VuiParameters {
    // TODO: Find a solution for this
    #[allow(clippy::too_many_arguments)]
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        sps_max_sub_layers_minus1: u8,
        bit_depth_y: u8,
        bit_depth_c: u8,
        chroma_format_idc: u8,
        general_frame_only_constraint_flag: bool,
        general_progressive_source_flag: bool,
        general_interlaced_source_flag: bool,
        conformance_window: &ConformanceWindow,
        sub_width_c: u8,
        pic_width_in_luma_samples: u64,
        sub_height_c: u8,
        pic_height_in_luma_samples: u64,
    ) -> io::Result<Self> {
        let mut aspect_ratio_info = AspectRatioInfo::Predefined(AspectRatioIdc::Unspecified);
        let mut overscan_appropriate_flag = None;
        let mut video_signal_type = None;
        let mut chroma_loc_info = None;
        let mut default_display_window = None;
        let mut vui_timing_info = None;

        let aspect_ratio_info_present_flag = bit_reader.read_bit()?;
        if aspect_ratio_info_present_flag {
            let aspect_ratio_idc = bit_reader.read_u8()?;
            if aspect_ratio_idc == AspectRatioIdc::ExtendedSar.0 {
                let sar_width = bit_reader.read_u16::<BigEndian>()?;
                let sar_height = bit_reader.read_u16::<BigEndian>()?;
                aspect_ratio_info = AspectRatioInfo::ExtendedSar { sar_width, sar_height };
            } else {
                aspect_ratio_info = AspectRatioInfo::Predefined(aspect_ratio_idc.into());
            }
        }

        let overscan_info_present_flag = bit_reader.read_bit()?;
        if overscan_info_present_flag {
            overscan_appropriate_flag = Some(bit_reader.read_bit()?);
        }

        let video_signal_type_present_flag = bit_reader.read_bit()?;
        if video_signal_type_present_flag {
            let video_format = VideoFormat::from(bit_reader.read_bits(3)? as u8);
            let video_full_range_flag = bit_reader.read_bit()?;
            let colour_description_present_flag = bit_reader.read_bit()?;

            if colour_description_present_flag {
                let color_primaries = bit_reader.read_u8()?;
                let transfer_characteristics = bit_reader.read_u8()?;
                let matrix_coeffs = bit_reader.read_u8()?;

                if matrix_coeffs == 0 && !(bit_depth_c == bit_depth_y && chroma_format_idc == 3) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "matrix_coeffs must not be 0 unless bit_depth_c == bit_depth_y and chroma_format_idc == 3",
                    ));
                }

                if matrix_coeffs == 8
                    && !(bit_depth_c == bit_depth_y || (bit_depth_c == bit_depth_y + 1 && chroma_format_idc == 3))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "matrix_coeffs must not be 8 unless bit_depth_c == bit_depth_y or (bit_depth_c == bit_depth_y + 1 and chroma_format_idc == 3)",
                    ));
                }

                video_signal_type = Some(VideoSignalType {
                    video_format,
                    video_full_range_flag,
                    color_primaries,
                    transfer_characteristics,
                    matrix_coeffs,
                });
            } else {
                video_signal_type = Some(VideoSignalType {
                    video_format,
                    video_full_range_flag,
                    ..Default::default()
                });
            }
        }

        let chroma_loc_info_present_flag = bit_reader.read_bit()?;

        if chroma_format_idc != 1 && chroma_loc_info_present_flag {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "chroma_loc_info_present_flag must be 0 if chroma_format_idc != 1",
            ));
        }

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

        if general_frame_only_constraint_flag && field_seq_flag {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "field_seq_flag must be 0 if general_frame_only_constraint_flag is 1",
            ));
        }

        let frame_field_info_present_flag = bit_reader.read_bit()?;

        if !frame_field_info_present_flag
            && (field_seq_flag || (general_progressive_source_flag && general_interlaced_source_flag))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame_field_info_present_flag must be 1 if field_seq_flag is 1 or general_progressive_source_flag and general_interlaced_source_flag are both 1",
            ));
        }

        let default_display_window_flag = bit_reader.read_bit()?;
        if default_display_window_flag {
            let def_disp_win_left_offset = bit_reader.read_exp_golomb()?;
            let def_disp_win_right_offset = bit_reader.read_exp_golomb()?;
            let def_disp_win_top_offset = bit_reader.read_exp_golomb()?;
            let def_disp_win_bottom_offset = bit_reader.read_exp_golomb()?;
            let left_offset = conformance_window.conf_win_left_offset + def_disp_win_left_offset;
            let right_offset = conformance_window.conf_win_right_offset + def_disp_win_right_offset;
            let top_offset = conformance_window.conf_win_top_offset + def_disp_win_top_offset;
            let bottom_offset = conformance_window.conf_win_bottom_offset + def_disp_win_bottom_offset;

            if sub_width_c as u64 * (left_offset + right_offset) >= pic_width_in_luma_samples {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "sub_width_c * (left_offset + right_offset) must be less than pic_width_in_luma_samples",
                ));
            }

            if sub_height_c as u64 * (top_offset + bottom_offset) >= pic_height_in_luma_samples {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "sub_height_c * (top_offset + bottom_offset) must be less than pic_height_in_luma_samples",
                ));
            }

            default_display_window = Some(DefaultDisplayWindow {
                def_disp_win_left_offset,
                def_disp_win_right_offset,
                def_disp_win_top_offset,
                def_disp_win_bottom_offset,
                left_offset,
                right_offset,
                top_offset,
                bottom_offset,
            });
        }

        let vui_timing_info_present_flag = bit_reader.read_bit()?;
        if vui_timing_info_present_flag {
            let vui_num_units_in_tick = bit_reader.read_u32::<BigEndian>()?;
            let vui_time_scale = NonZero::new(bit_reader.read_u32::<BigEndian>()?)
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "vui_time_scale must not be zero"))?;

            let mut num_ticks_poc_diff_one_minus1 = None;
            let vui_poc_proportional_to_timing_flag = bit_reader.read_bit()?;
            if vui_poc_proportional_to_timing_flag {
                let vui_num_ticks_poc_diff_one_minus1 = bit_reader.read_exp_golomb()?;
                range_check!(vui_num_ticks_poc_diff_one_minus1, 0, 2u64.pow(32) - 2)?;
                num_ticks_poc_diff_one_minus1 = Some(vui_num_ticks_poc_diff_one_minus1 as u32);
            }

            let mut vui_hrd_parameters = None;
            let vui_hrd_parameters_present_flag = bit_reader.read_bit()?;
            if vui_hrd_parameters_present_flag {
                vui_hrd_parameters = Some(HrdParameters::parse(bit_reader, true, sps_max_sub_layers_minus1)?);
            }

            vui_timing_info = Some(VuiTimingInfo {
                num_units_in_tick: vui_num_units_in_tick,
                time_scale: vui_time_scale,
                num_ticks_poc_diff_one_minus1,
                hrd_parameters: vui_hrd_parameters,
            });
        }

        let mut bitstream_restriction = BitStreamRestriction::default();
        let bitstream_restriction_flag = bit_reader.read_bit()?;
        if bitstream_restriction_flag {
            bitstream_restriction.tiles_fixed_structure_flag = bit_reader.read_bit()?;
            bitstream_restriction.motion_vectors_over_pic_boundaries_flag = bit_reader.read_bit()?;
            bitstream_restriction.restricted_ref_pic_lists_flag = Some(bit_reader.read_bit()?);

            let min_spatial_segmentation_idc = bit_reader.read_exp_golomb()?;
            range_check!(min_spatial_segmentation_idc, 0, 4095)?;
            bitstream_restriction.min_spatial_segmentation_idc = min_spatial_segmentation_idc as u16;

            let max_bytes_per_pic_denom = bit_reader.read_exp_golomb()?;
            range_check!(max_bytes_per_pic_denom, 0, 16)?;
            bitstream_restriction.max_bytes_per_pic_denom = max_bytes_per_pic_denom as u8;

            let max_bits_per_min_cu_denom = bit_reader.read_exp_golomb()?;
            range_check!(max_bits_per_min_cu_denom, 0, 16)?;
            bitstream_restriction.max_bits_per_min_cu_denom = max_bits_per_min_cu_denom as u8;

            let log2_max_mv_length_horizontal = bit_reader.read_exp_golomb()?;
            range_check!(log2_max_mv_length_horizontal, 0, 15)?;
            bitstream_restriction.log2_max_mv_length_horizontal = log2_max_mv_length_horizontal as u8;

            let log2_max_mv_length_vertical = bit_reader.read_exp_golomb()?;
            range_check!(log2_max_mv_length_vertical, 0, 15)?;
            bitstream_restriction.log2_max_mv_length_vertical = log2_max_mv_length_vertical as u8;
        }

        Ok(Self {
            aspect_ratio_info,
            overscan_appropriate_flag,
            video_signal_type: video_signal_type.unwrap_or_default(),
            chroma_loc_info,
            neutral_chroma_indication_flag,
            field_seq_flag,
            frame_field_info_present_flag,
            default_display_window: default_display_window.unwrap_or_default(),
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

impl Default for VideoSignalType {
    fn default() -> Self {
        Self {
            video_format: VideoFormat::Unspecified,
            video_full_range_flag: false,
            color_primaries: 2,
            transfer_characteristics: 2,
            matrix_coeffs: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaLocInfo {
    /// `chroma_sample_loc_type_top_field`
    pub top_field: u64,
    /// `chroma_sample_loc_type_bottom_field`
    pub bottom_field: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DefaultDisplayWindow {
    pub def_disp_win_left_offset: u64,
    pub def_disp_win_right_offset: u64,
    pub def_disp_win_top_offset: u64,
    pub def_disp_win_bottom_offset: u64,
    // Calculated values
    left_offset: u64,
    right_offset: u64,
    top_offset: u64,
    bottom_offset: u64,
}

impl DefaultDisplayWindow {
    #[inline]
    pub fn left_offset(&self) -> u64 {
        self.left_offset
    }

    #[inline]
    pub fn right_offset(&self) -> u64 {
        self.right_offset
    }

    #[inline]
    pub fn top_offset(&self) -> u64 {
        self.top_offset
    }

    #[inline]
    pub fn bottom_offset(&self) -> u64 {
        self.bottom_offset
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VuiTimingInfo {
    pub num_units_in_tick: u32,
    pub time_scale: NonZero<u32>,
    pub num_ticks_poc_diff_one_minus1: Option<u32>,
    pub hrd_parameters: Option<HrdParameters>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitStreamRestriction {
    pub tiles_fixed_structure_flag: bool,
    pub motion_vectors_over_pic_boundaries_flag: bool,
    pub restricted_ref_pic_lists_flag: Option<bool>,
    pub min_spatial_segmentation_idc: u16,
    pub max_bytes_per_pic_denom: u8,
    pub max_bits_per_min_cu_denom: u8,
    pub log2_max_mv_length_horizontal: u8,
    pub log2_max_mv_length_vertical: u8,
}

impl Default for BitStreamRestriction {
    fn default() -> Self {
        Self {
            tiles_fixed_structure_flag: false,
            motion_vectors_over_pic_boundaries_flag: true,
            restricted_ref_pic_lists_flag: None,
            min_spatial_segmentation_idc: 0,
            max_bytes_per_pic_denom: 2,
            max_bits_per_min_cu_denom: 1,
            log2_max_mv_length_horizontal: 15,
            log2_max_mv_length_vertical: 15,
        }
    }
}

impl BitStreamRestriction {
    pub fn min_spatial_segmentation_times4(&self) -> u16 {
        self.min_spatial_segmentation_idc + 4
    }
}
