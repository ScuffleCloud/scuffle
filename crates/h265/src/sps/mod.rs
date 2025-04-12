use std::io;
use std::num::NonZero;

use byteorder::ReadBytesExt;
use conformance_window::ConformanceWindow;
use pcm::Pcm;
use profile_tier_level::ProfileTierLevel;
use scaling_list::ScalingListData;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;
use scuffle_h264::EmulationPreventionIo;
use sps_3d_extension::Sps3dExtension;
use sps_multilayer_extension::SpsMultilayerExtension;
use sps_range_extension::SpsRangeExtension;
use sps_scc_extension::SpsSccExtension;
use st_ref_pic_set::ShortTermRefPicSets;
use sub_layer_ordering_info::SubLayerOrderingInfo;
use vui_parameters::VuiParameters;

use crate::NALUnitType;

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

/// The Sequence Parameter Set.
/// ISO/IEC-14496-10-2022 - 7.3.2.2
#[derive(Debug, Clone, PartialEq)]
pub struct Sps {
    /// The `nuh_layer_id` is 6 bits containing the id of the layer that a non/VCL NAL unit belongs to.
    ///
    /// This value ranges from \[0, 62\], with 63 being reserved for future use.
    ///
    /// If nalu_type is equal to EOB_NUT then this is set to 0.
    pub nuh_layer_id: u8,

    /// The `nuh_temporal_id_plus1` is 3 bits, where the value minus 1 is the temporal id for the NAL unit.
    ///
    /// This value cannot be 0.
    pub nuh_temporal_id_plus1: NonZero<u8>,

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
    pub chroma_format_idc: u64,

    /// The `separate_colour_plane_flag` is a single bit.
    ///
    /// 0 means the the color components aren't coded separately and `ChromaArrayType` is set to `chroma_format_idc`.
    ///
    /// 1 means the 3 color components of the 4:4:4 chroma format are coded separately and
    /// `ChromaArrayType` is set to 0.
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub separate_color_plane_flag: Option<bool>,

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

    /// An optional [`ConformanceWindow`] struct. This is computed by other fields, and isn't directly set.
    ///
    /// If the `conformance_window_flag` is set, then `conf_win_left_offset`, `conf_win_right_offset`,
    /// `conf_win_top_offset`, and `conf_win_bottom_offset` will be read and stored.
    ///
    /// Refer to the [`ConfWindowInfo`] struct for more info.
    pub conformance_window: Option<ConformanceWindow>,

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
    pub bit_depth_luma_minus8: u64,

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
    pub bit_depth_chroma_minus8: u64,

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
    pub log2_max_pic_order_cnt_lsb_minus4: u64,

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

    pub num_short_term_ref_pic_sets: u64,

    pub long_term_ref_pics_present_flag: bool,

    pub sps_temporal_mvp_enabled_flag: bool,
    pub strong_intra_smoothing_enabled_flag: bool,

    pub vui_parameters: Option<VuiParameters>,

    pub range_extension: Option<SpsRangeExtension>,
    pub multilayer_extension: Option<SpsMultilayerExtension>,
    pub sps_3d_extension: Option<Sps3dExtension>,
    pub scc_extension: Option<SpsSccExtension>,
}

