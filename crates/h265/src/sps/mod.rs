use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;
use scuffle_h264::EmulationPreventionIo;

use crate::NALUnitType;
use crate::nal_unit_header::NALUnitHeader;
use crate::range_check::range_check;

mod conformance_window;
mod pcm;
mod profile_tier_level;
mod scaling_list;
mod sps_3d_extension;
mod sps_multilayer_extension;
mod sps_range_extension;
mod sps_scc_extension;
mod st_ref_pic_set;
mod sub_layer_ordering_info;
mod vui_parameters;

pub use conformance_window::ConformanceWindow;
pub use pcm::Pcm;
pub use profile_tier_level::ProfileTierLevel;
pub use scaling_list::ScalingListData;
pub use sps_3d_extension::Sps3dExtension;
pub use sps_multilayer_extension::SpsMultilayerExtension;
pub use sps_range_extension::SpsRangeExtension;
pub use sps_scc_extension::SpsSccExtension;
pub use st_ref_pic_set::ShortTermRefPicSets;
pub use sub_layer_ordering_info::SubLayerOrderingInfo;
pub use vui_parameters::VuiParameters;

/// The Sequence Parameter Set.
///
/// ISO/IEC-14496-10-2022 - 7.3.2.2
#[derive(Debug, Clone, PartialEq)]
pub struct Sps {
    /// The NAL unit header.
    pub nal_unit_header: NALUnitHeader,

    /// The `sps_video_parameter_set_id` is 4 bits, and is the value of the
    /// `vps_video_parameter_set_id` of the active VPS.
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub sps_video_parameter_set_id: u8,

    /// The `sps_max_sub_layers_minus1` is 3 bits, where the value plus 1 is the max number of temporal
    /// sub-layers that might be in each CVS referring to the SPS.
    ///
    /// The value ranges from \[0, 6\]. The value must be less than or equal to `vps_max_sub_layers_minus1`.
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub sps_max_sub_layers_minus1: u8,

    /// The `sps_temporal_id_nesting_flag` is a single bit.
    ///
    /// When `sps_max_sub_layers_minus1 > 0`, this means inter-prediction is restricted for
    /// CVSs that refer to the Sps.
    ///
    /// When `vps_temporal_id_nesting_flag == 1`, this flag is 1.
    ///
    /// When `sps_max_sub_layers_minus1 == 0`, this flag is 1.
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub sps_temporal_id_nesting_flag: bool,

    pub profile_tier_level: ProfileTierLevel,

    /// The `sps_seq_parameter_set_id` is an id for the SPS.
    ///
    /// The value of this ranges from \[0, 15\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `15` which is encoded as `0 0001 0000`, which is 9 bits.
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub sps_seq_parameter_set_id: u64,

    /// The `chroma_format_idc` is the chroma sampling relative to the luma sampling from 6.2.
    ///
    /// The value of this ranges from \[0, 3\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `3` which is encoded as `0 0100`, which is 5 bits.
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub chroma_format_idc: u8,

    /// The `separate_colour_plane_flag` is a single bit.
    ///
    /// 0 means the the color components aren't coded separately and `ChromaArrayType` is set to `chroma_format_idc`.
    ///
    /// 1 means the 3 color components of the 4:4:4 chroma format are coded separately and
    /// `ChromaArrayType` is set to 0.
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub separate_color_plane_flag: bool,

    /// The `pic_width_in_luma_samples` is the width of each decoded picture in units of luma samples.
    ///
    /// The value cannot be 0 and must be an integer multiple of `MinCbSizeY`.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub pic_width_in_luma_samples: u64,

    /// The `pic_height_in_luma_samples` is the height of each decoded picture in units of luma samples.
    ///
    /// The value cannot be 0 and must be an integer multiple of `MinCbSizeY`.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub pic_height_in_luma_samples: u64,

    /// The [`ConformanceWindow`].
    pub conformance_window: ConformanceWindow,

    /// The `bit_depth_luma_minus8` defines the BitDepth_Y and QpBdOffset_Y as:
    ///
    /// `BitDepth_Y = 8 + bit_depth_luma_minus8`
    ///
    /// and
    ///
    /// `QpBdOffset_Y = 6 * bit_depth_luma_minus8`
    ///
    /// The value of this ranges from \[0, 8\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `8` which is encoded as `000 1001`, which is 7 bits.
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub bit_depth_luma_minus8: u8,

