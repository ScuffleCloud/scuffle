mod chroma_sample_loc;
use self::chroma_sample_loc::ChromaSampleLoc;

mod color_config;
use self::color_config::ColorConfig;

mod frame_crop_info;
use self::frame_crop_info::FrameCropInfo;

mod pic_order_count_type1;
use self::pic_order_count_type1::PicOrderCountType1;

mod sample_aspect_ratio;
use self::sample_aspect_ratio::SarDimensions;

mod sps_ext;
pub use self::sps_ext::SpsExtended;

mod timing_info;
use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::{BitReader, BitWriter};
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt, size_of_exp_golomb};

pub use self::timing_info::TimingInfo;
use crate::{EmulationPreventionIo, NALUnitType};

/// The Sequence Parameter Set.
/// ISO/IEC-14496-10-2022 - 7.3.2
#[derive(Debug, Clone, PartialEq)]
pub struct Sps {
    /// Test temp field for semver-checks
    pub test_change_for_semver_check: bool,
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
    pub fn parse(reader: impl io::Read) -> io::Result<Self> {
        let mut bit_reader = BitReader::new(reader);

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
                bit_reader.read_bits(3)?;
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
            bit_reader.read_bit()?;
            false
        } else {
            bit_reader.read_bit()?
        };

        let constraint_set4_flag = match profile_idc {
            // 7.4.2.1.1
            77 | 88 | 100 | 118 | 128 | 134 => bit_reader.read_bit()?,
            _ => {
                bit_reader.read_bit()?;
                false
            }
        };

        let constraint_set5_flag = match profile_idc {
            77 | 88 | 100 | 118 => bit_reader.read_bit()?,
            _ => {
                bit_reader.read_bit()?;
                false
            }
        };
        // reserved_zero_2bits
        bit_reader.read_bits(2)?;

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
            test_change_for_semver_check: true,
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
    pub fn build(&self, writer: impl io::Write) -> io::Result<()> {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_bit(false)?;
        bit_writer.write_bits(self.nal_ref_idc as u64, 2)?;
        bit_writer.write_bits(self.nal_unit_type.0 as u64, 5)?;
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

        bit_writer.write_bit(self.mb_adaptive_frame_field_flag.is_none())?;
        if let Some(flag) = self.mb_adaptive_frame_field_flag {
            bit_writer.write_bit(flag)?;
        }

        bit_writer.write_bit(self.direct_8x8_inference_flag)?;

        bit_writer.write_bit(self.frame_crop_info.is_some())?;
        if let Some(frame_crop_info) = &self.frame_crop_info {
            frame_crop_info.build(&mut bit_writer)?;
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
                bit_writer.write_bit(self.sample_aspect_ratio.is_some())?;
                if let Some(sar) = &self.sample_aspect_ratio {
                    sar.build(&mut bit_writer)?;
                }

                // overscan_info_present_flag
                bit_writer.write_bit(self.overscan_appropriate_flag.is_some())?;
                if let Some(overscan) = &self.overscan_appropriate_flag {
                    bit_writer.write_bit(*overscan)?;
                }

                // video_signal_type_prsent_flag
                bit_writer.write_bit(self.color_config.is_some())?;
                if let Some(color) = &self.color_config {
                    color.build(&mut bit_writer)?;
                }

                // chroma_log_info_present_flag
                bit_writer.write_bit(self.chroma_sample_loc.is_some())?;
                if let Some(chroma) = &self.chroma_sample_loc {
                    chroma.build(&mut bit_writer)?;
                }

                // timing_info_present_flag
                bit_writer.write_bit(self.timing_info.is_some())?;
                if let Some(timing) = &self.timing_info {
                    timing.build(&mut bit_writer)?;
                }
            }
        }
        bit_writer.finish()?;

