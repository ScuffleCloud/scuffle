use std::io;
use std::num::NonZeroU32;

use byteorder::{BigEndian, ReadBytesExt};
use scuffle_bytes_util::{BitReader, BitWriter};
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt, size_of_exp_golomb, size_of_signed_exp_golomb};

use crate::{AspectRatioIdc, NALUnitType, VideoFormat};

/// The Sequence Parameter Set.
/// ISO/IEC-14496-10-2022 - 7.3.2
#[derive(Debug, Clone, PartialEq)]
pub struct Sps {
    /// The number of emulation bytes that are in the Sps.
    ///
    /// This is necessary because reading in a vec of Sps in `AVCDecoderConfigurationRecord::build()` requires
    /// the length of the Sps, however, the existing size function would have to build the Sps into a bytestream
    /// and then determine how many emulation prevention bytes exist in the Sps. This field saves us from having to
    /// rebuild the entire Sps by simply storing the number of emulation prevention bytes present in the original Sps.
    ///
    /// Note that the input bytestream can potentially have 0 emulation prevention bytes, but when rebuilding the Sps we
    /// will insert appropriate emulation prevention bytes. If you notice your Sps seems to be different in this case,
    /// you can simply use [`Sps::reduce()`] on your original bytestream to add the emulation prevention bytes to your Sps.
    pub emu_bytes: u8,

    /// The `forbidden_zero_bit` is a single bit that must be set to 0. Otherwise
    /// `parse()` will return an error. ISO/IEC-14496-10-2022 - 7.4.1
    pub forbidden_zero_bit: bool,

    /// The `nal_ref_idc` is comprised of 2 bits.
    ///
    /// A nonzero value means the NAL unit has any of the following: SPS, SPS extension,
    /// subset SPS, PPS, slice of a reference picture, slice of a data partition of a reference picture,
    /// or a prefix NAL unit preceeding a slice of a reference picture.
    ///
    /// 0 means that the stream is decoded using the process from Clauses 2-9 (ISO/IEC-14496-10-2022)
    /// that the slice or slice data partition is part of a non-reference picture.
    /// Additionally, if `nal_ref_idc` is 0 for a NAL unit with `nal_unit_type`
    /// ranging from \[1, 4\] then `nal_ref_idc` must be 0 for all NAL units with `nal_unit_type` between [1, 4].
    ///
    /// If the `nal_unit_type` is 5, then the `nal_ref_idc` cannot be 0.
    ///
    /// If `nal_unit_type` is 6, 9, 10, 11, or 12, then the `nal_ref_idc` must be 0.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.1
    pub nal_ref_idc: u8,

    /// The `nal_unit_type` is comprised of 5 bits. See the NALUnitType nutype enum for more info.
    pub nal_unit_type: NALUnitType,

    /// The `profile_idc` of the coded video sequence as a u8.
    ///
    /// It is comprised of 8 bits or 1 byte. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub profile_idc: u8,

    /// `constraint_set0_flag`: `1` if it abides by the constraints in A.2.1, `0` if unsure or otherwise.
    ///
    /// If `profile_idc` is 44, 100, 110, 122, or 244, this is automatically set to false.
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub constraint_set0_flag: bool,

    /// `constraint_set1_flag`: `1` if it abides by the constraints in A.2.2, `0` if unsure or otherwise.
    ///
    /// If `profile_idc` is 44, 100, 110, 122, or 244, this is automatically set to false.
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub constraint_set1_flag: bool,

    /// `constraint_set2_flag`: `1` if it abides by the constraints in A.2.3, `0` if unsure or otherwise.
    ///
    /// If `profile_idc` is 44, 100, 110, 122, or 244, this is automatically set to false.
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub constraint_set2_flag: bool,

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
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub constraint_set3_flag: bool,

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
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub constraint_set4_flag: bool,

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
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub constraint_set5_flag: bool,

    /// The `level_idc` of the coded video sequence as a u8.
    ///
    /// It is comprised of 8 bits or 1 byte. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub level_idc: u8,

    /// The `seq_parameter_set_id` is the id of the SPS referred to by the PPS (picture parameter set).
    ///
    /// The value of this ranges from \[0, 31\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `31` which is encoded as `000 0010 0000`, which is 11 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub seq_parameter_set_id: u16,

    /// An optional `SpsExtended`. Refer to the SpsExtended struct for more info.
    ///
    /// This will be parsed if `profile_idc` is equal to any of the following values:
    /// 44, 83, 86, 100, 110, 118, 122, 128, 134, 135, 138, 139, or 244.
    pub ext: Option<SpsExtended>,

    /// The `log2_max_frame_num_minus4` is the value used when deriving MaxFrameNum from the equation:
    /// `MaxFrameNum` = 2^(`log2_max_frame_num_minus4` + 4)
    ///
    /// The value of this ranges from \[0, 12\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `12` which is encoded as `000 1101`, which is 7 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub log2_max_frame_num_minus4: u8,

    /// The `pic_order_cnt_type` specifies how to decode the picture order count in subclause 8.2.1.
    ///
    /// The value of this ranges from \[0, 2\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `2` which is encoded as `011`, which is 3 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// There are a few subsequent fields that are read if `pic_order_cnt_type` is 0 or 1.
    ///
    /// In the case of 0, `log2_max_pic_order_cnt_lsb_minus4` is read as an exp golomb (unsigned).
    ///
    /// In the case of 1, `delta_pic_order_always_zero_flag`, `offset_for_non_ref_pic`,
    /// `offset_for_top_to_bottom_field`, `num_ref_frames_in_pic_order_cnt_cycle` and
    /// `offset_for_ref_frame` will be read and stored in pic_order_cnt_type1.
    ///
    /// Refer to the PicOrderCountType1 struct for more info.
    pub pic_order_cnt_type: u8,

    /// The `log2_max_pic_order_cnt_lsb_minus4` is the value used when deriving MaxFrameNum from the equation:
    /// `MaxPicOrderCntLsb` = 2^(`log2_max_frame_num_minus4` + 4) from subclause 8.2.1.
    ///
    /// This is an `Option<u8>` because the value is only set if `pic_order_cnt_type == 0`.
    ///
    /// The value of this ranges from \[0, 12\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `12` which is encoded as `000 1101`, which is 7 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub log2_max_pic_order_cnt_lsb_minus4: Option<u8>,

    /// An optional `PicOrderCountType1`. This is computed from other fields, and isn't directly set.
    ///
    /// If `pic_order_cnt_type == 1`, then the `PicOrderCountType1` will be computed.
    ///
    /// Refer to the PicOrderCountType1 struct for more info.
    pub pic_order_cnt_type1: Option<PicOrderCountType1>,

    /// The `max_num_ref_frames` is the max short-term and long-term reference frames,
    /// complementary reference field pairs, and non-paired reference fields that
    /// can be used by the decoder for inter-prediction of pictures in the coded video.
    ///
    /// The value of this ranges from \[0, `MaxDpbFrames`\], which is specified in subclause A.3.1 or A.3.2.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `14` which is encoded as `000 1111`, which is 7 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub max_num_ref_frames: u8,

    /// The `gaps_in_frame_num_value_allowed_flag` is a single bit.
    ///
    /// The value specifies the allowed values of `frame_num` from subclause 7.4.3 and the decoding process
    /// if there is an inferred gap between the values of `frame_num` from subclause 8.2.5.2.
    pub gaps_in_frame_num_value_allowed_flag: bool,

    /// The `pic_width_in_mbs_minus1` is the width of each decoded picture in macroblocks as a u64.
    ///
    /// We then use this (along with the left and right frame crop offsets) to calculate the width as:
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub pic_width_in_mbs_minus1: u64,

    /// The `pic_height_in_map_units_minus1` is the height of each decoded frame in slice group map units as a u64.
    ///
    /// We then use this (along with the bottom and top frame crop offsets) to calculate the height as:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16) -
    /// frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub pic_height_in_map_units_minus1: u64,

    /// The `mb_adaptive_frame_field_flag` is a single bit.
    ///
    /// If `frame_mbs_only_flag` is NOT set then this field is read and stored.
    ///
    /// 0 means there is no switching between frame and field macroblocks in a picture.
    ///
    /// 1 means the might be switching between frame and field macroblocks in a picture.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub mb_adaptive_frame_field_flag: Option<bool>,

    /// The `direct_8x8_inference_flag` specifies the method used to derive the luma motion
    /// vectors for B_Skip, B_Direct_8x8 and B_Direct_16x16 from subclause 8.4.1.2, and is a single bit.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub direct_8x8_inference_flag: bool,

    /// An optional `frame_crop_info` struct. This is computed by other fields, and isn't directly set.
    ///
    /// If the `frame_cropping_flag` is set, then `frame_crop_left_offset`, `frame_crop_right_offset`,
    /// `frame_crop_top_offset`, and `frame_crop_bottom_offset` will be read and stored.
    ///
    /// Refer to the FrameCropInfo struct for more info.
    pub frame_crop_info: Option<FrameCropInfo>,

    /// An optional `SarDimensions` struct. This is computed by other fields, and isn't directly set.
    ///
    /// If the `aspect_ratio_info_present_flag` is set, then the `aspect_ratio_idc` will be read and stored.
    ///
    /// If the `aspect_ratio_idc` is 255, then the `sar_width` and `sar_height` will be read and stored.
    ///
    /// Also known as `sample_aspect_ratio` in the spec.
    ///
    /// The default values are set to 0 for the `aspect_ratio_idc`, `sar_width`, and `sar_height`.
    /// Therefore, this will always be returned by the parse function.
    /// ISO/IEC-14496-10-2022 - E.2.1
    ///
    /// Refer to the SarDimensions struct for more info.
    pub sample_aspect_ratio: Option<SarDimensions>,

    /// An optional `overscan_appropriate_flag` is a single bit.
    ///
    /// If the `overscan_info_present_flag` is set, then this field will be read and stored.
    ///
    /// 0 means the overscan should not be used. (ex: screensharing or security cameras)
    ///
    /// 1 means the overscan can be used. (ex: entertainment TV programming or live video conference)
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub overscan_appropriate_flag: Option<bool>,

    /// An optional `ColorConfig`. This is computed from other fields, and isn't directly set.
    ///
    /// If `video_signal_type_present_flag` is set, then the `ColorConfig` will be computed, and
    /// if the `color_description_present_flag` is set, then the `ColorConfig` will be
    /// comprised of the `video_full_range_flag` (1 bit), `color_primaries` (1 byte as a u8),
    /// `transfer_characteristics` (1 byte as a u8), and `matrix_coefficients` (1 byte as a u8).
    ///
    /// Otherwise, `color_primaries`, `transfer_characteristics`, and `matrix_coefficients` are set
    /// to 2 (unspecified) by default.
    ///
    /// Refer to the ColorConfig struct for more info.
    pub color_config: Option<ColorConfig>,