    /// The `bit_depth_luma_minus8` defines the BitDepth_C and QpBdOffset_C as:
    ///
    /// `BitDepth_C = 8 + bit_depth_chroma_minus8`
    ///
    /// and
    ///
    /// `QpBdOffset_C = 6 * bit_depth_chroma_minus8`
    ///
    /// The value of this ranges from \[0, 8\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `8` which is encoded as `000 1001`, which is 7 bits.
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub bit_depth_chroma_minus8: u8,

    /// The `log2_max_pic_order_cnt_lsb_minus4` defines the MaxPicOrderCntLsb as:
    ///
    /// `MaxPicOrderCntLsb = 2^(log2_max_pic_order_cnt_lsb_minus4 + 4)`
    ///
    /// The value of this ranges from \[0, 12\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `12` which is encoded as `000 1101`, which is 7 bits.
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub log2_max_pic_order_cnt_lsb_minus4: u8,

    pub sub_layer_ordering_info: SubLayerOrderingInfo,

    /// The `log2_min_luma_coding_block_size_minus3` plus 3defines the minimum luma coding block size.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub log2_min_luma_coding_block_size_minus3: u64,

    /// The `log2_diff_max_min_luma_coding_block_size` defines the difference between the maximum and minimum luma coding block size.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub log2_diff_max_min_luma_coding_block_size: u64,

    /// The `log2_min_luma_transform_block_size_minus2` plus 2 defines the minimum luma transform block size.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub log2_min_luma_transform_block_size_minus2: u64,

    /// The `log2_diff_max_min_luma_transform_block_size` defines the difference between the maximum and minimum luma transform block size.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub log2_diff_max_min_luma_transform_block_size: u64,

    /// The `max_transform_hierarchy_depth_inter` defines the maximum transform hierarchy depth for inter prediction.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub max_transform_hierarchy_depth_inter: u64,

    /// The `max_transform_hierarchy_depth_intra` defines the maximum transform hierarchy depth for intra prediction.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub max_transform_hierarchy_depth_intra: u64,

    pub scaling_list_data: Option<ScalingListData>,

    pub amp_enabled_flag: bool,
    pub sample_adaptive_offset_enabled_flag: bool,

    pub pcm: Option<Pcm>,

    pub short_term_ref_pic_sets: ShortTermRefPicSets,
    pub long_term_ref_pics_present_flag: bool,

    pub sps_temporal_mvp_enabled_flag: bool,
    pub strong_intra_smoothing_enabled_flag: bool,

    pub vui_parameters: Option<VuiParameters>,

    pub range_extension: Option<SpsRangeExtension>,
    pub multilayer_extension: Option<SpsMultilayerExtension>,
    pub sps_3d_extension: Option<Sps3dExtension>,
    pub scc_extension: Option<SpsSccExtension>,

    // Calculated fields
    sub_width_c: u8,
    sub_height_c: u8,
    bit_depth_y: u8,
    bit_depth_c: u8,
    min_cb_log2_size_y: u64,
    ctb_log2_size_y: u64,
    min_tb_log2_size_y: u64,
}

