#pragma endian big
#include <std/core.pat>
#include <std/io.pat>
#include <std/math.pat>

bitfield LeadingZeros {
    bit : 1;
    if (bit == 1)
       break;
} [[inline]];

bitfield OneBit {
    bit : 1;
} [[inline]];

bitfield ExpGolomb {
  LeadingZeros exp_golomb_0[while(true)];
  OneBit exp_golomb_1[std::core::member_count(exp_golomb_0) - 1];
} [[sealed, format("exp_golomb_value")]];

fn exp_golomb_value(ref ExpGolomb exp_golomb) {
    u64 result = 1;

    for (u64 i = 0, i < std::core::member_count(exp_golomb.exp_golomb_1), i = i + 1) {
        result = result << 1;
        result = result | exp_golomb.exp_golomb_1[i].bit;
    }

    return result - 1;
};

bitfield SignedExpGolomb {
  LeadingZeros exp_golomb_0[while(true)];
  OneBit exp_golomb_1[std::core::member_count(exp_golomb_0) - 1];
} [[sealed, format("signed_exp_golomb_value")]];

fn signed_exp_golomb_value(ref SignedExpGolomb signed_exp_golomb) {
    u64 exp_golomb = exp_golomb_value(signed_exp_golomb);
    if (exp_golomb % 2 == 0) {
        return -(s64(exp_golomb / 2));
    } else {
        return s64(exp_golomb / 2) + 1;
    }
};

enum NALUnitType : u8 {
    TrailN = 0,
    TrailR = 1,
    TsaN = 2,
    TsaR = 3,
    StsaN = 4,
    StsaR = 5,
    RadlN = 6,
    RadlR = 7,
    RaslN = 8,
    RaslR = 9,
    RsvVclN10 = 10,
    RsvVclR11 = 11,
    RsvVclN12 = 12,
    RsvVclR13 = 13,
    RsvVclN14 = 14,
    RsvVclR15 = 15,
    BlaWLp = 16,
    BlaWRadl = 17,
    BlaNLp = 18,
    IdrWRadl = 19,
    IdrNLp = 20,
    CraNut = 21,
    RsvIrapVcl22 = 22,
    RsvIrapVcl23 = 23,
    RsvVcl24 = 24,
    RsvVcl25 = 25,
    RsvVcl26 = 26,
    RsvVcl27 = 27,
    RsvVcl28 = 28,
    RsvVcl29 = 29,
    RsvVcl30 = 30,
    RsvVcl31 = 31,
    VpsNut = 32,
    SpsNut = 33,
    PpsNut = 34,
    AudNut = 35,
    EosNut = 36,
    EobNut = 37,
    FdNut = 38,
    PrefixSeiNut = 39,
    SuffixSeiNut = 40,
    RsvNvcl41 = 41,
    RsvNvcl42 = 42,
    RsvNvcl43 = 43,
    RsvNvcl44 = 44,
    RsvNvcl45 = 45,
    RsvNvcl46 = 46,
    RsvNvcl47 = 47,
};

bitfield ProfileTierLevelSubLayerPresent {
    bool sub_layer_profile_present_flag : 1;
    bool sub_layer_level_present_flag : 1;
};

bitfield ReservedZero2Bits {
    bits : 2;
} [[inline]];

bitfield ProfileTierLevelSubLayerProfile {
    sub_layer_profile_space : 2;
    bool sub_layer_tier_flag : 1;
    sub_layer_profile_idc : 5;
    sub_layer_profile_compatibility_flag : 32;
    bool sub_layer_progressive_source_flag : 1;
    bool sub_layer_interlaced_source_flag : 1;
    bool sub_layer_non_packed_constraint_flag : 1;
    bool sub_layer_frame_only_constraint_flag : 1;
    sub_layer_reserved_zero_43bits : 43;
    sub_layer_reserved_zero_bit : 1;
};

bitfield ProfileTierLevelSubLayerLevel {
    sub_layer_level_idc : 8;
};

bitfield ProfileTierLevelSubLayer {
    if (parent.sub_layers_present[std::core::array_index()].sub_layer_profile_present_flag) {
        ProfileTierLevelSubLayerProfile profile;
    }
    if (parent.sub_layers_present[std::core::array_index()].sub_layer_level_present_flag) {
        ProfileTierLevelSubLayerLevel level;
    }
};