    /// An optional `ChromaSampleLoc`. This is computed from other fields, and isn't directly set.
    ///
    /// If `chrome_loc_info_present_flag` is set, then the `ChromaSampleLoc` will be computed, and
    /// is comprised of `chroma_sample_loc_type_top_field` and `chroma_sample_loc_type_bottom_field`.
    ///
    /// Refer to the ChromaSampleLoc struct for more info.
    pub chroma_sample_loc: Option<ChromaSampleLoc>,

    /// An optional `TimingInfo`. This is computed from other fields, and isn't directly set.
    ///
    /// If `timing_info_present_flag` is set, then the `TimingInfo` will be computed, and
    /// is comprised of `num_units_in_tick` and `time_scale`.
    ///
    /// Refer to the TimingInfo struct for more info.
    pub timing_info: Option<TimingInfo>,
}

impl Sps {
    /// Parses an Sps from the input bytes.
    ///
    /// Returns an `Sps` struct.
    pub fn parse(data: &[u8]) -> io::Result<Self> {
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
        let mut emu_bytes = 0;
        let mut i = 0;
        while i < data.len() {
            if i + 2 < data.len() && data[i] == 0x00 && data[i + 1] == 0x00 && data[i + 2] == 0x03 {
                vec.push(0x00);
                vec.push(0x00);
                i += 3; // Skip the emulation prevention byte.
                emu_bytes += 1;
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

        let nal_ref_idc = bit_reader.read_bits(2)? as u8;
        let nal_unit_type = bit_reader.read_bits(5)? as u8;
        if NALUnitType(nal_unit_type) != NALUnitType::SPS {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "NAL unit type is not SPS"));
        }

        let profile_idc = bit_reader.read_u8()?;

        let constraint_set0_flag;
        let constraint_set1_flag;
        let constraint_set2_flag;

        match profile_idc {
            // 7.4.2.1.1
            44 | 100 | 110 | 122 | 244 => {
                // constraint_set0 thru 2 must be false in this case
                bit_reader.seek_bits(3)?;
                constraint_set0_flag = false;
                constraint_set1_flag = false;
                constraint_set2_flag = false;
            }
            _ => {
                // otherwise we parse the bits as expected
                constraint_set0_flag = bit_reader.read_bit()?;
                constraint_set1_flag = bit_reader.read_bit()?;
                constraint_set2_flag = bit_reader.read_bit()?;
            }
        }

        let constraint_set3_flag = if profile_idc == 44 {
            bit_reader.seek_bits(1)?;
            false
        } else {
            bit_reader.read_bit()?
        };

        let constraint_set4_flag = match profile_idc {
            // 7.4.2.1.1
            77 | 88 | 100 | 118 | 128 | 134 => bit_reader.read_bit()?,
            _ => {
                bit_reader.seek_bits(1)?;
                false
            }
        };

        let constraint_set5_flag = match profile_idc {
            77 | 88 | 100 | 118 => bit_reader.read_bit()?,
            _ => {
                bit_reader.seek_bits(1)?;
                false
            }
        };
        // reserved_zero_2bits
        bit_reader.seek_bits(2)?;

        let level_idc = bit_reader.read_u8()?;
        let seq_parameter_set_id = bit_reader.read_exp_golomb()? as u16;

        let sps_ext = match profile_idc {
            100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135 => {
                Some(SpsExtended::parse(&mut bit_reader)?)
            }
            _ => None,
        };

        let log2_max_frame_num_minus4 = bit_reader.read_exp_golomb()? as u8;
        let pic_order_cnt_type = bit_reader.read_exp_golomb()? as u8;

        let mut log2_max_pic_order_cnt_lsb_minus4 = None;
        let mut pic_order_cnt_type1 = None;

        if pic_order_cnt_type == 0 {
            log2_max_pic_order_cnt_lsb_minus4 = Some(bit_reader.read_exp_golomb()? as u8);
        } else if pic_order_cnt_type == 1 {
            pic_order_cnt_type1 = Some(PicOrderCountType1::parse(&mut bit_reader)?)
        }

        let max_num_ref_frames = bit_reader.read_exp_golomb()? as u8;
        let gaps_in_frame_num_value_allowed_flag = bit_reader.read_bit()?;
        let pic_width_in_mbs_minus1 = bit_reader.read_exp_golomb()?;
        let pic_height_in_map_units_minus1 = bit_reader.read_exp_golomb()?;

        let frame_mbs_only_flag = bit_reader.read_bit()?;
        let mut mb_adaptive_frame_field_flag = None;
        if !frame_mbs_only_flag {
            mb_adaptive_frame_field_flag = Some(bit_reader.read_bit()?);
        }

        let direct_8x8_inference_flag = bit_reader.read_bit()?;

        let mut frame_crop_info = None;

        let frame_cropping_flag = bit_reader.read_bit()?;
        if frame_cropping_flag {
            frame_crop_info = Some(FrameCropInfo::parse(&mut bit_reader)?)
        }

        // setting default values for vui section
        let mut sample_aspect_ratio = None;
        let mut overscan_appropriate_flag = None;
        let mut color_config = None;
        let mut chroma_sample_loc = None;
        let mut timing_info = None;

        let vui_parameters_present_flag = bit_reader.read_bit()?;
        if vui_parameters_present_flag {
            // We read the VUI parameters to get the frame rate.

            let aspect_ratio_info_present_flag = bit_reader.read_bit()?;
            if aspect_ratio_info_present_flag {
                sample_aspect_ratio = Some(SarDimensions::parse(&mut bit_reader)?)
            }

            let overscan_info_present_flag = bit_reader.read_bit()?;
            if overscan_info_present_flag {
                overscan_appropriate_flag = Some(bit_reader.read_bit()?);
            }

            let video_signal_type_present_flag = bit_reader.read_bit()?;
            if video_signal_type_present_flag {
                color_config = Some(ColorConfig::parse(&mut bit_reader)?)
            }

            let chroma_loc_info_present_flag = bit_reader.read_bit()?;
            if sps_ext.as_ref().unwrap_or(&SpsExtended::default()).chroma_format_idc != 1 && chroma_loc_info_present_flag {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "chroma_loc_info_present_flag cannot be set to 1 when chroma_format_idc is not 1",
                ));
            }

            if chroma_loc_info_present_flag {
                chroma_sample_loc = Some(ChromaSampleLoc::parse(&mut bit_reader)?)
            }