impl Sps {
    /// Parses an SPS from the input bytes.
    /// Returns an `Sps` struct.
    pub fn parse(reader: impl io::Read) -> io::Result<Self> {
        let mut bit_reader = BitReader::new(reader);

        let forbidden_zero_bit = bit_reader.read_bit()?;
        if forbidden_zero_bit {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "forbidden_zero_bit is not zero"));
        }

        let nalu_type = bit_reader.read_bits(6)? as u8;
        if nalu_type != NALUnitType::SpsNut.0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "NAL unit type is not SPS"));
        }

        // nal unit header (excluding forbidden zero bit and nal_unit_type)
        // ISO/IEC-23008-2-2020 - 7.3.1.2
        let nuh_layer_id = bit_reader.read_bits(6)? as u8;
        // nuh_temporal_id_plus1 TODO a lot of logic for this apparently, 7.4.2.2
        let nuh_temporal_id_plus1 = bit_reader.read_bits(3)? as u8;

        // begin ISO/IEC-23008-2-2020 - 7.3.2.2.1
        // semantics in ISO/IEC-23008-2-2020 - 7.4.3.2.1
        let sps_video_parameter_set_id = bit_reader.read_bits(4)? as u8;

        let sps_max_sub_layers_minus1 = bit_reader.read_bits(3)? as u8;
        let sps_temporal_id_nesting_flag = bit_reader.read_bit()?;

        dbg!(
            sps_video_parameter_set_id,
            sps_max_sub_layers_minus1,
            sps_temporal_id_nesting_flag
        );

        // ISO/IEC-23008-2-2020 - 7.3.3
        let profile_tier_level = ProfileTierLevel::parse(
            &mut bit_reader,
            true, // profile_present_flag
            sps_max_sub_layers_minus1,
        )?;

        dbg!(&profile_tier_level);

        // back to ISO/IEC-23008-2-2020 - 7.3.2.2.1
        let sps_seq_parameter_set_id = bit_reader.read_exp_golomb()?;

        let chroma_format_idc = bit_reader.read_exp_golomb()?;

        let mut separate_color_plane_flag = None;
        if chroma_format_idc == 3 {
            separate_color_plane_flag = Some(bit_reader.read_bit()?);
        }

        let pic_width_in_luma_samples = bit_reader.read_exp_golomb()?;
        let pic_height_in_luma_samples = bit_reader.read_exp_golomb()?;

        let conformance_window_flag = bit_reader.read_bit()?;

        let mut conformance_window = None;
        if conformance_window_flag {
            conformance_window = Some(ConformanceWindow::parse(&mut bit_reader)?)
        }

        let bit_depth_luma_minus8 = bit_reader.read_exp_golomb()?;
        let bit_depth_y = 8 + bit_depth_luma_minus8; // BitDepth_Y
        let bit_depth_chroma_minus8 = bit_reader.read_exp_golomb()?;
        let bit_depth_c = 8 + bit_depth_chroma_minus8; // BitDepth_C

        let log2_max_pic_order_cnt_lsb_minus4 = bit_reader.read_exp_golomb()?;

        let sps_sub_layer_ordering_info_present_flag = bit_reader.read_bit()?;
        let sub_layer_ordering_info = SubLayerOrderingInfo::parse(
            &mut bit_reader,
            sps_sub_layer_ordering_info_present_flag,
            sps_max_sub_layers_minus1,
        )?;

        let log2_min_luma_coding_block_size_minus3 = bit_reader.read_exp_golomb()?;
        let log2_diff_max_min_luma_coding_block_size = bit_reader.read_exp_golomb()?;
        let log2_min_luma_transform_block_size_minus2 = bit_reader.read_exp_golomb()?;
        let log2_diff_max_min_luma_transform_block_size = bit_reader.read_exp_golomb()?;
        let max_transform_hierarchy_depth_inter = bit_reader.read_exp_golomb()?;
        let max_transform_hierarchy_depth_intra = bit_reader.read_exp_golomb()?;

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
            pcm = Some(Pcm::parse(&mut bit_reader)?);
        }

        let num_short_term_ref_pic_sets = bit_reader.read_exp_golomb()?;
        ShortTermRefPicSets::skip(&mut bit_reader, num_short_term_ref_pic_sets as usize)?;

        let long_term_ref_pics_present_flag = bit_reader.read_bit()?;
        if long_term_ref_pics_present_flag {
            let num_long_term_ref_pics_sps = bit_reader.read_exp_golomb()?;
            for _ in 0..num_long_term_ref_pics_sps {
                bit_reader.read_bits((log2_max_pic_order_cnt_lsb_minus4 + 4).try_into().unwrap_or(0))?; // lt_ref_pic_poc_lsb_sps
                bit_reader.read_bits(1)?; // used_by_curr_pic_lt_sps_flag
            }
        }

        let sps_temporal_mvp_enabled_flag = bit_reader.read_bit()?;
        let strong_intra_smoothing_enabled_flag = bit_reader.read_bit()?;

        let mut vui_parameters = None;
        let vui_parameters_present_flag = bit_reader.read_bit()?;
        if vui_parameters_present_flag {
            vui_parameters = Some(VuiParameters::parse(&mut bit_reader, sps_max_sub_layers_minus1)?);
        }

        // Extensions

        let mut sps_range_extension_flag = false;
        let mut sps_multilayer_extension_flag = false;
        let mut sps_3d_extension_flag = false;
        let mut sps_scc_extension_flag = false;

        let sps_extension_flag = bit_reader.read_bit()?;
        if sps_extension_flag {
            sps_range_extension_flag = bit_reader.read_bit()?;
            sps_multilayer_extension_flag = bit_reader.read_bit()?;
            sps_3d_extension_flag = bit_reader.read_bit()?;
            sps_scc_extension_flag = bit_reader.read_bit()?;
            bit_reader.read_bits(4)?; // sps_extension_4bits
        }

        let mut range_extension = None;
        if sps_range_extension_flag {
            range_extension = Some(SpsRangeExtension::parse(&mut bit_reader)?);
        }

        let mut multilayer_extension = None;
        if sps_multilayer_extension_flag {
            multilayer_extension = Some(SpsMultilayerExtension::parse(&mut bit_reader)?);
        }

        let mut sps_3d_extension = None;
        if sps_3d_extension_flag {
            sps_3d_extension = Some(Sps3dExtension::parse(&mut bit_reader)?);
        }

        let mut scc_extension = None;
        if sps_scc_extension_flag {
            scc_extension = Some(SpsSccExtension::parse(
                &mut bit_reader,
                chroma_format_idc,
                bit_depth_y,
                bit_depth_c,
            )?);
        }

        // Ignore sps_extension_data_flag as specified by 7.4.3.2.1, page 101

        bit_reader.align()?;

        // Skip to the end or rbsp_trailing_bits()
        while match bit_reader.read_u8() {
            Ok(byte) => byte & 0b0000_0001 == 0,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => false,
            Err(e) => return Err(e),
        } {}

        Ok(Sps {
            nuh_layer_id,
            nuh_temporal_id_plus1: NonZero::new(nuh_temporal_id_plus1).unwrap(),
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
            num_short_term_ref_pic_sets,
            long_term_ref_pics_present_flag,
            sps_temporal_mvp_enabled_flag,
            strong_intra_smoothing_enabled_flag,
            vui_parameters,
            range_extension,
            multilayer_extension,
            sps_3d_extension,
            scc_extension,
        })
    }

    pub fn parse_with_emulation_prevention(reader: impl io::Read) -> io::Result<Self> {
        Self::parse(EmulationPreventionIo::new(reader))
    }

    /// The height as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `height = pic_height_in_luma_samples - sub_height_c * (conf_win_top_offset + conf_win_bottom_offset)`
    pub fn height(&self) -> u64 {
        let sub_height_c = if matches!(self.chroma_format_idc, 1) { 2 } else { 1 };

        let sum = self
            .conformance_window
            .as_ref()
            .map_or(0, |cwi| (cwi.conf_win_top_offset + cwi.conf_win_bottom_offset));

        self.pic_height_in_luma_samples - sub_height_c * sum
    }

    /// The width as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `width = pic_width_in_luma_samples - sub_width_c * (conf_win_left_offset + conf_win_right_offset)`
    pub fn width(&self) -> u64 {
        let sub_width_c = if matches!(self.chroma_format_idc, 1 | 2) { 2 } else { 1 };

        let sum = self
            .conformance_window
            .as_ref()
            .map_or(0, |cwi| (cwi.conf_win_left_offset + cwi.conf_win_right_offset));

        self.pic_width_in_luma_samples - sub_width_c * sum
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
        let data = b"\x42\x01\x01\x01\x40\x00\x00\x03\x00\x90\x00\x00\x03\x00\x00\x03\x00\x78\xA0\x03\xC0\x80\x11\x07\xCB\x96\xB4\xA4\x25\x92\xE3\x01\x6A\x02\x02\x02\x08\x00\x00\x03\x00\x08\x00\x00\x03\x00\xF3\x00\x2E\xF2\x88\x00\x02\x62\x5A\x00\x00\x13\x12\xD0\x20\x22";

        let sps = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();
        assert_eq!(sps.width(), 1920);
        assert_eq!(sps.height(), 1080);
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
        // 0x40 = 0100 0000:
        //   forbidden_zero_bit = 0;
        //   next 6 bits (100000) = 32 ≠ 33.
        let data = [0x40];
        let err = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "NAL unit type is not SPS");
    }

    #[test]
    fn test_sub_layer_for_loop() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x40\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \xC0\x16\x88\x07\xC5\xDF\x84\x00";
        let result = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_sub_layer_loop_without_level_idc() {
        let data = b"\x42\x00\x03\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\
                    \x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \xC0\x0F\x02\x00\x43\x97\x7E\x10";
        let result = Sps::parse_with_emulation_prevention(io::Cursor::new(data)).unwrap();

        insta::assert_debug_snapshot!(result);
    }
}