bitfield ProfileTierLevel {
    general_profile_space : 2;
    bool general_tier_flag : 1;
    general_profile_idc : 5;
    general_profile_compatibility_flag : 32;
    bool general_progressive_source_flag : 1;
    bool general_interlaced_source_flag : 1;
    bool general_non_packed_constraint_flag : 1;
    bool general_frame_only_constraint_flag : 1;
    general_reserved_zero_43bits : 43;
    general_reserved_zero_bit : 1;
    general_level_idc : 8;
    ProfileTierLevelSubLayerPresent sub_layers_present[parent.sps_max_sub_layers_minus1];
    if (parent.sps_max_sub_layers_minus1 > 0) {
        ReservedZero2Bits reserved_zero_2bits[8 - parent.sps_max_sub_layers_minus1];
    }
    ProfileTierLevelSubLayer sub_layers[parent.sps_max_sub_layers_minus1];
};

bitfield SubLayerOrderingInfo {
    u64 len = 1;
    if (parent.sps_sub_layer_ordering_info_present_flag) {
        len = parent.sps_max_sub_layers_minus1 + 1;
    };
    ExpGolomb sps_max_dec_pic_buffering_minus1[len];
    ExpGolomb sps_max_num_reorder_pics[len];
    ExpGolomb sps_max_latency_increase_plus1[len];
};

bitfield ScalingListDataInnerLoopDeltaCoef {
    SignedExpGolomb scaling_list_delta_coef;
};

bitfield ScalingListDataMatrixColumn {
    bool scaling_list_pred_mode_flag : 1;
    if (!scaling_list_pred_mode_flag) {
        ExpGolomb scaling_list_pred_matrix_id_delta;
    } else {
        u64 coefNum = std::math::min(64, (1 << (4 + (parent.sizeId << 1))));
        if (parent.sizeId > 1) {
            SignedExpGolomb scaling_list_dc_coef_minus8;
        }
        ScalingListDataInnerLoopDeltaCoef scaling_list_delta_coef[coefNum];
    }
};

bitfield ScalingListDataMatrix {
    u8 sizeId = std::core::array_index();
    u8 len = 6;
    if (sizeId == 3) {
        len = 2;
    };
    ScalingListDataMatrixColumn column[len];
};

bitfield ScalingListData {
    ScalingListDataMatrix matrix[4];
};

bitfield StRefPicSetNegativePic {
    ExpGolomb delta_poc_s0_minus1;
    bool used_by_curr_pic_s0_flag : 1;
};

bitfield StRefPicSetPositivePic {
    ExpGolomb delta_poc_s1_minus1;
    bool used_by_curr_pic_s1_flag : 1;
};

bitfield StRefPicSet {
    u64 stRpsIdx = std::core::array_index();

    bool inter_ref_pic_set_prediction_flag = false;
    if (stRpsIdx != 0) {
        bool inter_ref_pic_set_prediction_flag : 1;
    };

    if (inter_ref_pic_set_prediction_flag) {
        std::unimplemented();
    } else {
        ExpGolomb num_negative_pics;
        ExpGolomb num_positive_pics;
        StRefPicSetNegativePic negative_pics[exp_golomb_value(num_negative_pics)];
        StRefPicSetPositivePic positive_pics[exp_golomb_value(num_positive_pics)];
    };
};

bitfield LtRefPicSet {
    lt_ref_pic_poc_lsb_sps : parent.log2_max_pic_order_cnt_lsb_minus4 + 4;
    bool used_by_curr_pic_lt_sps_flag : 1;
};

bitfield SubLayerHrdParameters {
    ExpGolomb bit_rate_value_minus1;
    ExpGolomb cpb_size_value_minus1;
    if (parent.parent.sub_pic_hrd_params_present_flag) {
        ExpGolomb cpb_size_du_value_minus1;
        ExpGolomb bit_rate_du_value_minus1;
    }
    bool cbr_flag : 1;
};

bitfield HrdParametersSubLayer {
    bool fixed_pic_rate_general_flag : 1;
    bool fixed_pic_rate_within_cvs_flag = true;
    if (!fixed_pic_rate_general_flag) {
        bool fixed_pic_rate_within_cvs_flag : 1;
    }
    bool low_delay_hrd_flag = false;
    if (fixed_pic_rate_within_cvs_flag) {
        ExpGolomb elemental_duration_in_tc_minus1;
    } else {
        bool low_delay_hrd_flag : 1;
    }
    u64 cpb_cnt_minus1_value = 0;
    if (!low_delay_hrd_flag) {
        ExpGolomb cpb_cnt_minus1;
        cpb_cnt_minus1_value = exp_golomb_value(cpb_cnt_minus1);
    }
    if (parent.nal_hrd_parameters_present_flag) {
        SubLayerHrdParameters sub_layer_hrd_parameters[cpb_cnt_minus1_value];
    }
    if (parent.vcl_hrd_parameters_present_flag) {
        SubLayerHrdParameters sub_layer_hrd_parameters[cpb_cnt_minus1_value];
    }
};