            let timing_info_present_flag = bit_reader.read_bit()?;
            if timing_info_present_flag {
                timing_info = Some(TimingInfo::parse(&mut bit_reader)?)
            }
        }

        Ok(Sps {
            emu_bytes,
            forbidden_zero_bit,
            nal_ref_idc,
            nal_unit_type: NALUnitType(nal_unit_type),
            profile_idc,
            constraint_set0_flag,
            constraint_set1_flag,
            constraint_set2_flag,
            constraint_set3_flag,
            constraint_set4_flag,
            constraint_set5_flag,
            level_idc,
            seq_parameter_set_id,
            ext: sps_ext,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4,
            pic_order_cnt_type1,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed_flag,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            mb_adaptive_frame_field_flag,
            direct_8x8_inference_flag,
            frame_crop_info,
            sample_aspect_ratio,
            overscan_appropriate_flag,
            color_config,
            chroma_sample_loc,
            timing_info,
        })
    }

    /// Builds the Sps struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        let mut temp_bytestream = Vec::new();
        let mut bit_writer = BitWriter::new(&mut temp_bytestream);

        bit_writer.write_bit(false)?;
        bit_writer.write_bits(self.nal_ref_idc as u64, 2)?;
        bit_writer.write_bits(self.nal_unit_type.into(), 5)?;
        bit_writer.write_bits(self.profile_idc as u64, 8)?;

        bit_writer.write_bit(self.constraint_set0_flag)?;
        bit_writer.write_bit(self.constraint_set1_flag)?;
        bit_writer.write_bit(self.constraint_set2_flag)?;
        bit_writer.write_bit(self.constraint_set3_flag)?;
        bit_writer.write_bit(self.constraint_set4_flag)?;
        bit_writer.write_bit(self.constraint_set5_flag)?;
        // reserved 2 bits
        bit_writer.write_bits(0, 2)?;

        bit_writer.write_bits(self.level_idc as u64, 8)?;
        bit_writer.write_exp_golomb(self.seq_parameter_set_id as u64)?;

        // sps ext
        if let Some(ext) = &self.ext {
            ext.build(&mut bit_writer)?;
        }

        bit_writer.write_exp_golomb(self.log2_max_frame_num_minus4 as u64)?;
        bit_writer.write_exp_golomb(self.pic_order_cnt_type as u64)?;

        if self.pic_order_cnt_type == 0 {
            bit_writer.write_exp_golomb(self.log2_max_pic_order_cnt_lsb_minus4.unwrap() as u64)?;
        } else if let Some(pic_order_cnt) = &self.pic_order_cnt_type1 {
            pic_order_cnt.build(&mut bit_writer)?;
        }

        bit_writer.write_exp_golomb(self.max_num_ref_frames as u64)?;
        bit_writer.write_bit(self.gaps_in_frame_num_value_allowed_flag)?;
        bit_writer.write_exp_golomb(self.pic_width_in_mbs_minus1)?;
        bit_writer.write_exp_golomb(self.pic_height_in_map_units_minus1)?;

        if let Some(flag) = self.mb_adaptive_frame_field_flag {
            bit_writer.write_bit(false)?;
            bit_writer.write_bit(flag)?;
        } else {
            bit_writer.write_bit(true)?;
        }

        bit_writer.write_bit(self.direct_8x8_inference_flag)?;

        if let Some(frame_crop_info) = &self.frame_crop_info {
            bit_writer.write_bit(true)?;
            frame_crop_info.build(&mut bit_writer)?;
        } else {
            bit_writer.write_bit(false)?;
        }

        match (
            &self.sample_aspect_ratio,
            &self.overscan_appropriate_flag,
            &self.color_config,
            &self.chroma_sample_loc,
            &self.timing_info,
        ) {
            (None, None, None, None, None) => {
                bit_writer.write_bit(false)?;
            }
            _ => {
                // vui_parameters_present_flag
                bit_writer.write_bit(true)?;

                // aspect_ratio_info_present_flag
                if let Some(sar) = &self.sample_aspect_ratio {
                    bit_writer.write_bit(true)?;
                    sar.build(&mut bit_writer)?;
                } else {
                    bit_writer.write_bit(false)?;
                }

                // overscan_info_present_flag
                if let Some(overscan) = self.overscan_appropriate_flag {
                    bit_writer.write_bit(true)?;
                    bit_writer.write_bit(overscan)?;
                } else {
                    bit_writer.write_bit(false)?;
                }

                // video_signal_type_prsent_flag
                if let Some(color) = &self.color_config {
                    bit_writer.write_bit(true)?;
                    color.build(&mut bit_writer)?;
                } else {
                    bit_writer.write_bit(false)?;
                }

                // chroma_log_info_present_flag
                if let Some(chroma) = &self.chroma_sample_loc {
                    bit_writer.write_bit(true)?;
                    chroma.build(&mut bit_writer)?;
                } else {
                    bit_writer.write_bit(false)?;
                }

                // timing_info_present_flag
                if let Some(timing) = &self.timing_info {
                    bit_writer.write_bit(true)?;
                    timing.build(&mut bit_writer)?;
                } else {
                    bit_writer.write_bit(false)?;
                }
            }
        }
        bit_writer.finish()?;

        // now we reinsert the emulation bytes
        let mut zero_byte_count = 0;
        // create the actual writer
        let mut final_writer = BitWriter::new(writer);

        for &byte in &temp_bytestream {
            // check bytes and insert an emulation byte if necessary
            if zero_byte_count == 2 && byte <= 0x03 {
                final_writer.write_bits(0x03, 8)?;
                zero_byte_count = 0;
            }

            // insert current byte
            final_writer.write_bits(byte as u64, 8)?;

            // if current byte is 0x00 then incr the count
            // otherwise reset the count
            if byte == 0x00 {
                zero_byte_count += 1;
            } else {
                zero_byte_count = 0;
            }
        }

        final_writer.finish()?;

        Ok(())
    }

    /// Reduces an Sps from the input bytes to the returned output bytes.
    ///
    /// This takes the same `&[u8]` as the [`Sps::parse`] function, but instead outputs
    /// a result containing a vec of bytes.
    ///
    /// The reductions occur in 2 places, the first is from rebuilding the `color_config`
    /// and the second is when rebuilding the structs from `vui_parameters_present_flag`.
    ///
    /// These reductions minimize the bytes from the original Sps header bytes while remaining
    /// functionally equivalent. This function also prevents having to manually handle rebuilding
    /// and reparsing to have the same output, as ideally we can reduce and parse via the following:
    ///
    /// ```rust
    /// use scuffle_h264::Sps;
    /// let data = b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0";
    ///
    /// let reduced_sps = Sps::reduce(data).unwrap();
    /// let sps = Sps::parse(&reduced_sps).unwrap();
    /// ```
    ///
    /// This returns a result containing a vec of bytes.
    pub fn reduce(data: &[u8]) -> io::Result<Vec<u8>> {
        let original_sps = Sps::parse(data)?;

        // the two reductions happen by building
        // create a writer for the builder
        let mut reduced = Vec::new();
        let mut writer = BitWriter::new(&mut reduced);

        // build from the example sps
        original_sps.build(&mut writer)?;
        writer.finish()?;
        Ok(reduced)
    }

    /// Returns the total byte size of the Sps struct.
    pub fn size(&self) -> u64 {
        // the emulation bytes
        self.emu_bytes as u64 +
        (
        1 + // forbidden zero bit
        2 + // nal_ref_idc
        5 + // nal_unit_type
        8 + // profile_idc
        8 + // 6 constraint_setn_flags + 2 reserved bits
        8 + // level_idc
        size_of_exp_golomb(self.seq_parameter_set_id as u64) +
        self.ext.as_ref().map_or(0, |ext| ext.bitsize()) +
        size_of_exp_golomb(self.log2_max_frame_num_minus4 as u64) +
        size_of_exp_golomb(self.pic_order_cnt_type as u64) +
        match self.pic_order_cnt_type {
            0 => size_of_exp_golomb(self.log2_max_pic_order_cnt_lsb_minus4.unwrap() as u64),
            1 => self.pic_order_cnt_type1.as_ref().unwrap().bitsize(),
            _ => 0
        } +
        size_of_exp_golomb(self.max_num_ref_frames as u64) +
        1 + // gaps_in_frame_num_value_allowed_flag
        size_of_exp_golomb(self.pic_width_in_mbs_minus1) +
        size_of_exp_golomb(self.pic_height_in_map_units_minus1) +
        1 + // frame_mbs_only_flag
        self.mb_adaptive_frame_field_flag.is_some() as u64 +
        1 + // direct_8x8_inference_flag
        1 + // frame_cropping_flag
        self.frame_crop_info.as_ref().map_or(0, |frame| frame.bitsize()) +
        1 + // vui_parameters_present_flag
        if matches!(
            (&self.sample_aspect_ratio, &self.overscan_appropriate_flag, &self.color_config, &self.chroma_sample_loc, &self.timing_info),
            (None, None, None, None, None)
        ) {
            0
        } else {
            self.sample_aspect_ratio.as_ref().map_or(1, |sar| 1 + sar.bitsize()) +
            self.overscan_appropriate_flag.map_or(1, |_| 2) +
            self.color_config.as_ref().map_or(1, |color| 1 + color.bitsize()) +
            self.chroma_sample_loc.as_ref().map_or(1, |chroma| 1 + chroma.bitsize()) +
            self.timing_info.as_ref().map_or(1, |timing| 1 + timing.bitsize())
        }).div_ceil(8)
    }

    /// The height as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16) -
    /// frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// We don't directly store `frame_mbs_only_flag` since we can tell if it's set:
    /// If `mb_adaptive_frame_field_flag` is None, then `frame_mbs_only_flag` is set (1).
    /// Otherwise `mb_adaptive_frame_field_flag` unset (0).
    pub fn height(&self) -> u64 {
        let base_height =
            (2 - self.mb_adaptive_frame_field_flag.is_none() as u64) * (self.pic_height_in_map_units_minus1 + 1) * 16;

        self.frame_crop_info.as_ref().map_or(base_height, |crop| {
            base_height - (crop.frame_crop_top_offset + crop.frame_crop_bottom_offset) * 2
        })
    }

    /// The width as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    pub fn width(&self) -> u64 {
        let base_width = (self.pic_width_in_mbs_minus1 + 1) * 16;

        self.frame_crop_info.as_ref().map_or(base_width, |crop| {
            base_width - (crop.frame_crop_left_offset + crop.frame_crop_right_offset) * 2
        })
    }

    /// Returns the frame rate as a f64.
    ///
    /// If `timing_info_present_flag` is set, then the `frame_rate` will be computed, and
    /// if `num_units_in_tick` is nonzero, then the framerate will be:
    /// `frame_rate = time_scale as f64 / (2.0 * num_units_in_tick as f64)`
    pub fn frame_rate(&self) -> f64 {
        self.timing_info.as_ref().map_or(0.0, |timing| {
            timing.time_scale.get() as f64 / (2.0 * timing.num_units_in_tick.get() as f64)
        })
    }
}

/// The Sequence Parameter Set extension.
/// ISO/IEC-14496-10-2022 - 7.3.2
#[derive(Debug, Clone, PartialEq)]
pub struct SpsExtended {
    /// The `chroma_format_idc` as a u8. This is the chroma sampling relative
    /// to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 3\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `3` which is encoded as `0 0100`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_format_idc: u8,

    /// The `separate_colour_plane_flag` is a single bit.
    ///
    /// 0 means the the color components aren't coded separately and `ChromaArrayType` is set to `chroma_format_idc`.
    ///
    /// 1 means the 3 color components of the 4:4:4 chroma format are coded separately and
    /// `ChromaArrayType` is set to 0.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub separate_color_plane_flag: bool,

    /// The `bit_depth_luma_minus8` as a u8. This is the chroma sampling relative
    /// to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 6\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `6` which is encoded as `0 0111`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub bit_depth_luma_minus8: u8,

    /// The `bit_depth_chroma_minus8` as a u8. This is the chroma sampling
    /// relative to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 6\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `6` which is encoded as `0 0111`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub bit_depth_chroma_minus8: u8,

    /// The `qpprime_y_zero_transform_bypass_flag` is a single bit.
    ///
    /// 0 means the transform coefficient decoding and picture construction processes wont
    /// use the transform bypass operation.
    ///
    /// 1 means that when QP'_Y is 0 then a transform bypass operation for the transform
    /// coefficient decoding and picture construction processes will be applied before
    /// the deblocking filter process from subclause 8.5.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub qpprime_y_zero_transform_bypass_flag: bool,

    /// The `scaling_matrix`. If the length is nonzero, then
    /// `seq_scaling_matrix_present_flag` must have been set.
    pub scaling_matrix: Vec<Vec<i64>>,
}

impl Default for SpsExtended {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SpsExtended {
    // default values defined in 7.4.2.1.1
    const DEFAULT: SpsExtended = SpsExtended {
        chroma_format_idc: 1,
        separate_color_plane_flag: false,
        bit_depth_luma_minus8: 0,
        bit_depth_chroma_minus8: 0,
        qpprime_y_zero_transform_bypass_flag: false,
        scaling_matrix: vec![],
    };

    /// Parses an extended SPS from a bitstream.
    /// Returns an `SpsExtended` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let chroma_format_idc = reader.read_exp_golomb()? as u8;
        // Defaults to false: ISO/IEC-14496-10-2022 - 7.4.2.1.1
        let mut separate_color_plane_flag = false;
        if chroma_format_idc == 3 {
            separate_color_plane_flag = reader.read_bit()?;
        }

        let bit_depth_luma_minus8 = reader.read_exp_golomb()? as u8;
        let bit_depth_chroma_minus8 = reader.read_exp_golomb()? as u8;
        let qpprime_y_zero_transform_bypass_flag = reader.read_bit()?;
        let seq_scaling_matrix_present_flag = reader.read_bit()?;
        let mut scaling_matrix: Vec<Vec<i64>> = vec![];

        if seq_scaling_matrix_present_flag {
            // We need to read the scaling matrices here, but we don't need them
            // for decoding, so we just skip them.
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for i in 0..count {
                let bit = reader.read_bit()?;
                scaling_matrix.push(vec![]);
                if bit {
                    let size = if i < 6 { 16 } else { 64 };
                    let mut next_scale = 8;
                    for _ in 0..size {
                        let delta_scale = reader.read_signed_exp_golomb()?;
                        scaling_matrix[i].push(delta_scale);
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
            separate_color_plane_flag,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag,
            scaling_matrix,
        })
    }

    /// Builds the SPSExtended struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_exp_golomb(self.chroma_format_idc as u64)?;

        if self.chroma_format_idc == 3 {
            writer.write_bit(self.separate_color_plane_flag)?;
        }

        writer.write_exp_golomb(self.bit_depth_luma_minus8 as u64)?;
        writer.write_exp_golomb(self.bit_depth_chroma_minus8 as u64)?;
        writer.write_bit(self.qpprime_y_zero_transform_bypass_flag)?;

        writer.write_bit(!self.scaling_matrix.is_empty())?;

        for vec in &self.scaling_matrix {
            writer.write_bit(!vec.is_empty())?;

            for expg in vec {
                writer.write_signed_exp_golomb(*expg)?;
            }
        }
        Ok(())
    }

    /// Returns the total bits of the SpsExtended struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        size_of_exp_golomb(self.chroma_format_idc as u64) +
        (self.chroma_format_idc == 3) as u64 +
        size_of_exp_golomb(self.bit_depth_luma_minus8 as u64) +
        size_of_exp_golomb(self.bit_depth_chroma_minus8 as u64) +
        1 + // qpprime_y_zero_transform_bypass_flag
        1 + // scaling_matrix.is_empty()
        // scaling matrix
        self.scaling_matrix.len() as u64 +
        self.scaling_matrix.iter().flat_map(|inner| inner.iter()).map(|&x| size_of_signed_exp_golomb(x)).sum::<u64>()
    }