        Ok(())
    }

    /// Parses the Sps struct from a reader that may contain emulation prevention bytes.
    /// Is the same as calling [`Self::parse`] with an [`EmulationPreventionIo`] wrapper.
    pub fn parse_with_emulation_prevention(reader: impl io::Read) -> io::Result<Self> {
        Self::parse(EmulationPreventionIo::new(reader))
    }

    /// Builds the Sps struct into a byte stream that may contain emulation prevention bytes.
    /// Is the same as calling [`Self::build`] with an [`EmulationPreventionIo`] wrapper.
    pub fn build_with_emulation_prevention(self, writer: impl io::Write) -> io::Result<()> {
        self.build(EmulationPreventionIo::new(writer))
    }

    /// Returns the total byte size of the Sps struct.
    pub fn size(&self) -> u64 {
        (1 + // forbidden zero bit
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
    pub fn frame_rate(&self) -> Option<f64> {
        self.timing_info.as_ref().map(|timing| timing.frame_rate())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use scuffle_bytes_util::BitWriter;
    use scuffle_expgolomb::{BitWriterExpGolombExt, size_of_exp_golomb, size_of_signed_exp_golomb};

    use crate::sps::Sps;

    #[test]
    fn test_parse_sps_set_forbidden_bit() {
        let mut sps = Vec::new();
        let mut writer = BitWriter::new(&mut sps);

        writer.write_bit(true).unwrap(); // sets the forbidden bit
        writer.finish().unwrap();

        let result = Sps::parse(std::io::Cursor::new(sps));

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
        writer.finish().unwrap();

        let result = Sps::parse(std::io::Cursor::new(sps));

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

        let result = Sps::parse(std::io::Cursor::new(sps)).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            test_change_for_semver_check: true,
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

        assert_eq!(Some(144.0), result.frame_rate());
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
        let reduced = Sps::parse(std::io::Cursor::new(&buf)).unwrap(); // <- this is where things break
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

        let result = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            test_change_for_semver_check: true,
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

        assert_eq!(Some(480.0), result.frame_rate());
        assert_eq!(1920, result.width());
        assert_eq!(1080, result.height());

        // create a writer for the builder
        let mut buf = Vec::new();
        result.build(&mut buf).unwrap();

        assert_eq!(buf, sps);
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

        let result = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            test_change_for_semver_check: true,
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

        assert_eq!(None, result.frame_rate());
        assert_eq!(1280, result.width());
        assert_eq!(800, result.height());

        // create a writer for the builder
        let mut buf = Vec::new();
        result.build(&mut buf).unwrap();

        assert_eq!(buf, sps);
    }

    #[test]
    fn test_parse_build_sps_pic_order_cnt_type_2() {
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
        writer.write_exp_golomb(2).unwrap();
        // log2_max_pic_order_cnt_lsb_minus4
        writer.write_exp_golomb(0).unwrap();

        // max_num_ref_frames is expg
        writer.write_exp_golomb(0).unwrap();
        // gaps_in_frame_num_value_allowed_flag
        writer.write_bit(false).unwrap();
        writer.write_exp_golomb(1).unwrap();
        writer.write_exp_golomb(2).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(true).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(false).unwrap();

        // enter vui to set redundant parameters so they get reduced
        // vui_parameters_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            test_change_for_semver_check: true,
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
            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: true,
            pic_width_in_mbs_minus1: 3,
            pic_height_in_map_units_minus1: 0,
            mb_adaptive_frame_field_flag: None,
            direct_8x8_inference_flag: true,
            frame_crop_info: None,
            sample_aspect_ratio: None,
            overscan_appropriate_flag: None,
            color_config: None,
            chroma_sample_loc: None,
            timing_info: None,
        }
        ");

        assert_eq!(None, result.frame_rate());
        assert_eq!(result.size(), 7);

        // create a writer for the builder
        let mut buf = Vec::new();
        result.build_with_emulation_prevention(&mut buf).unwrap();

        assert_eq!(buf, sps);
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

        let result = Sps::parse(std::io::Cursor::new(&sps));

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

        let result = Sps::parse(std::io::Cursor::new(&sps));

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

        let result = Sps::parse(std::io::Cursor::new(&sps));

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
        writer.write_exp_golomb(0).unwrap();

        // skip sps ext since profile_idc = 77
        // log2_max_frame_num_minus4 is expg so 0b1 (true) = false
        writer.write_exp_golomb(0).unwrap();
        // we can try setting pic_order_cnt_type to 1
        writer.write_exp_golomb(1).unwrap();

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
        writer.write_exp_golomb(999).unwrap();
        // we want 1080 height:
        // 1080 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
        // we set offset1 and offset2 to both be 2 later
        // m is frame_mbs_only_flag which we set to 0 later
        // 1080 = (2 - 0) * (p + 1) * 16 - 2 * 2 - 2 * 2
        // 1080 = 2 * (p + 1) * 16 - 8
        // p = 33
        // pic_height_in_map_units_minus1 is expg so:
        // 000 0010 0010
        writer.write_exp_golomb(899).unwrap();

        // frame_mbs_only_flag
        writer.write_bit(false).unwrap();
        // mb_adaptive_frame_field_flag
        writer.write_bit(false).unwrap();

        // direct_8x8_inference_flag
        writer.write_bit(false).unwrap();
        // frame_cropping_flag
        writer.write_bit(true).unwrap();

        // frame_crop_left_offset is expg
        writer.write_exp_golomb(100).unwrap();
        // frame_crop_right_offset is expg
        writer.write_exp_golomb(200).unwrap();
        // frame_crop_top_offset is expg
        writer.write_exp_golomb(300).unwrap();
        // frame_crop_bottom_offset is expg
        writer.write_exp_golomb(400).unwrap();

        // vui_parameters_present_flag
        writer.write_bit(false).unwrap();
        writer.finish().unwrap();

        let result = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

        insta::assert_debug_snapshot!(result, @r"
        Sps {
            test_change_for_semver_check: true,
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
            pic_width_in_mbs_minus1: 999,
            pic_height_in_map_units_minus1: 899,
            mb_adaptive_frame_field_flag: Some(
                false,
            ),
            direct_8x8_inference_flag: false,
            frame_crop_info: Some(
                FrameCropInfo {
                    frame_crop_left_offset: 100,
                    frame_crop_right_offset: 200,
                    frame_crop_top_offset: 300,
                    frame_crop_bottom_offset: 400,
                },
            ),
            sample_aspect_ratio: None,
            overscan_appropriate_flag: None,
            color_config: None,
            chroma_sample_loc: None,
            timing_info: None,
        }
        ");

        // create a writer for the builder
        let mut buf = Vec::new();
        // build from the example sps
        result.build(&mut buf).unwrap();

        assert_eq!(buf, sps);
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

        let result = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

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

        let reduced_sps = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

        let mut reduced_buf = Vec::new();
        reduced_sps.build(&mut reduced_buf).unwrap();

        assert_ne!(sps, reduced_buf);
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

        let result = Sps::parse(std::io::Cursor::new(&sps)).unwrap();

        let mut reduced_buf = Vec::new();
        result.build(&mut reduced_buf).unwrap();

        let reduced_result = Sps::parse(std::io::Cursor::new(&reduced_buf)).unwrap();
        assert_eq!(result.size(), reduced_result.size());

        insta::assert_debug_snapshot!(reduced_result, @r"
        Sps {
            test_change_for_semver_check: true,
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
            pic_width_in_mbs_minus1: 0,
            pic_height_in_map_units_minus1: 0,
            mb_adaptive_frame_field_flag: None,
            direct_8x8_inference_flag: false,
            frame_crop_info: None,
            sample_aspect_ratio: None,
            overscan_appropriate_flag: None,
            color_config: None,
            chroma_sample_loc: None,
            timing_info: None,
        }
        ");
    }
}