bitfield HrdParameters {
    bool nal_hrd_parameters_present_flag : 1;
    bool vcl_hrd_parameters_present_flag : 1;
    if (nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag) {
        bool sub_pic_hrd_params_present_flag : 1;
        if (sub_pic_hrd_params_present_flag) {
            tick_divisor_minus2 : 8;
            du_cpb_removal_delay_increment_length_minus1 : 5;
            bool sub_pic_cpb_params_in_pic_timing_sei_flag : 1;
            dpb_output_delay_du_length_minus1 : 5;
        }
        bit_rate_scale : 4;
        cpb_size_scale : 4;
        if (sub_pic_hrd_params_present_flag) {
            cpb_size_du_scale : 4;
        }
        initial_cpb_removal_delay_length_minus1 : 5;
        au_cpb_removal_delay_length_minus1: 5;
        dpb_output_delay_length_minus1 : 5;
    }
    HrdParametersSubLayer sub_layers[parent.parent.sps_max_sub_layers_minus1 + 1];
};

enum VideoFormat : u8 {
    Component = 0,
    PAL = 1,
    NTSC = 2,
    SECAM = 3,
    MAC = 4,
    Unspecified = 5,
};

enum AspectRatioIdc : u8 {
    Unspecified = 0,
    Square = 1,
    Aspect12_11 = 2,
    Aspect10_11 = 3,
    Aspect16_11 = 4,
    Aspect40_33 = 5,
    Aspect24_11 = 6,
    Aspect20_11 = 7,
    Aspect32_11 = 8,
    Aspect80_33 = 9,
    Aspect18_11 = 10,
    Aspect15_11 = 11,
    Aspect64_33 = 12,
    Aspect160_99 = 13,
    Aspect4_3 = 14,
    Aspect3_2 = 15,
    Aspect2_1 = 16,
    ExtendedSar = 255,
};

bitfield VuiParameters {
    bool aspect_ratio_info_present_flag : 1;
    if (aspect_ratio_info_present_flag) {
        AspectRatioIdc aspect_ratio_idc : 8;
        if (aspect_ratio_idc == AspectRatioIdc::ExtendedSar) {
            sar_width : 16;
            sar_height : 16;
        }
    }
    bool overscan_info_present_flag : 1;
    if (overscan_info_present_flag) {
        bool overscan_appropriate_flag : 1;
    }
    bool video_signal_type_present_flag : 1;
    if (video_signal_type_present_flag) {
        VideoFormat video_format : 3;
        bool video_full_range_flag : 1;
        bool colour_description_present_flag : 1;
        if (colour_description_present_flag) {
            colour_primaries : 8;
            transfer_characteristics : 8;
            matrix_coeffs : 8;
        }
    }
    bool chroma_loc_info_present_flag : 1;
    if (chroma_loc_info_present_flag) {
        ExpGolomb chroma_sample_loc_type_top_field;
        ExpGolomb chroma_sample_loc_type_bottom_field;
    }
    bool neutral_chroma_indication_flag : 1;
    bool field_seq_flag : 1;
    bool frame_field_info_present_flag : 1;
    bool default_display_window_flag : 1;
    if (default_display_window_flag) {
        ExpGolomb def_disp_win_left_offset;
        ExpGolomb def_disp_win_right_offset;
        ExpGolomb def_disp_win_top_offset;
        ExpGolomb def_disp_win_bottom_offset;
    }
    bool vui_timing_info_present_flag : 1;
    if (vui_timing_info_present_flag) {
        vui_num_units_in_tick : 32;
        vui_time_scale : 32;
        bool vui_poc_proportional_to_timing_flag : 1;
        if (vui_poc_proportional_to_timing_flag) {
            ExpGolomb vui_num_ticks_poc_diff_one_minus1;
        }
        bool vui_hrd_parameters_present_flag : 1;
        if (vui_hrd_parameters_present_flag) {
            HrdParameters hrd_parameters;
        }
    }
    bool bitstream_restriction_flag : 1;
    if (bitstream_restriction_flag) {
        bool tiles_fixed_structure_flag : 1;
        bool motion_vectors_over_pic_boundaries_flag : 1;
        bool restricted_ref_pic_lists_flag : 1;
        bool min_spatial_segmentation_idc : 1;
        bool max_bytes_per_pic_denom : 1;
        bool max_bits_per_min_cu_denom : 1;
        bool log2_max_mv_length_horizontal : 1;
        bool log2_max_mv_length_vertical : 1;
    }
};