    /// Returns the total bytes of the SpsExtended struct.
    ///
    /// Note that this calls [`SpsExtended::bitsize()`] and calculates the number of bytes
    /// including any necessary padding such that the bitstream is byte aligned.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

/// `PicOrderCountType1` contains the fields that are set when `pic_order_cnt_type == 1`.
///
/// This contains the following fields: `delta_pic_order_always_zero_flag`,
/// `offset_for_non_ref_pic`, `offset_for_top_to_bottom_field`, and
/// `offset_for_ref_frame`.
#[derive(Debug, Clone, PartialEq)]
pub struct PicOrderCountType1 {
    /// The `delta_pic_order_always_zero_flag` is a single bit.
    ///
    /// 0 means the `delta_pic_order_cnt[0]` is in the slice headers and `delta_pic_order_cnt[1]`
    /// might not be in the slice headers.
    ///
    /// 1 means the `delta_pic_order_cnt[0]` and `delta_pic_order_cnt[1]` are NOT in the slice headers
    /// and will be set to 0 by default.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub delta_pic_order_always_zero_flag: bool,

    /// The `offset_for_non_ref_pic` is used to calculate the pic order count for a non-reference picture
    /// from subclause 8.2.1.
    ///
    /// The value of this ranges from \[-2^(31), 2^(31) - 1\].
    ///
    /// This is a variable number of bits as it is encoded by a SIGNED exp golomb.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub offset_for_non_ref_pic: i64,

    /// The `offset_for_top_to_bottom_field` is used to calculate the pic order count of a bottom field from
    /// subclause 8.2.1.
    ///
    /// The value of this ranges from \[-2^(31), 2^(31) - 1\].
    ///
    /// This is a variable number of bits as it is encoded by a SIGNED exp golomb.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub offset_for_top_to_bottom_field: i64,

    /// The `num_ref_frames_in_pic_order_cnt_cycle` is used in the decoding process for the picture order
    /// count in 8.2.1.
    ///
    /// The value of this ranges from \[0, 255\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `255` which is encoded as `0 0000 0001 0000 0000`, which is 17 bits.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub num_ref_frames_in_pic_order_cnt_cycle: u64,

    /// The `offset_for_ref_frame` is a vec where each value used in decoding the picture order count
    /// from subclause 8.2.1.
    ///
    /// When `pic_order_cnt_type == 1`, `ExpectedDeltaPerPicOrderCntCycle` can be derived by:
    /// ```python
    /// ExpectedDeltaPerPicOrderCntCycle = sum(offset_for_ref_frame)
    /// ```
    ///
    /// The value of this ranges from \[-2^(31), 2^(31) - 1\].
    ///
    /// This is a variable number of bits as it is encoded by a SIGNED exp golomb.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub offset_for_ref_frame: Vec<i64>,
}

impl PicOrderCountType1 {
    /// Parses the fields defined when the `pic_order_count_type == 1` from a bitstream.
    /// Returns a `PicOrderCountType1` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let delta_pic_order_always_zero_flag = reader.read_bit()?;
        let offset_for_non_ref_pic = reader.read_signed_exp_golomb()?;
        let offset_for_top_to_bottom_field = reader.read_signed_exp_golomb()?;
        let num_ref_frames_in_pic_order_cnt_cycle = reader.read_exp_golomb()?;

        let mut offset_for_ref_frame = vec![];
        for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
            offset_for_ref_frame.push(reader.read_signed_exp_golomb()?);
        }

        Ok(PicOrderCountType1 {
            delta_pic_order_always_zero_flag,
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frame,
        })
    }

    /// Builds the PicOrderCountType1 struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_bit(self.delta_pic_order_always_zero_flag)?;
        writer.write_signed_exp_golomb(self.offset_for_non_ref_pic)?;
        writer.write_signed_exp_golomb(self.offset_for_top_to_bottom_field)?;
        writer.write_exp_golomb(self.num_ref_frames_in_pic_order_cnt_cycle)?;

        for num in &self.offset_for_ref_frame {
            writer.write_signed_exp_golomb(*num)?;
        }
        Ok(())
    }

    /// Returns the total bits of the PicOrderCountType1 struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        1 + // delta_pic_order_always_zero_flag
        size_of_signed_exp_golomb(self.offset_for_non_ref_pic) +
        size_of_signed_exp_golomb(self.offset_for_top_to_bottom_field) +
        size_of_exp_golomb(self.num_ref_frames_in_pic_order_cnt_cycle) +
        self.offset_for_ref_frame.iter().map(|x| size_of_signed_exp_golomb(*x)).sum::<u64>()
    }

    /// Returns the total bytes of the PicOrderCountType1 struct.
    ///
    /// Note that this calls [`PicOrderCountType1::bitsize()`] and calculates the number of bytes
    /// including any necessary padding such that the bitstream is byte aligned.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

/// `FrameCropInfo` contains the frame cropping info.
///
/// This includes `frame_crop_left_offset`, `frame_crop_right_offset`, `frame_crop_top_offset`,
/// and `frame_crop_bottom_offset`.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameCropInfo {
    /// The `frame_crop_left_offset` is the the left crop offset which is used to compute the width:
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_left_offset: u64,

    /// The `frame_crop_right_offset` is the the right crop offset which is used to compute the width:
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_right_offset: u64,

    /// The `frame_crop_top_offset` is the the top crop offset which is used to compute the height:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
    /// - frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_top_offset: u64,

    /// The `frame_crop_bottom_offset` is the the bottom crop offset which is used to compute the height:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
    /// - frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_bottom_offset: u64,
}

impl FrameCropInfo {
    /// Parses the fields defined when the `frame_cropping_flag == 1` from a bitstream.
    /// Returns a `FrameCropInfo` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let frame_crop_left_offset = reader.read_exp_golomb()?;
        let frame_crop_right_offset = reader.read_exp_golomb()?;
        let frame_crop_top_offset = reader.read_exp_golomb()?;
        let frame_crop_bottom_offset = reader.read_exp_golomb()?;

        Ok(FrameCropInfo {
            frame_crop_left_offset,
            frame_crop_right_offset,
            frame_crop_top_offset,
            frame_crop_bottom_offset,
        })
    }

    /// Builds the FrameCropInfo struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_exp_golomb(self.frame_crop_left_offset)?;
        writer.write_exp_golomb(self.frame_crop_right_offset)?;
        writer.write_exp_golomb(self.frame_crop_top_offset)?;
        writer.write_exp_golomb(self.frame_crop_bottom_offset)?;
        Ok(())
    }

    /// Returns the total bits of the FrameCropInfo struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        size_of_exp_golomb(self.frame_crop_left_offset)
            + size_of_exp_golomb(self.frame_crop_right_offset)
            + size_of_exp_golomb(self.frame_crop_top_offset)
            + size_of_exp_golomb(self.frame_crop_bottom_offset)
    }

    /// Returns the total bytes of the FrameCropInfo struct.
    ///
    /// Note that this calls [`FrameCropInfo::bitsize()`] and calculates the number of bytes
    /// including any necessary padding such that the bitstream is byte aligned.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

/// `SarDimensions` contains the fields that are set when `aspect_ratio_info_present_flag == 1`,
/// and `aspect_ratio_idc == 255`.
///
/// This contains the following fields: `sar_width` and `sar_height`.
#[derive(Debug, Clone, PartialEq)]
pub struct SarDimensions {
    /// The `aspect_ratio_idc` is the sample aspect ratio of the luma samples as a u8.
    ///
    /// This is a full byte, and defaults to 0.
    ///
    /// Refer to the `AspectRatioIdc` nutype enum for more info.
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1 Table E-1
    pub aspect_ratio_idc: AspectRatioIdc,

    /// The `sar_width` is the horizontal size of the aspect ratio as a u16.
    ///
    /// This is a full 2 bytes.
    ///
    /// The value is supposed to be "relatively prime or equal to 0". If set to 0,
    /// the sample aspect ratio is considered to be unspecified by ISO/IEC-14496-10-2022.
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub sar_width: u16,

    /// The `offset_for_non_ref_pic` is the vertical size of the aspect ratio as a u16.
    ///
    /// This is a full 2 bytes.
    ///
    /// The value is supposed to be "relatively prime or equal to 0". If set to 0,
    /// the sample aspect ratio is considered to be unspecified by ISO/IEC-14496-10-2022.
    ///
    /// The value is supposed to be "relatively prime or equal to 0".
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub sar_height: u16,
}

impl SarDimensions {
    /// Parses the fields defined when the `aspect_ratio_info_present_flag == 1` from a bitstream.
    /// Returns a `SarDimensions` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let mut sar_width = 0; // defaults to 0, E.2.1
        let mut sar_height = 0; // deafults to 0, E.2.1

        let aspect_ratio_idc = reader.read_u8()?;
        if aspect_ratio_idc == 255 {
            sar_width = reader.read_bits(16)? as u16;
            sar_height = reader.read_bits(16)? as u16;
        }

        Ok(SarDimensions {
            aspect_ratio_idc: AspectRatioIdc(aspect_ratio_idc),
            sar_width,
            sar_height,
        })
    }

    /// Builds the SarDimensions struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_bits(self.aspect_ratio_idc.into(), 8)?;

        if self.aspect_ratio_idc == AspectRatioIdc(255) {
            writer.write_bits(self.sar_width as u64, 16)?;
            writer.write_bits(self.sar_height as u64, 16)?;
        }
        Ok(())
    }

    /// Returns the total bits of the SarDimensions struct.
    pub fn bitsize(&self) -> u64 {
        8 + // aspect_ratio_idc
        ((self.aspect_ratio_idc == AspectRatioIdc(255)) as u64) * 32
    }

    /// Returns the total bytes of the SarDimensions struct.
    ///
    /// Note that this calls [`SarDimensions::bitsize()`] and calculates the number of bytes.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

/// The color config for SPS. ISO/IEC-14496-10-2022 - E.2.1
#[derive(Debug, Clone, PartialEq)]
pub struct ColorConfig {
    /// The `video_format` is comprised of 3 bits stored as a u8.
    ///
    /// Refer to the `VideoFormat` nutype enum for more info.
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1 Table E-2
    pub video_format: VideoFormat,

    /// The `video_full_range_flag` is a single bit indicating the black level and range of
    /// luma and chroma signals.
    ///
    /// This field is passed into the `ColorConfig`.
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub video_full_range_flag: bool,

    /// The `colour_primaries` byte as a u8. If `color_description_present_flag` is not set,
    /// the value defaults to 2. ISO/IEC-14496-10-2022 - E.2.1 Table E-3
    pub color_primaries: u8,

    /// The `transfer_characteristics` byte as a u8. If `color_description_present_flag` is not set,
    /// the value defaults to 2. ISO/IEC-14496-10-2022 - E.2.1 Table E-4
    pub transfer_characteristics: u8,

    /// The `matrix_coefficients` byte as a u8. If `color_description_present_flag` is not set,
    /// the value defaults to 2. ISO/IEC-14496-10-2022 - E.2.1 Table E-5
    pub matrix_coefficients: u8,
}