impl Sps {
    /// Parses an SPS from the input bytes.
    ///
    /// Returns an [`Sps`] struct.
    pub fn parse(reader: impl io::Read) -> io::Result<Self> {
        let mut bit_reader = BitReader::new(reader);

        let nal_unit_header = NALUnitHeader::parse(&mut bit_reader)?;
        if nal_unit_header.nal_unit_type != NALUnitType::SpsNut {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "nal_unit_type is not SPS_NUT"));
        }

        // begin ISO/IEC-23008-2-2020 - 7.3.2.2.1
        // semantics in ISO/IEC-23008-2-2020 - 7.4.3.2.1
        let sps_video_parameter_set_id = bit_reader.read_bits(4)? as u8;

        let sps_max_sub_layers_minus1 = bit_reader.read_bits(3)? as u8;
        range_check!(sps_max_sub_layers_minus1, 0, 6)?;

        let sps_temporal_id_nesting_flag = bit_reader.read_bit()?;

        if sps_max_sub_layers_minus1 == 0 && !sps_temporal_id_nesting_flag {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "sps_temporal_id_nesting_flag must be 1 when sps_max_sub_layers_minus1 is 0",
            ));
        }

        // ISO/IEC-23008-2-2020 - 7.3.3
        let profile_tier_level = ProfileTierLevel::parse(&mut bit_reader, sps_max_sub_layers_minus1)?;

        // back to ISO/IEC-23008-2-2020 - 7.3.2.2.1
        let sps_seq_parameter_set_id = bit_reader.read_exp_golomb()?;
        range_check!(sps_seq_parameter_set_id, 0, 15)?;

        let chroma_format_idc = bit_reader.read_exp_golomb()?;
        range_check!(chroma_format_idc, 0, 3)?;
        let chroma_format_idc = chroma_format_idc as u8;

        let mut separate_color_plane_flag = false;
        if chroma_format_idc == 3 {
            separate_color_plane_flag = bit_reader.read_bit()?;
        }

        let sub_width_c = if chroma_format_idc == 1 || chroma_format_idc == 2 {
            2
        } else {
            1
        };
        let sub_height_c = if chroma_format_idc == 1 { 2 } else { 1 };

        let pic_width_in_luma_samples = bit_reader.read_exp_golomb()?;
        if pic_width_in_luma_samples == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pic_width_in_luma_samples must not be 0",
            ));
        }

        let pic_height_in_luma_samples = bit_reader.read_exp_golomb()?;
        if pic_height_in_luma_samples == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pic_height_in_luma_samples must not be 0",
            ));
        }

        let conformance_window_flag = bit_reader.read_bit()?;

        let conformance_window = conformance_window_flag
            .then(|| ConformanceWindow::parse(&mut bit_reader))
            .transpose()?
            .unwrap_or_default();

        let bit_depth_luma_minus8 = bit_reader.read_exp_golomb()?;
        range_check!(bit_depth_luma_minus8, 0, 8)?;
        let bit_depth_luma_minus8 = bit_depth_luma_minus8 as u8;
        let bit_depth_y = 8 + bit_depth_luma_minus8; // BitDepth_Y
        let bit_depth_chroma_minus8 = bit_reader.read_exp_golomb()?;
        range_check!(bit_depth_chroma_minus8, 0, 8)?;
        let bit_depth_chroma_minus8 = bit_depth_chroma_minus8 as u8;
        let bit_depth_c = 8 + bit_depth_chroma_minus8; // BitDepth_C

        let log2_max_pic_order_cnt_lsb_minus4 = bit_reader.read_exp_golomb()?;
        range_check!(log2_max_pic_order_cnt_lsb_minus4, 0, 12)?;
        let log2_max_pic_order_cnt_lsb_minus4 = log2_max_pic_order_cnt_lsb_minus4 as u8;

        let sps_sub_layer_ordering_info_present_flag = bit_reader.read_bit()?;
        let sub_layer_ordering_info = SubLayerOrderingInfo::parse(
            &mut bit_reader,
            sps_sub_layer_ordering_info_present_flag,
            sps_max_sub_layers_minus1,
        )?;

        let log2_min_luma_coding_block_size_minus3 = bit_reader.read_exp_golomb()?;
        let log2_diff_max_min_luma_coding_block_size = bit_reader.read_exp_golomb()?;

        let min_cb_log2_size_y = log2_min_luma_coding_block_size_minus3 + 3;
        let ctb_log2_size_y = min_cb_log2_size_y + log2_diff_max_min_luma_coding_block_size;

        let log2_min_luma_transform_block_size_minus2 = bit_reader.read_exp_golomb()?;

        let min_tb_log2_size_y = log2_min_luma_transform_block_size_minus2 + 2;

        let log2_diff_max_min_luma_transform_block_size = bit_reader.read_exp_golomb()?;
        let max_transform_hierarchy_depth_inter = bit_reader.read_exp_golomb()?;
        range_check!(max_transform_hierarchy_depth_inter, 0, ctb_log2_size_y - min_tb_log2_size_y)?;
        let max_transform_hierarchy_depth_intra = bit_reader.read_exp_golomb()?;
        range_check!(max_transform_hierarchy_depth_intra, 0, ctb_log2_size_y - min_tb_log2_size_y)?;

        let scaling_list_enabled_flag = bit_reader.read_bit()?;

        let mut scaling_list_data = None;
        if scaling_list_enabled_flag {
            let sps_scaling_list_data_present_flag = bit_reader.read_bit()?;

            if sps_scaling_list_data_present_flag {
                scaling_list_data = Some(ScalingListData::parse(&mut bit_reader)?);
            }
        }

        let amp_enabled_flag = bit_reader.read_bit()?;
        let sample_adaptive_offset_enabled_flag = bit_reader.read_bit()?;

        let mut pcm = None;
        let pcm_enabled_flag = bit_reader.read_bit()?;
        if pcm_enabled_flag {
            pcm = Some(Pcm::parse(
                &mut bit_reader,
                bit_depth_y,
                bit_depth_c,
                min_cb_log2_size_y,
                ctb_log2_size_y,
            )?);
        }

        let num_short_term_ref_pic_sets = bit_reader.read_exp_golomb()?;
        range_check!(num_short_term_ref_pic_sets, 0, 64)?;
        let num_short_term_ref_pic_sets = num_short_term_ref_pic_sets as u8;
        let short_term_ref_pic_sets = ShortTermRefPicSets::parse(&mut bit_reader, num_short_term_ref_pic_sets as usize)?;

        let long_term_ref_pics_present_flag = bit_reader.read_bit()?;
        if long_term_ref_pics_present_flag {
            let num_long_term_ref_pics_sps = bit_reader.read_exp_golomb()?;
            range_check!(num_long_term_ref_pics_sps, 0, 32)?;
            for _ in 0..num_long_term_ref_pics_sps {
                bit_reader.read_bits(log2_max_pic_order_cnt_lsb_minus4 + 4)?; // lt_ref_pic_poc_lsb_sps
                bit_reader.read_bits(1)?; // used_by_curr_pic_lt_sps_flag
            }
        }

        let sps_temporal_mvp_enabled_flag = bit_reader.read_bit()?;
        let strong_intra_smoothing_enabled_flag = bit_reader.read_bit()?;

        let mut vui_parameters = None;
        let vui_parameters_present_flag = bit_reader.read_bit()?;
        if vui_parameters_present_flag {
            vui_parameters = Some(VuiParameters::parse(
                &mut bit_reader,
                sps_max_sub_layers_minus1,
                bit_depth_y,
                bit_depth_c,
                chroma_format_idc,
                profile_tier_level.general_profile.frame_only_constraint_flag,
                profile_tier_level.general_profile.progressive_source_flag,
                profile_tier_level.general_profile.interlaced_source_flag,
                &conformance_window,
                sub_width_c,
                pic_width_in_luma_samples,
                sub_height_c,
                pic_height_in_luma_samples,
            )?);
        }

        // Extensions
        let mut range_extension = None;
        let mut multilayer_extension = None;
        let mut sps_3d_extension = None;
        let mut scc_extension = None;

        let sps_extension_flag = bit_reader.read_bit()?;
        if sps_extension_flag {
            let sps_range_extension_flag = bit_reader.read_bit()?;
            let sps_multilayer_extension_flag = bit_reader.read_bit()?;
            let sps_3d_extension_flag = bit_reader.read_bit()?;
            let sps_scc_extension_flag = bit_reader.read_bit()?;
            bit_reader.read_bits(4)?; // sps_extension_4bits

            if sps_range_extension_flag {
                range_extension = Some(SpsRangeExtension::parse(&mut bit_reader)?);
            }

            if sps_multilayer_extension_flag {
                multilayer_extension = Some(SpsMultilayerExtension::parse(&mut bit_reader)?);
            }

            if sps_3d_extension_flag {
                sps_3d_extension = Some(Sps3dExtension::parse(&mut bit_reader, min_cb_log2_size_y, ctb_log2_size_y)?);
            }

            if sps_scc_extension_flag {
                scc_extension = Some(SpsSccExtension::parse(
                    &mut bit_reader,
                    chroma_format_idc,
                    bit_depth_y,
                    bit_depth_c,
                )?);
            }

            // Ignore sps_extension_data_flag as specified by 7.4.3.2.1, page 101
        }

        bit_reader.align()?;

        // Skip to the end or rbsp_trailing_bits()
        while match bit_reader.read_u8() {
            Ok(byte) => byte & 0b0000_0001 == 0,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => false,
            Err(e) => return Err(e),
        } {}

        Ok(Sps {
            nal_unit_header,
            sps_video_parameter_set_id,
            sps_max_sub_layers_minus1,
            sps_temporal_id_nesting_flag,
            profile_tier_level,
            sps_seq_parameter_set_id,
            chroma_format_idc,
            separate_color_plane_flag,
            pic_width_in_luma_samples,
            pic_height_in_luma_samples,
            conformance_window,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            log2_max_pic_order_cnt_lsb_minus4,
            sub_layer_ordering_info,
            log2_min_luma_coding_block_size_minus3,
            log2_diff_max_min_luma_coding_block_size,
            log2_min_luma_transform_block_size_minus2,
            log2_diff_max_min_luma_transform_block_size,
            max_transform_hierarchy_depth_inter,
            max_transform_hierarchy_depth_intra,
            scaling_list_data,
            amp_enabled_flag,
            sample_adaptive_offset_enabled_flag,
            pcm,
            short_term_ref_pic_sets,
            long_term_ref_pics_present_flag,
            sps_temporal_mvp_enabled_flag,
            strong_intra_smoothing_enabled_flag,
            vui_parameters,
            range_extension,
            multilayer_extension,
            sps_3d_extension,
            scc_extension,
            // Calculated fields
            sub_width_c,
            sub_height_c,
            bit_depth_y,
            bit_depth_c,
            min_cb_log2_size_y,
            ctb_log2_size_y,
            min_tb_log2_size_y,
        })
    }

    pub fn parse_with_emulation_prevention(reader: impl io::Read) -> io::Result<Self> {
        Self::parse(EmulationPreventionIo::new(reader))
    }

    /// The height as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `height = pic_height_in_luma_samples - sub_height_c * (conf_win_top_offset + conf_win_bottom_offset)`
    pub fn height(&self) -> u64 {
        self.pic_height_in_luma_samples
            - self.sub_height_c() as u64
                * (self.conformance_window.conf_win_top_offset + self.conformance_window.conf_win_bottom_offset)
    }

    /// The width as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `width = pic_width_in_luma_samples - sub_width_c * (conf_win_left_offset + conf_win_right_offset)`
    pub fn width(&self) -> u64 {
        self.pic_width_in_luma_samples
            - self.sub_width_c() as u64
                * (self.conformance_window.conf_win_left_offset + self.conformance_window.conf_win_right_offset)
    }

    pub fn chroma_array_type(&self) -> u8 {
        if self.separate_color_plane_flag {
            0
        } else {
            self.chroma_format_idc
        }
    }

    #[inline]
    pub fn sub_width_c(&self) -> u8 {
        self.sub_width_c
    }

    #[inline]
    pub fn sub_height_c(&self) -> u8 {
        self.sub_height_c
    }

    #[inline]
    pub fn bit_depth_y(&self) -> u8 {
        self.bit_depth_y
    }

    pub fn qp_bd_offset_y(&self) -> u8 {
        6 * self.bit_depth_y
    }

    #[inline]
    pub fn bit_depth_c(&self) -> u8 {
        self.bit_depth_c
    }

    pub fn qp_bd_offset_c(&self) -> u8 {
        6 * self.bit_depth_c
    }

    pub fn max_pic_order_cnt_lsb(&self) -> u32 {
        2u32.pow(self.log2_max_pic_order_cnt_lsb_minus4 as u32 + 4)
    }

    #[inline]
    pub fn min_cb_log2_size_y(&self) -> u64 {
        self.min_cb_log2_size_y
    }

    #[inline]
    pub fn ctb_log2_size_y(&self) -> u64 {
        self.ctb_log2_size_y
    }

    pub fn min_cb_size_y(&self) -> u64 {
        1 << self.min_cb_log2_size_y()
    }

    pub fn ctb_size_y(&self) -> u64 {
        1 << self.ctb_log2_size_y()
    }

    pub fn pic_width_in_min_cbs_y(&self) -> f64 {
        self.pic_width_in_luma_samples as f64 / self.min_cb_size_y() as f64
    }

    pub fn pic_width_in_ctbs_y(&self) -> u64 {
        (self.pic_width_in_luma_samples / self.ctb_size_y()) + 1
    }

    pub fn pic_height_in_min_cbs_y(&self) -> f64 {
        self.pic_height_in_luma_samples as f64 / self.min_cb_size_y() as f64
    }

    pub fn pic_height_in_ctbs_y(&self) -> u64 {
        (self.pic_height_in_luma_samples / self.ctb_size_y()) + 1
    }

    pub fn pic_size_in_min_cbs_y(&self) -> f64 {
        self.pic_width_in_min_cbs_y() * self.pic_height_in_min_cbs_y()
    }

    pub fn pic_size_in_ctbs_y(&self) -> u64 {
        self.pic_width_in_ctbs_y() * self.pic_height_in_ctbs_y()
    }

    pub fn pic_size_in_samples_y(&self) -> u64 {
        self.pic_width_in_luma_samples * self.pic_height_in_luma_samples
    }

    pub fn pic_width_in_samples_c(&self) -> u64 {
        self.pic_width_in_luma_samples / self.sub_width_c() as u64
    }

    pub fn pic_height_in_samples_c(&self) -> u64 {
        self.pic_height_in_luma_samples / self.sub_height_c() as u64
    }

    pub fn ctb_width_c(&self) -> u64 {
        if self.chroma_format_idc == 0 || self.separate_color_plane_flag {
            0
        } else {
            self.ctb_size_y() / self.sub_width_c() as u64
        }
    }

    pub fn ctb_height_c(&self) -> u64 {
        if self.chroma_format_idc == 0 || self.separate_color_plane_flag {
            0
        } else {
            self.ctb_size_y() / self.sub_height_c() as u64
        }
    }

    pub fn min_tb_log2_size_y(&self) -> u64 {
        self.min_tb_log2_size_y
    }

    pub fn max_tb_log2_size_y(&self) -> u64 {
        self.log2_min_luma_transform_block_size_minus2 + 2 + self.log2_diff_max_min_luma_transform_block_size
    }

    pub fn scan_order(&self) {
        todo!()
    }

    pub fn raw_ctu_bits(&self) -> u64 {
        // defined by A-1
        self.ctb_size_y() * self.ctb_size_y() * self.bit_depth_y() as u64
            + 2 * (self.ctb_width_c() * self.ctb_height_c()) * self.bit_depth_c() as u64
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use crate::Sps;

    #[test]
    fn test_sps_parse() {
        let data = b"B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\x80\x80\x82\0\0\x03\0\x02\0\0\x03\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 2560);
        assert_eq!(sps.height(), 1440);
        insta::assert_debug_snapshot!(sps);
    }

    #[test]
    fn test_sps_parse2() {
        // This is a real SPS from an mp4 video file recorded with OBS.
        let data = b"\x42\x01\x01\x01\x40\x00\x00\x03\x00\x90\x00\x00\x03\x00\x00\x03\x00\x78\xa0\x03\xc0\x80\x11\x07\xcb\x96\xb4\xa4\x25\x92\xe3\x01\x6a\x02\x02\x02\x08\x00\x00\x03\x00\x08\x00\x00\x03\x00\xf3\x00\x2e\xf2\x88\x00\x02\x62\x5a\x00\x00\x13\x12\xd0\x20";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 1920);
        assert_eq!(sps.height(), 1080);
        insta::assert_debug_snapshot!(sps);
    }

    #[test]
    fn test_sps_parse3() {
        // This is a real SPS from here: https://kodi.wiki/view/Samples
        let data = b"\x42\x01\x01\x22\x20\x00\x00\x03\x00\x90\x00\x00\x03\x00\x00\x03\x00\x99\xA0\x01\xE0\x20\x02\x1C\x4D\x8D\x35\x92\x4F\x84\x14\x70\xF1\xC0\x90\x3B\x0E\x18\x36\x1A\x08\x42\xF0\x81\x21\x00\x88\x40\x10\x06\xE1\xA3\x06\xC3\x41\x08\x5C\xA0\xA0\x21\x04\x41\x70\xB0\x2A\x0A\xC2\x80\x35\x40\x70\x80\xE0\x07\xD0\x2B\x41\x80\xA8\x20\x0B\x85\x81\x50\x56\x14\x01\xAA\x03\x84\x07\x00\x3E\x81\x58\xA1\x0D\x35\xE9\xE8\x60\xD7\x43\x03\x41\xB1\xB8\xC0\xD0\x70\x3A\x1B\x1B\x18\x1A\x0E\x43\x21\x30\xC8\x60\x24\x18\x10\x1F\x1F\x1C\x1E\x30\x74\x26\x12\x0E\x0C\x04\x30\x40\x38\x10\x82\x00\x94\x0F\xF0\x86\x9A\xF2\x17\x20\x48\x26\x59\x02\x41\x20\x98\x4F\x09\x04\x83\x81\xD0\x98\x4E\x12\x09\x07\x21\x90\x98\x5C\x2C\x12\x0C\x08\x0F\x8F\x8E\x0F\x18\x3A\x13\x09\x07\x06\x02\x18\x20\x1C\x08\x41\x00\x4A\x07\xF2\x86\x89\x4D\x08\x2C\x83\x8E\x52\x18\x17\x02\xF2\xC8\x0B\x80\xDC\x06\xB0\x5F\x82\xE0\x35\x03\xA0\x66\x06\xB0\x63\x06\x00\x6A\x06\x40\xE0\x0B\x20\x73\x06\x60\xC8\x0E\x40\x58\x03\x90\x0A\xB0\x77\x07\x40\x2A\x81\xC7\xFF\xC1\x24\x34\x49\x8E\x61\x82\x62\x0C\x72\x90\xC0\xB8\x17\x96\x40\x5C\x06\xE0\x35\x82\xFC\x17\x01\xA8\x1D\x03\x30\x35\x83\x18\x30\x03\x50\x32\x07\x00\x59\x03\x98\x33\x06\x40\x72\x02\xC0\x1C\x80\x55\x83\xB8\x3A\x01\x54\x0E\x3F\xFE\x09\x0A\x10\xE9\xAF\x4F\x43\x06\xBA\x18\x1A\x0D\x8D\xC6\x06\x83\x81\xD0\xD8\xD8\xC0\xD0\x72\x19\x09\x86\x43\x01\x20\xC0\x80\xF8\xF8\xE0\xF1\x83\xA1\x30\x90\x70\x60\x21\x82\x01\xC0\x84\x10\x04\xA0\x7F\x84\x3A\x6B\xC8\x5C\x81\x20\x99\x64\x09\x04\x82\x61\x3C\x24\x12\x0E\x07\x42\x61\x38\x48\x24\x1C\x86\x42\x61\x70\xB0\x48\x30\x20\x3E\x3E\x38\x3C\x60\xE8\x4C\x24\x1C\x18\x08\x60\x80\x70\x21\x04\x01\x28\x1F\xCA\x1A\x92\x9A\x10\x59\x07\x1C\xA4\x30\x2E\x05\xE5\x90\x17\x01\xB8\x0D\x60\xBF\x05\xC0\x6A\x07\x40\xCC\x0D\x60\xC6\x0C\x00\xD4\x0C\x81\xC0\x16\x40\xE6\x0C\xC1\x90\x1C\x80\xB0\x07\x20\x15\x60\xEE\x0E\x80\x55\x03\x8F\xFF\x82\x48\x6A\x49\x8E\x61\x82\x62\x0C\x72\x90\xC0\xB8\x17\x96\x40\x5C\x06\xE0\x35\x82\xFC\x17\x01\xA8\x1D\x03\x30\x35\x83\x18\x30\x03\x50\x32\x07\x00\x59\x03\x98\x33\x06\x40\x72\x02\xC0\x1C\x80\x55\x83\xB8\x3A\x01\x54\x0E\x3F\xFE\x09\x0A\x10\xE9\xAF\x4F\x43\x06\xBA\x18\x1A\x0D\x8D\xC6\x06\x83\x81\xD0\xD8\xD8\xC0\xD0\x72\x19\x09\x86\x43\x01\x20\xC0\x80\xF8\xF8\xE0\xF1\x83\xA1\x30\x90\x70\x60\x21\x82\x01\xC0\x84\x10\x04\xA0\x7F\x86\xA4\x98\xE6\x18\x26\x20\xC7\x29\x0C\x0B\x81\x79\x64\x05\xC0\x6E\x03\x58\x2F\xC1\x70\x1A\x81\xD0\x33\x03\x58\x31\x83\x00\x35\x03\x20\x70\x05\x90\x39\x83\x30\x64\x07\x20\x2C\x01\xC8\x05\x58\x3B\x83\xA0\x15\x40\xE3\xFF\xE0\x91\x11\x5C\x96\xA5\xDE\x02\xD4\x24\x40\x26\xD9\x40\x00\x07\xD2\x00\x01\xD4\xC0\x3E\x46\x81\x8D\xC0\x00\x26\x25\xA0\x00\x13\x12\xD0\x00\x04\xC4\xB4\x00\x02\x62\x5A\x8B\x84\x02\x08\xA2\x00\x01\x00\x08\x44\x01\xC1\x72\x43\x8D\x62\x24\x00\x00\x00\x14";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 3840);
        assert_eq!(sps.height(), 2160);
        insta::assert_debug_snapshot!(sps);
    }

    #[test]
    fn test_sps_parse4() {
        // This is a real SPS from here: https://lf-tk-sg.ibytedtos.com/obj/tcs-client-sg/resources/video_demo_hevc.html#main-bt709-sample-5
        let data = b"\x42\x01\x01\x01\x60\x00\x00\x03\x00\x90\x00\x00\x03\x00\x00\x03\x00\xB4\xA0\x00\xF0\x08\x00\x43\x85\x96\x56\x69\x24\xC2\xB0\x16\x80\x80\x00\x00\x03\x00\x80\x00\x00\x05\x04\x22\x00\x01";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 7680);
        assert_eq!(sps.height(), 4320);
        insta::assert_debug_snapshot!(sps);
    }

    #[test]
    fn test_sps_parse5() {
        // This is a real SPS from here: https://lf-tk-sg.ibytedtos.com/obj/tcs-client-sg/resources/video_demo_hevc.html#msp-bt709-sample-1
        let data = b"\x42\x01\x01\x03\x70\x00\x00\x03\x00\x00\x03\x00\x00\x03\x00\x00\x03\x00\x78\xA0\x03\xC0\x80\x10\xE7\xF9\x7E\x49\x1B\x65\xB2\x22\x00\x01\x00\x07\x44\x01\xC1\x90\x95\x81\x12\x00\x00\x00\x14";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 1920);
        assert_eq!(sps.height(), 1080);
        insta::assert_debug_snapshot!(sps);
    }

    #[test]
    fn test_sps_parse6() {
        // This is a real SPS from here: https://lf-tk-sg.ibytedtos.com/obj/tcs-client-sg/resources/video_demo_hevc.html#rext-bt709-sample-1
        let data = b"\x42\x01\x01\x24\x08\x00\x00\x03\x00\x9D\x08\x00\x00\x03\x00\x00\x99\xB0\x01\xE0\x20\x02\x1C\x4D\x94\xD6\xED\xBE\x41\x12\x64\xEB\x25\x11\x44\x1A\x6C\x9D\x64\xA2\x29\x09\x26\xBA\xF5\xFF\xEB\xFA\xFD\x7F\xEB\xF5\x44\x51\x04\x93\x5D\x7A\xFF\xF5\xFD\x7E\xBF\xF5\xFA\xC8\xA4\x92\x4D\x75\xEB\xFF\xD7\xF5\xFA\xFF\xD7\xEA\x88\xA2\x24\x93\x5D\x7A\xFF\xF5\xFD\x7E\xBF\xF5\xFA\xC8\x94\x08\x53\x49\x29\x24\x89\x55\x12\xA5\x2A\x94\xC1\x35\x01\x01\x01\x03\xB8\x40\x20\x80\xA2\x00\x01\x00\x07\x44\x01\xC0\x72\xB0\x3C\x90\x00\x00\x00\x13\x63\x6F\x6C\x72\x6E\x63\x6C\x78\x00\x01\x00\x01\x00\x01\x00\x00\x00\x00\x18";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 3840);
        assert_eq!(sps.height(), 2160);
        insta::assert_debug_snapshot!(sps);
    }

    #[test]
    fn test_forbidden_zero_bit() {
        // 0x80 = 1000 0000: forbidden_zero_bit (first bit) is 1.
        let data = [0x80];
        let err = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "forbidden_zero_bit is not zero");
    }

    #[test]
    fn test_invalid_nalu_type() {
        // 1 forbidden_zero_bit = 0
        // nal_unit_type (100000) = 32 ≠ 33
        // nuh_layer_id (000000) = 0
        // nuh_temporal_id_plus1 (001) = 1
        #[allow(clippy::unusual_byte_groupings)]
        let data = [0b0_100000_0, 0b00000_001];
        let err = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "nal_unit_type is not SPS_NUT");
    }
}