bitfield SPS {
    forbidden_zero_bit : 1;
    NALUnitType nal_unit_type : 6;
    nuh_layer_id : 6;
    nuh_temporal_id_plus1 : 3;
    sps_video_parameter_set_id : 4;
    sps_max_sub_layers_minus1 : 3;
    bool sps_temporal_id_nesting_flag : 1;
    ProfileTierLevel profile_tier_level;
    ExpGolomb sps_seq_parameter_set_id;
    ExpGolomb chroma_format_idc;
    if (exp_golomb_value(chroma_format_idc) == 3) {
        bool separate_colour_plane_flag : 1;
    }
    ExpGolomb pic_width_in_luma_samples;
    ExpGolomb pic_height_in_luma_samples;
    bool conformance_window_flag : 1;
    if (conformance_window_flag) {
        ExpGolomb conf_win_left_offset;
        ExpGolomb conf_win_right_offset;
        ExpGolomb conf_win_top_offset;
        ExpGolomb conf_win_bottom_offset;
    }
    ExpGolomb bit_depth_luma_minus8;
    ExpGolomb bit_depth_chroma_minus8;
    ExpGolomb log2_max_pic_order_cnt_lsb_minus4;
    bool sps_sub_layer_ordering_info_present_flag : 1;
    SubLayerOrderingInfo sub_layer_ordering_info;
    ExpGolomb log2_min_luma_coding_block_size_minus3;
    ExpGolomb log2_diff_max_min_luma_coding_block_size;
    ExpGolomb log2_min_luma_transform_block_size_minus2;
    ExpGolomb log2_diff_max_min_luma_transform_block_size;
    ExpGolomb max_transform_hierarchy_depth_inter;
    ExpGolomb max_transform_hierarchy_depth_intra;
    bool scaling_list_enabled_flag : 1;
    if (scaling_list_enabled_flag) {
        bool sps_scaling_list_data_present_flag : 1;
        if (sps_scaling_list_data_present_flag) {
            ScalingListData scaling_list_data;
        }
    }
    bool amp_enabled_flag : 1;
    bool sample_adaptive_offset_enabled_flag : 1;
    bool pcm_enabled_flag : 1;
    if (pcm_enabled_flag) {
        pcm_sample_bit_depth_luma_minus1 : 4;
        pcm_sample_bit_depth_chroma_minus1 : 4;
        ExpGolomb log2_min_pcm_luma_coding_block_size_minus3;
        ExpGolomb log2_diff_max_min_pcm_luma_coding_block_size;
        bool pcm_loop_filter_disabled_flag : 1;
    }
    ExpGolomb num_short_term_ref_pic_sets;
    StRefPicSet st_ref_pic_set[exp_golomb_value(num_short_term_ref_pic_sets)];
    bool long_term_ref_pics_present_flag : 1;
    if (long_term_ref_pics_present_flag) {
        ExpGolomb num_long_term_ref_pics_sps;
    }
    bool sps_temporal_mvp_enabled_flag : 1;
    bool strong_intra_smoothing_enabled_flag : 1;
    bool vui_parameters_present_flag : 1;
    if (vui_parameters_present_flag) {
        VuiParameters vui_parameters;
    }

    bool sps_extension_present_flag : 1;
    if (sps_extension_present_flag) {
        bool sps_range_extension_flag : 1;
        bool sps_multilayer_extension_flag : 1;
        bool sps_3d_extension_flag : 1;
        bool sps_scc_extension_flag : 1;
        bool sps_extension_4bits : 4;
        if (sps_range_extension_flag) {
            std::unimplemented();
        }
        if (sps_multilayer_extension_flag) {
            std::unimplemented();
        }
        if (sps_3d_extension_flag) {
            std::unimplemented();
        }
        if (sps_scc_extension_flag) {
            std::unimplemented();
        }
        if (sps_extension_4bits != 0) {
            std::unimplemented();
        }
    }
};

// Make sure that the SPS is already processed by emulation prevention.
// This script does not support doing emulation prevention.
SPS sps @ 0x00;