impl ColorConfig {
    /// Parses the fields defined when the `video_signal_type_present_flag == 1` from a bitstream.
    /// Returns a `ColorConfig` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let video_format = reader.read_bits(3)? as u8;
        let video_full_range_flag = reader.read_bit()?;

        let color_primaries;
        let transfer_characteristics;
        let matrix_coefficients;

        let color_description_present_flag = reader.read_bit()?;
        if color_description_present_flag {
            color_primaries = reader.read_u8()?;
            transfer_characteristics = reader.read_u8()?;
            matrix_coefficients = reader.read_u8()?;
        } else {
            color_primaries = 2; // UNSPECIFIED
            transfer_characteristics = 2; // UNSPECIFIED
            matrix_coefficients = 2; // UNSPECIFIED
        }

        Ok(ColorConfig {
            video_format: VideoFormat(video_format), // defalut value is 5 E.2.1 Table E-2
            video_full_range_flag,
            color_primaries,
            transfer_characteristics,
            matrix_coefficients,
        })
    }

    /// Builds the ColorConfig struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_bits(self.video_format.into(), 3)?;
        writer.write_bit(self.video_full_range_flag)?;

        match (self.color_primaries, self.transfer_characteristics, self.matrix_coefficients) {
            (2, 2, 2) => {
                writer.write_bit(false)?;
            }
            (color_priamries, transfer_characteristics, matrix_coefficients) => {
                writer.write_bit(true)?;
                writer.write_bits(color_priamries as u64, 8)?;
                writer.write_bits(transfer_characteristics as u64, 8)?;
                writer.write_bits(matrix_coefficients as u64, 8)?;
            }
        }
        Ok(())
    }

    /// Returns the total bits of the ColorConfig struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        3 + // video_format
        1 + // video_full_range_flag
        1 + // color_description_present_flag
        match (self.color_primaries, self.transfer_characteristics, self.matrix_coefficients) {
            (2, 2, 2) => 0,
            _ => 24
        }
    }

    /// Returns the total bytes of the ColorConfig struct.
    ///
    /// Note that this calls [`ColorConfig::bitsize()`] and calculates the number of bytes
    /// including any necessary padding such that the bitstream is byte aligned.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

/// `ChromaSampleLoc` contains the fields that are set when `chroma_loc_info_present_flag == 1`,
///
/// This contains the following fields: `chroma_sample_loc_type_top_field` and `chroma_sample_loc_type_bottom_field`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSampleLoc {
    /// The `chroma_sample_loc_type_top_field` specifies the location of chroma samples.
    ///
    /// The value of this ranges from \[0, 5\]. By default, this value is set to 0.
    ///
    /// See ISO/IEC-14496-10-2022 - E.2.1 Figure E-1 for more info.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `5` which is encoded as `0 0110`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - E.2.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_sample_loc_type_top_field: u8,

    /// The `chroma_sample_loc_type_bottom_field`
    ///
    /// The value of this ranges from \[0, 5\]. By default, this value is set to 0.
    ///
    /// See ISO/IEC-14496-10-2022 - E.2.1 Figure E-1 for more info.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `5` which is encoded as `0 0110`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - E.2.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_sample_loc_type_bottom_field: u8,
}

impl ChromaSampleLoc {
    /// Parses the fields defined when the `chroma_loc_info_present_flag == 1` from a bitstream.
    /// Returns a `ChromaSampleLoc` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let chroma_sample_loc_type_top_field = reader.read_exp_golomb()? as u8;
        let chroma_sample_loc_type_bottom_field = reader.read_exp_golomb()? as u8;

        Ok(ChromaSampleLoc {
            chroma_sample_loc_type_top_field,
            chroma_sample_loc_type_bottom_field,
        })
    }

    /// Builds the ChromaSampleLoc struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_exp_golomb(self.chroma_sample_loc_type_top_field as u64)?;
        writer.write_exp_golomb(self.chroma_sample_loc_type_bottom_field as u64)?;
        Ok(())
    }

    /// Returns the total bits of the ChromaSampleLoc struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        size_of_exp_golomb(self.chroma_sample_loc_type_top_field as u64)
            + size_of_exp_golomb(self.chroma_sample_loc_type_bottom_field as u64)
    }

    /// Returns the total bytes of the ChromaSampleLoc struct.
    ///
    /// Note that this calls [`ChromaSampleLoc::bitsize()`] and calculates the number of bytes
    /// including any necessary padding such that the bitstream is byte aligned.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

/// `TimingInfo` contains the fields that are set when `timing_info_present_flag == 1`.
///
/// This contains the following fields: `num_units_in_tick` and `time_scale`.
///
/// ISO/IEC-14496-10-2022 - E.2.1
///
/// Refer to the direct fields for more information.
#[derive(Debug, Clone, PartialEq)]
pub struct TimingInfo {
    /// The `num_units_in_tick` is the smallest unit used to measure time.
    ///
    /// It is used alongside `time_scale` to compute the `frame_rate` as follows:
    ///
    /// `frame_rate = time_scale / (2 * num_units_in_tick)`
    ///
    /// It must be greater than 0, therefore, it is a `NonZeroU32`. If it isn't provided,
    /// the value is defaulted to None instead of 0.
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub num_units_in_tick: NonZeroU32,

    /// The `time_scale` is the number of time units that pass in 1 second (hz).
    ///
    /// It is used alongside `num_units_in_tick` to compute the `frame_rate` as follows:
    ///
    /// `frame_rate = time_scale / (2 * num_units_in_tick)`
    ///
    /// It must be greater than 0, therefore, it is a `NonZeroU32`. If it isn't provided,
    /// the value is defaulted to None instead of 0.
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub time_scale: NonZeroU32,
}

impl TimingInfo {
    /// Parses the fields defined when the `timing_info_present_flag == 1` from a bitstream.
    /// Returns a `TimingInfo` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let num_units_in_tick = NonZeroU32::new(reader.read_u32::<BigEndian>()?)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "num_units_in_tick cannot be 0"))?;

        let time_scale = NonZeroU32::new(reader.read_u32::<BigEndian>()?)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "time_scale cannot be 0"))?;

        Ok(TimingInfo {
            num_units_in_tick,
            time_scale,
        })
    }

    /// Builds the TimingInfo struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_bits(self.num_units_in_tick.get() as u64, 32)?;
        writer.write_bits(self.time_scale.get() as u64, 32)?;
        Ok(())
    }

    /// Returns the total bits of the TimingInfo struct. It is always 64 bits (8 bytes).
    pub fn bitsize(&self) -> u64 {
        64
    }

    /// Returns the total bytes of the TimingInfo struct. It is always 8 bytes (64 bits).
    pub fn bytesize(&self) -> u64 {
        8
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use scuffle_bytes_util::{BitReader, BitWriter};
    use scuffle_expgolomb::{BitWriterExpGolombExt, size_of_exp_golomb, size_of_signed_exp_golomb};

    use super::TimingInfo;
    use crate::sps::{ChromaSampleLoc, FrameCropInfo, PicOrderCountType1, SarDimensions, Sps};
    use crate::{ColorConfig, SpsExtended};

    #[test]
    fn test_parse_sps_insufficient_bytes_() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        writer.write_bit(true).unwrap(); // only write 1 bit
        writer.finish().unwrap();

        let result = Sps::parse(&sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Insufficient data: SPS must be at least 4 bytes long");
    }

    #[test]
    fn test_parse_sps_set_forbidden_bit() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        writer.write_bit(true).unwrap(); // sets the forbidden bit
        writer.write_bits(0x00, 8).unwrap(); // ensure length > 3 bytes
        writer.write_bits(0x00, 8).unwrap(); // ensure length > 3 bytes
        writer.write_bits(0x00, 8).unwrap(); // ensure length > 3 bytes
        writer.finish().unwrap();

        let result = Sps::parse(&sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Forbidden zero bit is set");
    }

    #[test]
    fn test_parse_sps_invalid_nal() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        writer.write_bit(false).unwrap(); // forbidden zero bit must be unset
        writer.write_bits(0b00, 2).unwrap(); // nal_ref_idc is 00
        writer.write_bits(0b000, 3).unwrap(); // set nal_unit_type to something that isn't 7

        writer.write_bits(0x00, 8).unwrap(); // ensure length > 3 bytes
        writer.write_bits(0x00, 8).unwrap(); // ensure length > 3 bytes
        writer.write_bits(0x00, 8).unwrap(); // ensure length > 3 bytes
        writer.finish().unwrap();

        let result = Sps::parse(&sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "NAL unit type is not SPS");
    }

    #[test]
    fn test_parse_build_sps_4k_144fps() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 100
        writer.write_bits(100, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // sps ext
        // chroma_format_idc is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // seq_scaling_matrix_present_flag
        writer.write_bit(false).unwrap();

        // back to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4 is expg
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 3840 width:
        // 3840 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // 3840 = (p + 1) * 16
        // p = 239
        writer.write_exp_golomb(239).unwrap();
        // we want 2160 height:
        // 2160 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // m is frame_mbs_only_flag which we set to 1 later
        // 2160 = (2 - 1) * (p + 1) * 16
        // 2160 = (p + 1) * 16
        // p = 134
        writer.write_exp_golomb(134).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(true).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(true).unwrap();
        // we want square (1:1) for 16:9 for 4k w/o overscan
        // aspect_ratio_idc
        writer.write_bits(1, 8).unwrap();

        // overscan_info_present_flag
        writer.write_bit(true).unwrap();
        // we dont want overscan
        // overscan_appropriate_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(false).unwrap();
        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();

        // timing_info_present_flag
        writer.write_bit(true).unwrap();
        // we can set this to 100 for example
        // num_units_in_tick is a u32
        writer.write_bits(100, 32).unwrap();
        // fps = time_scale / (2 * num_units_in_tick)
        // since we want 144 fps:
        // 144 = time_scale / (2 * 100)
        // 28800 = time_scale
        // time_scale is a u32
        writer.write_bits(28800, 32).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(&sps).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            emu_bytes: 0,
            forbidden_zero_bit: false,
            nal_ref_idc: 0,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 100,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 0,
            seq_parameter_set_id: 0,
            ext: Some(
                SpsExtended {
                    chroma_format_idc: 0,
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
                0,
            ),
            pic_order_cnt_type1: None,
            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 239,
            pic_height_in_map_units_minus1: 134,
            mb_adaptive_frame_field_flag: None,
            direct_8x8_inference_flag: false,
            frame_crop_info: None,
            sample_aspect_ratio: Some(
                SarDimensions {
                    aspect_ratio_idc: AspectRatioIdc::Square,
                    sar_width: 0,
                    sar_height: 0,
                },
            ),
            overscan_appropriate_flag: Some(
                false,
            ),
            color_config: None,
            chroma_sample_loc: None,
            timing_info: Some(
                TimingInfo {
                    num_units_in_tick: 100,
                    time_scale: 28800,
                },
            ),
        }
        ");

        assert_eq!(144.0, result.frame_rate());
        assert_eq!(3840, result.width());
        assert_eq!(2160, result.height());

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example sps
        result.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        // sometimes bits can get lost because we save
        // some space with how the SPS is rebuilt.
        // so we can just confirm that they're the same
        // by rebuilding it.
        let reduced = Sps::parse(&buf).unwrap(); // <- this is where things break
        assert_eq!(reduced, result);

        // now we can check that the bitstream from
        // the reduced version should be the same
        let mut reduced_buf = Vec::new();
        let mut writer3 = BitWriter::new(&mut reduced_buf);

        reduced.build(&mut writer3).unwrap();
        writer3.finish().unwrap();
        assert_eq!(reduced_buf, buf);

        // now we can check the size:
        assert_eq!(reduced.size(), result.size());
    }

    #[test]
    fn test_parse_build_sps_1080_480fps_scaling_matrix() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 44
        writer.write_bits(44, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();
        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // sps ext
        // we want to try out chroma_format_idc = 3
        // chroma_format_idc is expg
        writer.write_exp_golomb(3).unwrap();
        // separate_color_plane_flag
        writer.write_bit(false).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // we want to simulate a scaling matrix
        // seq_scaling_matrix_present_flag
        writer.write_bit(true).unwrap();

        // enter scaling matrix, we loop 12 times since
        // chroma_format_idc = 3.
        // loop 1 of 12
        // true to enter if statement
        writer.write_bit(true).unwrap();
        // i < 6, so size is 16, so we loop 16 times
        // sub-loop 1 of 16
        // delta_scale is a SIGNED expg so we can try out
        // entering -4 so next_scale becomes 8 + 4 = 12
        writer.write_signed_exp_golomb(4).unwrap();
        // sub-loop 2 of 16
        // delta_scale is a SIGNED expg so we can try out
        // entering -12 so next scale becomes 12 - 12 = 0
        writer.write_signed_exp_golomb(-12).unwrap();
        // at this point next_scale is 0, which means we break
        // loop 2 through 12
        // we don't need to try anything else so we can just skip through them by writing `0` bit 11 times.
        writer.write_bits(0, 11).unwrap();

        // back to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // we can try setting pic_order_cnt_type to 1
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(1).unwrap();

        // delta_pic_order_always_zero_flag
        writer.write_bit(false).unwrap();
        // offset_for_non_ref_pic
        writer.write_bit(true).unwrap();
        // offset_for_top_to_bottom_field
        writer.write_bit(true).unwrap();
        // num_ref_frames_in_pic_order_cnt_cycle is expg
        writer.write_exp_golomb(1).unwrap();
        // loop num_ref_frames_in_pic_order_cnt_cycle times (1)
        // offset_for_ref_frame is expg
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 1920 width:
        // 1920 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 4 later
        // 1920 = (p + 1) * 16 - 2 * 4 - 2 * 4
        // 1920 = (p + 1) * 16 - 16
        // p = 120
        // pic_width_in_mbs_minus1 is expg
        writer.write_exp_golomb(120).unwrap();
        // we want 1080 height:
        // 1080 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 2 later
        // m is frame_mbs_only_flag which we set to 0 later
        // 1080 = (2 - 0) * (p + 1) * 16 - 2 * 2 - 2 * 2
        // 1080 = 2 * (p + 1) * 16 - 8
        // p = 33
        // pic_height_in_map_units_minus1 is expg
        writer.write_exp_golomb(33).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(true).unwrap();

        // frame_crop_left_offset is expg
        writer.write_exp_golomb(4).unwrap();
        // frame_crop_left_offset is expg
        writer.write_exp_golomb(4).unwrap();
        // frame_crop_left_offset is expg
        writer.write_exp_golomb(2).unwrap();
        // frame_crop_left_offset is expg
        writer.write_exp_golomb(2).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(true).unwrap();
        // we can try 255 to set the sar_width and sar_height
        // aspect_ratio_idc
        writer.write_bits(255, 8).unwrap();
        // sar_width
        writer.write_bits(0, 16).unwrap();
        // sar_height
        writer.write_bits(0, 16).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();
        // video_format
        writer.write_bits(0, 3).unwrap();
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        // color_description_present_flag
        writer.write_bit(true).unwrap();
        // color_primaries
        writer.write_bits(1, 8).unwrap();
        // transfer_characteristics
        writer.write_bits(1, 8).unwrap();
        // matrix_coefficients
        writer.write_bits(1, 8).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();

        // timing_info_present_flag
        writer.write_bit(true).unwrap();
        // we can set this to 1000 for example
        // num_units_in_tick is a u32
        writer.write_bits(1000, 32).unwrap();
        // fps = time_scale / (2 * num_units_in_tick)
        // since we want 480 fps:
        // 480 = time_scale / (2 * 1000)
        // 960 000 = time_scale
        // time_scale is a u32
        writer.write_bits(960000, 32).unwrap();
        writer.finish().unwrap();

        // we "reduce" it which adds an emulation byte which the above sps is supposed to have.
        let fixed_sps = Sps::reduce(&sps).unwrap();

        let result = Sps::parse(&fixed_sps).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            emu_bytes: 1,
            forbidden_zero_bit: false,
            nal_ref_idc: 0,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 44,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 0,
            seq_parameter_set_id: 0,
            ext: Some(
                SpsExtended {
                    chroma_format_idc: 3,
                    separate_color_plane_flag: false,
                    bit_depth_luma_minus8: 0,
                    bit_depth_chroma_minus8: 0,
                    qpprime_y_zero_transform_bypass_flag: false,
                    scaling_matrix: [
                        [
                            4,
                            -12,
                        ],
                        [],
                        [],
                        [],
                        [],
                        [],
                        [],
                        [],
                        [],
                        [],
                        [],
                        [],
                    ],
                },
            ),
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 1,
            log2_max_pic_order_cnt_lsb_minus4: None,
            pic_order_cnt_type1: Some(
                PicOrderCountType1 {
                    delta_pic_order_always_zero_flag: false,
                    offset_for_non_ref_pic: 0,
                    offset_for_top_to_bottom_field: 0,
                    num_ref_frames_in_pic_order_cnt_cycle: 1,
                    offset_for_ref_frame: [
                        0,
                    ],
                },
            ),
            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 120,
            pic_height_in_map_units_minus1: 33,
            mb_adaptive_frame_field_flag: Some(
                false,
            ),
            direct_8x8_inference_flag: false,
            frame_crop_info: Some(
                FrameCropInfo {
                    frame_crop_left_offset: 4,
                    frame_crop_right_offset: 4,
                    frame_crop_top_offset: 2,
                    frame_crop_bottom_offset: 2,
                },
            ),
            sample_aspect_ratio: Some(
                SarDimensions {
                    aspect_ratio_idc: AspectRatioIdc::ExtendedSar,
                    sar_width: 0,
                    sar_height: 0,
                },
            ),
            overscan_appropriate_flag: None,
            color_config: Some(
                ColorConfig {
                    video_format: VideoFormat::Component,
                    video_full_range_flag: false,
                    color_primaries: 1,
                    transfer_characteristics: 1,
                    matrix_coefficients: 1,
                },
            ),
            chroma_sample_loc: None,
            timing_info: Some(
                TimingInfo {
                    num_units_in_tick: 1000,
                    time_scale: 960000,
                },
            ),
        }
        ");

        assert_eq!(480.0, result.frame_rate());
        assert_eq!(1920, result.width());
        assert_eq!(1080, result.height());

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example sps
        result.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        // sometimes bits can get lost because we save
        // some space with how the SPS is rebuilt.
        // so we can just confirm that they're the same
        // by rebuilding it.
        let reduced = Sps::parse(&buf).unwrap();
        assert_eq!(reduced, result);

        // now we can check that the bitstream from
        // the reduced version should be the same
        let mut reduced_buf = Vec::new();
        let mut writer3 = BitWriter::new(&mut reduced_buf);

        reduced.build(&mut writer3).unwrap();
        writer3.finish().unwrap();
        assert_eq!(reduced_buf, buf);

        // now we can check the size:
        assert_eq!(reduced.size(), result.size());
    }

    #[test]
    fn test_parse_build_sps_1280x800_0fps() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 77
        writer.write_bits(77, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // profile_idc = 77 means we skip the sps_ext
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 1280 width:
        // 1280 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // 1280 = (p + 1) * 16
        // p = 79
        writer.write_exp_golomb(79).unwrap();
        // we want 800 height:
        // 800 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // m is frame_mbs_only_flag which we set to 1 later
        // 800 = (2 - 1) * (p + 1) * 16 - 2 * 0 - 2 * 0
        // 800 = (p + 1) * 16
        // p = 49
        writer.write_exp_golomb(49).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(true).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(false).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();
        // video_format
        writer.write_bits(0, 3).unwrap();
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        // color_description_present_flag
        writer.write_bit(false).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(true).unwrap();
        // chroma_sample_loc_type_top_field is expg
        writer.write_exp_golomb(2).unwrap();
        // chroma_sample_loc_type_bottom_field is expg
        writer.write_exp_golomb(2).unwrap();

        // timing_info_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(&sps).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            emu_bytes: 0,
            forbidden_zero_bit: false,
            nal_ref_idc: 0,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 77,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 0,
            seq_parameter_set_id: 0,
            ext: None,
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 0,
            log2_max_pic_order_cnt_lsb_minus4: Some(
                0,
            ),
            pic_order_cnt_type1: None,
            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 79,
            pic_height_in_map_units_minus1: 49,
            mb_adaptive_frame_field_flag: None,
            direct_8x8_inference_flag: false,
            frame_crop_info: None,
            sample_aspect_ratio: None,
            overscan_appropriate_flag: None,
            color_config: Some(
                ColorConfig {
                    video_format: VideoFormat::Component,
                    video_full_range_flag: false,
                    color_primaries: 2,
                    transfer_characteristics: 2,
                    matrix_coefficients: 2,
                },
            ),
            chroma_sample_loc: Some(
                ChromaSampleLoc {
                    chroma_sample_loc_type_top_field: 2,
                    chroma_sample_loc_type_bottom_field: 2,
                },
            ),
            timing_info: None,
        }
        ");

        assert_eq!(0.0, result.frame_rate());
        assert_eq!(1280, result.width());
        assert_eq!(800, result.height());

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example sps
        result.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        // sometimes bits can get lost because we save
        // some space with how the SPS is rebuilt.
        // so we can just confirm that they're the same
        // by rebuilding it.
        let reduced = Sps::parse(&buf).unwrap();
        assert_eq!(reduced, result);

        // now we can check that the bitstream from
        // the reduced version should be the same
        let mut reduced_buf = Vec::new();
        let mut writer3 = BitWriter::new(&mut reduced_buf);

        reduced.build(&mut writer3).unwrap();
        writer3.finish().unwrap();
        assert_eq!(reduced_buf, buf);

        // now we can check the size:
        assert_eq!(reduced.size(), result.size());
    }

    #[test]
    fn test_parse_sps_chroma_loc_info_error() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 100
        writer.write_bits(100, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // ext
        // chroma_format_idc is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // seq_scaling_matrix_present_flag
        writer.write_bit(false).unwrap();

        // return to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 1280 width:
        // 1280 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // 1280 = (p + 1) * 16
        // p = 79
        writer.write_exp_golomb(79).unwrap();
        // we want 800 height:
        // 800 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // m is frame_mbs_only_flag which we set to 1 later
        // 800 = (2 - 1) * (p + 1) * 16 - 2 * 0 - 2 * 0
        // 800 = 2 * (p + 1) * 16 - 8
        // p = 33
        writer.write_exp_golomb(33).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(false).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();
        // video_format
        writer.write_bits(0, 3).unwrap();
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        // color_description_present_flag
        writer.write_bit(false).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(true).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(&sps);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            err.to_string(),
            "chroma_loc_info_present_flag cannot be set to 1 when chroma_format_idc is not 1"
        );
    }

    #[test]
    fn test_invalid_num_units_in_tick() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 100
        writer.write_bits(100, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // ext
        // chroma_format_idc is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // seq_scaling_matrix_present_flag
        writer.write_bit(false).unwrap();

        // return to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 1280 width:
        // 1280 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // 1280 = (p + 1) * 16
        // p = 79
        writer.write_exp_golomb(79).unwrap();
        // we want 800 height:
        // 800 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // m is frame_mbs_only_flag which we set to 1 later
        // 800 = (2 - 1) * (p + 1) * 16 - 2 * 0 - 2 * 0
        // 800 = 2 * (p + 1) * 16 - 8
        // p = 33
        writer.write_exp_golomb(33).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(false).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();
        // video_format
        writer.write_bits(0, 3).unwrap();
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        // color_description_present_flag
        writer.write_bit(false).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();

        // timing_info_present_flag
        writer.write_bit(true).unwrap();
        // num_units_in_tick to 0 (invalid)
        writer.write_bits(0, 32).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(&sps);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "num_units_in_tick cannot be 0");
    }

    #[test]
    fn test_invalid_time_scale() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 100
        writer.write_bits(100, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // ext
        // chroma_format_idc is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // seq_scaling_matrix_present_flag
        writer.write_bit(false).unwrap();

        // return to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 1280 width:
        // 1280 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // 1280 = (p + 1) * 16
        // p = 79
        writer.write_exp_golomb(79).unwrap();
        // we want 800 height:
        // 800 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 0 later
        // m is frame_mbs_only_flag which we set to 1 later
        // 800 = (2 - 1) * (p + 1) * 16 - 2 * 0 - 2 * 0
        // 800 = 2 * (p + 1) * 16 - 8
        // p = 33
        writer.write_exp_golomb(33).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(false).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();
        // video_format
        writer.write_bits(0, 3).unwrap();
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        // color_description_present_flag
        writer.write_bit(false).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();

        // timing_info_present_flag
        writer.write_bit(true).unwrap();
        // num_units_in_tick to 0 (invalid)
        writer.write_bits(0, 32).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(&sps);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "num_units_in_tick cannot be 0");
    }

    #[test]
    fn test_parse_build_sps_no_vui() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 77
        writer.write_bits(77, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg so 0b1 (true) = false
        writer.write_bit(true).unwrap();

        // skip sps ext since profile_idc = 77
        // log2_max_frame_num_minus4 is expg so 0b1 (true) = false
        writer.write_bit(true).unwrap();
        // we can try setting pic_order_cnt_type to 2
        writer.write_exp_golomb(2).unwrap();

        // delta_pic_order_always_zero_flag
        writer.write_bit(false).unwrap();
        // offset_for_non_ref_pic
        writer.write_bit(true).unwrap();
        // offset_for_top_to_bottom_field
        writer.write_bit(true).unwrap();
        // num_ref_frames_in_pic_order_cnt_cycle is expg so 0b010 = 1
        writer.write_bits(0b010, 3).unwrap();
        // loop num_ref_frames_in_pic_order_cnt_cycle times (1)
        // offset_for_ref_frame is expg so 0b1 (true) = false
        writer.write_bit(true).unwrap();

        // max_num_ref_frames is expg so 0b1 (true) = false
        writer.write_bit(true).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // 1920 width:
        // 1920 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 4 later
        // 1920 = (p + 1) * 16 - 2 * 4 - 2 * 4
        // 1920 = (p + 1) * 16 - 16
        // p = 120
        // pic_width_in_mbs_minus1 is expg so:
        // 0 0000 0111 1001
        writer.write_bits(0b0000001111001, 13).unwrap();
        // we want 1080 height:
        // 1080 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 2 later
        // m is frame_mbs_only_flag which we set to 0 later
        // 1080 = (2 - 0) * (p + 1) * 16 - 2 * 2 - 2 * 2
        // 1080 = 2 * (p + 1) * 16 - 8
        // p = 33
        // pic_height_in_map_units_minus1 is expg so:
        // 000 0010 0010
        writer.write_bits(0b00000100010, 11).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(true).unwrap();

        // frame_crop_left_offset is expg
        writer.write_bits(0b00101, 5).unwrap();
        // frame_crop_right_offset is expg
        writer.write_bits(0b00101, 5).unwrap();
        // frame_crop_top_offset is expg
        writer.write_bits(0b011, 3).unwrap();
        // frame_crop_bottom_offset is expg
        writer.write_bits(0b011, 3).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(&sps).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            emu_bytes: 0,
            forbidden_zero_bit: false,
            nal_ref_idc: 0,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 77,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 0,
            seq_parameter_set_id: 0,
            ext: None,
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 2,
            log2_max_pic_order_cnt_lsb_minus4: None,
            pic_order_cnt_type1: None,
            max_num_ref_frames: 2,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 0,
            pic_height_in_map_units_minus1: 2,
            mb_adaptive_frame_field_flag: Some(
                false,
            ),
            direct_8x8_inference_flag: false,
            frame_crop_info: None,
            sample_aspect_ratio: None,
            overscan_appropriate_flag: None,
            color_config: None,
            chroma_sample_loc: None,
            timing_info: None,
        }
        ");

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example sps
        result.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        // sometimes bits can get lost because we save
        // some space with how the SPS is rebuilt.
        // so we can just confirm that they're the same
        // by rebuilding it.
        let reduced = Sps::parse(&buf).unwrap();
        assert_eq!(reduced, result);

        // now we can check that the bitstream from
        // the reduced version should be the same
        let mut reduced_buf = Vec::new();
        let mut writer3 = BitWriter::new(&mut reduced_buf);

        reduced.build(&mut writer3).unwrap();
        writer3.finish().unwrap();
        assert_eq!(reduced_buf, buf);

        // now we can check the size:
        assert_eq!(reduced.size(), result.size());
    }

    #[test]
    fn test_build_size_sps_ext_chroma_not_3_and_no_scaling_matrix_and_size() {
        // create data bitstream for sps_ext
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_exp_golomb(1).unwrap();
        writer.write_exp_golomb(2).unwrap();
        writer.write_exp_golomb(4).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let sps_ext = SpsExtended::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        sps_ext.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_sps_ext = SpsExtended::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_sps_ext.bitsize(), sps_ext.bitsize());
        assert_eq!(rebuilt_sps_ext.bytesize(), sps_ext.bytesize());
    }

    #[test]
    fn test_build_size_sps_ext_chroma_3_and_scaling_matrix() {
        // create bitstream for sps_ext
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        // set chroma_format_idc = 3
        writer.write_exp_golomb(3).unwrap();
        // separate_color_plane_flag since chroma_format_idc = 3
        writer.write_bit(true).unwrap();
        writer.write_exp_golomb(2).unwrap();
        writer.write_exp_golomb(4).unwrap();
        writer.write_bit(true).unwrap();
        // set seq_scaling_matrix_present_flag
        writer.write_bit(true).unwrap();

        // scaling matrix loop happens 12 times since chroma_format_idc is 3
        // loop 1 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64
        // next_scale starts as 8, we add 1 so it's 9
        writer.write_signed_exp_golomb(1).unwrap();
        // subloop 2 of 64
        // next_scale is 9, we add 2 so it's 11
        writer.write_signed_exp_golomb(2).unwrap();
        // subloop 3 of 64
        // next_scale is 11, we add 3 so it's 14
        writer.write_signed_exp_golomb(3).unwrap();
        // subloop 4 of 64: we want to break out of the loop now
        // next_scale is 14, we subtract 14 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-14).unwrap();

        // loop 2 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64
        // next_scale starts at 8, we add 3 so it's 11
        writer.write_signed_exp_golomb(3).unwrap();
        // subloop 2 of 64
        // next_scale is 11, we add 5 so it's 16
        writer.write_signed_exp_golomb(5).unwrap();
        // subloop 3 of 64; we want to break out of the loop now
        // next_scale is 16, we subtract 16 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-16).unwrap();

        // loop 3 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64
        // next_scale starts at 8, we add 1 so it's 9
        writer.write_signed_exp_golomb(1).unwrap();
        // subloop 2 of 64; we want to break out of the loop now
        // next_scale is 9, we subtract 9 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-9).unwrap();

        // loop 4 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64; we want to break out of the loop now
        // next scale starts at 8, we subtract 8 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-8).unwrap();

        // loop 5 thru 11: try writing nothing
        writer.write_bits(0, 7).unwrap();

        // loop 12 of 12: try writing something
        writer.write_bit(true).unwrap();
        // subloop 1 of 64; we want to break out of the loop now
        // next scale starts at 8, we subtract 8 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-8).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let sps_ext = SpsExtended::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        sps_ext.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_sps_ext = SpsExtended::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_sps_ext.bitsize(), sps_ext.bitsize());
        assert_eq!(rebuilt_sps_ext.bytesize(), sps_ext.bytesize());
    }

    #[test]
    fn test_build_size_pic_order() {
        // create bitstream for pic_order_count_type1
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bit(true).unwrap();
        writer.write_signed_exp_golomb(3).unwrap();
        writer.write_signed_exp_golomb(7).unwrap();
        writer.write_exp_golomb(2).unwrap();

        // loop
        writer.write_signed_exp_golomb(4).unwrap();
        writer.write_signed_exp_golomb(8).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let pic_order_count_type1 = PicOrderCountType1::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        pic_order_count_type1.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_pic_order_count_type1 = PicOrderCountType1::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_pic_order_count_type1.bitsize(), pic_order_count_type1.bitsize());
        assert_eq!(rebuilt_pic_order_count_type1.bytesize(), pic_order_count_type1.bytesize());
    }

    #[test]
    fn test_build_size_frame_crop() {
        // create bitstream for frame_crop
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_exp_golomb(1).unwrap();
        writer.write_exp_golomb(10).unwrap();
        writer.write_exp_golomb(7).unwrap();
        writer.write_exp_golomb(38).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let frame_crop_info = FrameCropInfo::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        frame_crop_info.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_frame_crop_info = FrameCropInfo::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_frame_crop_info.bitsize(), frame_crop_info.bitsize());
        assert_eq!(rebuilt_frame_crop_info.bytesize(), frame_crop_info.bytesize());
    }

    #[test]
    fn test_build_size_sar_idc_not_255() {
        // create bitstream for sample_aspect_ratio
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bits(1, 8).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let sample_aspect_ratio = SarDimensions::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        sample_aspect_ratio.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_sample_aspect_ratio = SarDimensions::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_sample_aspect_ratio.bitsize(), sample_aspect_ratio.bitsize());
        assert_eq!(rebuilt_sample_aspect_ratio.bytesize(), sample_aspect_ratio.bytesize());
    }

    #[test]
    fn test_build_size_sar_idc_255() {
        // create bitstream for sample_aspect_ratio
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bits(255, 8).unwrap();
        writer.write_bits(11, 16).unwrap();
        writer.write_bits(32, 16).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let sample_aspect_ratio = SarDimensions::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        sample_aspect_ratio.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_sample_aspect_ratio = SarDimensions::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_sample_aspect_ratio.bitsize(), sample_aspect_ratio.bitsize());
        assert_eq!(rebuilt_sample_aspect_ratio.bytesize(), sample_aspect_ratio.bytesize());
    }

    #[test]
    fn test_build_size_color_config() {
        // create bitstream for color_config
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bits(4, 3).unwrap();
        writer.write_bit(true).unwrap();

        // color_desc_present_flag
        writer.write_bit(true).unwrap();
        writer.write_bits(2, 8).unwrap();
        writer.write_bits(6, 8).unwrap();
        writer.write_bits(1, 8).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let color_config = ColorConfig::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        color_config.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);
        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_color_config = ColorConfig::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_color_config.bitsize(), color_config.bitsize());
        assert_eq!(rebuilt_color_config.bytesize(), color_config.bytesize());
    }

    #[test]
    fn test_build_size_color_config_no_desc() {
        // create bitstream for color_config
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bits(4, 3).unwrap();
        writer.write_bit(true).unwrap();

        // color_desc_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let color_config = ColorConfig::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        color_config.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_color_config = ColorConfig::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_color_config.bitsize(), color_config.bitsize());
        assert_eq!(rebuilt_color_config.bytesize(), color_config.bytesize());
    }

    #[test]
    fn test_build_size_chroma_sample() {
        // create bitstream for chroma_sample_loc
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_exp_golomb(111).unwrap();
        writer.write_exp_golomb(222).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let chroma_sample_loc = ChromaSampleLoc::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        chroma_sample_loc.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_chroma_sample_loc = ChromaSampleLoc::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_chroma_sample_loc.bitsize(), chroma_sample_loc.bitsize());
        assert_eq!(rebuilt_chroma_sample_loc.bytesize(), chroma_sample_loc.bytesize());
    }

    #[test]
    fn test_build_size_timing_info() {
        // create bitstream for timing_info
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bits(1234, 32).unwrap();
        writer.write_bits(321, 32).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let timing_info = TimingInfo::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        timing_info.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_timing_info = TimingInfo::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_timing_info.bitsize(), timing_info.bitsize());
        assert_eq!(rebuilt_timing_info.bytesize(), timing_info.bytesize());
    }

    #[test]
    fn test_size_sps() {
        let mut bit_count = 0;
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        bit_count += 2;
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();
        bit_count += 5;

        // profile_idc = 44
        writer.write_bits(44, 8).unwrap();
        bit_count += 8;
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        bit_count += 8;
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();
        bit_count += 8;
        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();
        bit_count += size_of_exp_golomb(0);

        // sps ext
        // we want to try out chroma_format_idc = 3
        // chroma_format_idc is expg
        writer.write_exp_golomb(3).unwrap();
        bit_count += size_of_exp_golomb(3);
        // separate_color_plane_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        bit_count += size_of_exp_golomb(0);
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        bit_count += size_of_exp_golomb(0);
        // qpprime
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // we want to simulate a scaling matrix
        // seq_scaling_matrix_present_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;

        // enter scaling matrix, we loop 12 times since
        // chroma_format_idc = 3.
        // loop 1 of 12
        // true to enter if statement
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // i < 6, so size is 16, so we loop 16 times
        // sub-loop 1 of 16
        // delta_scale is a SIGNED expg so we can try out
        // entering -4 so next_scale becomes 8 + 4 = 12
        writer.write_signed_exp_golomb(4).unwrap();
        bit_count += size_of_signed_exp_golomb(4);
        // sub-loop 2 of 16
        // delta_scale is a SIGNED expg so we can try out
        // entering -12 so next scale becomes 12 - 12 = 0
        writer.write_signed_exp_golomb(-12).unwrap();
        bit_count += size_of_signed_exp_golomb(-12);
        // at this point next_scale is 0, which means we break
        // loop 2 through 12
        // we don't need to try anything else so we can just skip through them by writing `0` bit 11 times.
        writer.write_bits(0, 11).unwrap();
        bit_count += 11;

        // back to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        bit_count += size_of_exp_golomb(0);
        // we can try setting pic_order_cnt_type to 1
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(1).unwrap();
        bit_count += size_of_exp_golomb(1);

        // delta_pic_order_always_zero_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // offset_for_non_ref_pic
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // offset_for_top_to_bottom_field
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // num_ref_frames_in_pic_order_cnt_cycle is expg
        writer.write_exp_golomb(1).unwrap();
        bit_count += size_of_exp_golomb(1);
        // loop num_ref_frames_in_pic_order_cnt_cycle times (1)
        // offset_for_ref_frame is expg
        writer.write_exp_golomb(0).unwrap();
        bit_count += size_of_exp_golomb(0);

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        bit_count += size_of_exp_golomb(0);
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // 1920 width:
        // 1920 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 4 later
        // 1920 = (p + 1) * 16 - 2 * 4 - 2 * 4
        // 1920 = (p + 1) * 16 - 16
        // p = 120
        // pic_width_in_mbs_minus1 is expg
        writer.write_exp_golomb(120).unwrap();
        bit_count += size_of_exp_golomb(120);
        // we want 1080 height:
        // 1080 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 2 later
        // m is frame_mbs_only_flag which we set to 0 later
        // 1080 = (2 - 0) * (p + 1) * 16 - 2 * 2 - 2 * 2
        // 1080 = 2 * (p + 1) * 16 - 8
        // p = 33
        // pic_height_in_map_units_minus1 is expg
        writer.write_exp_golomb(33).unwrap();
        bit_count += size_of_exp_golomb(33);

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // frame_cropping_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;

        // frame_crop_left_offset is expg
        writer.write_exp_golomb(4).unwrap();
        bit_count += size_of_exp_golomb(4);
        // frame_crop_left_offset is expg
        writer.write_exp_golomb(4).unwrap();
        bit_count += size_of_exp_golomb(4);
        // frame_crop_left_offset is expg
        writer.write_exp_golomb(2).unwrap();
        bit_count += size_of_exp_golomb(2);
        // frame_crop_left_offset is expg
        writer.write_exp_golomb(2).unwrap();
        bit_count += size_of_exp_golomb(2);

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;

        // enter vui to set the framerate
        // aspect_ratio_info_present_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // we can try 255 to set the sar_width and sar_height
        // aspect_ratio_idc
        writer.write_bits(255, 8).unwrap();
        bit_count += 8;
        // sar_width
        writer.write_bits(0, 16).unwrap();
        bit_count += 16;
        // sar_height
        writer.write_bits(0, 16).unwrap();
        bit_count += 16;

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;

        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // video_format
        writer.write_bits(0, 3).unwrap();
        bit_count += 3;
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;
        // color_description_present_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // color_primaries
        writer.write_bits(1, 8).unwrap();
        bit_count += 8;
        // transfer_characteristics
        writer.write_bits(1, 8).unwrap();
        bit_count += 8;
        // matrix_coefficients
        writer.write_bits(1, 8).unwrap();
        bit_count += 8;

        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();
        bit_count += 1;

        // timing_info_present_flag
        writer.write_bit(true).unwrap();
        bit_count += 1;
        // we can set this to 1000 for example
        // num_units_in_tick is a u32
        writer.write_bits(1000, 32).unwrap();
        bit_count += 32;
        // fps = time_scale / (2 * num_units_in_tick)
        // since we want 480 fps:
        // 480 = time_scale / (2 * 1000)
        // 960 000 = time_scale
        // time_scale is a u32
        writer.write_bits(960000, 32).unwrap();
        bit_count += 32;
        writer.finish().unwrap();

        let result = Sps::parse(&sps).unwrap();

        // now we can check the size:
        assert_eq!(result.size(), bit_count.div_ceil(8));
    }

    #[test]
    fn test_reduce_color_config() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 100
        writer.write_bits(100, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // sps ext
        // chroma_format_idc is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // seq_scaling_matrix_present_flag
        writer.write_bit(false).unwrap();

        // back to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4 is expg
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // width
        writer.write_exp_golomb(0).unwrap();
        // height
        writer.write_exp_golomb(0).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(true).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // aspect_ratio_info_present_flag
        writer.write_bit(false).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // we want to change the color_config
        // video_signal_type_present_flag
        writer.write_bit(true).unwrap();

        // video_format
        writer.write_bits(1, 3).unwrap();
        // video_full_range_flag
        writer.write_bit(false).unwrap();
        // color_description_present_flag: we want this to be true
        writer.write_bit(true).unwrap();

        // now we set these to redundant values (each should be 2)
        writer.write_bits(2, 8).unwrap();
        writer.write_bits(2, 8).unwrap();
        writer.write_bits(2, 8).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();

        // timing_info_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        let reduced_sps = Sps::reduce(&sps).unwrap();

        assert_ne!(sps, reduced_sps);
    }

    #[test]
    fn test_reduce_vui() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        // forbidden zero bit must be unset
        writer.write_bit(false).unwrap();
        // nal_ref_idc is 0
        writer.write_bits(0, 2).unwrap();
        // nal_unit_type must be 7
        writer.write_bits(7, 5).unwrap();

        // profile_idc = 100
        writer.write_bits(100, 8).unwrap();
        // constraint_setn_flags all false
        writer.write_bits(0, 8).unwrap();
        // level_idc = 0
        writer.write_bits(0, 8).unwrap();

        // seq_parameter_set_id is expg
        writer.write_exp_golomb(0).unwrap();

        // sps ext
        // chroma_format_idc is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_luma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // bit_depth_chroma_minus8 is expg
        writer.write_exp_golomb(0).unwrap();
        // qpprime
        writer.write_bit(false).unwrap();
        // seq_scaling_matrix_present_flag
        writer.write_bit(false).unwrap();

        // back to sps
        // log2_max_frame_num_minus4 is expg
        writer.write_exp_golomb(0).unwrap();
        // pic_order_cnt_type is expg
        writer.write_exp_golomb(0).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4 is expg
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        // width
        writer.write_exp_golomb(0).unwrap();
        // height
        writer.write_exp_golomb(0).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(true).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // we want to set this flag to be true and all subsequent flags to be false.
        // vui_parameters_present_flag
        writer.write_bit(true).unwrap();

        // aspect_ratio_info_present_flag
        writer.write_bit(false).unwrap();

        // overscan_info_present_flag
        writer.write_bit(false).unwrap();

        // video_signal_type_present_flag
        writer.write_bit(false).unwrap();

        // chroma_loc_info_present_flag
        writer.write_bit(false).unwrap();

        // timing_info_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        let reduced_sps = Sps::reduce(&sps).unwrap();

        assert_ne!(sps, reduced_sps);
    }
}
