use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// HRD parameters.
///
/// `hrd_parameters(commonInfPresentFlag, maxNumSubLayersMinus1)`
///
/// - ISO/IEC 23008-2 - E.2.2
/// - ISO/IEC 23008-2 - E.3.2
#[derive(Debug, Clone, PartialEq)]
pub struct HrdParameters {
    /// HRD parameters information unrelated to sub-layers.
    pub common_inf: CommonInf,
    /// Sub-layer HRD parameters.
    pub sub_layers: Vec<HrdParametersSubLayer>,
}

impl HrdParameters {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        common_inf_present_flag: bool,
        max_num_sub_layers_minus1: u8,
    ) -> io::Result<Self> {
        let mut common_inf = CommonInf::default();

        let mut nal_hrd_parameters_present_flag = false;
        let mut vcl_hrd_parameters_present_flag = false;

        if common_inf_present_flag {
            nal_hrd_parameters_present_flag = bit_reader.read_bit()?;
            vcl_hrd_parameters_present_flag = bit_reader.read_bit()?;

            if nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag {
                let sub_pic_hrd_params_present_flag = bit_reader.read_bit()?;
                if sub_pic_hrd_params_present_flag {
                    let tick_divisor_minus2 = bit_reader.read_u8()?;
                    let du_cpb_removal_delay_increment_length_minus1 = bit_reader.read_bits(5)? as u8;
                    let sub_pic_cpb_params_in_pic_timing_sei_flag = bit_reader.read_bit()?;
                    let dpb_output_delay_du_length_minus1 = bit_reader.read_bits(5)? as u8;

                    common_inf.sub_pic_hrd_params = Some(SubPicHrdParams {
                        tick_divisor_minus2,
                        du_cpb_removal_delay_increment_length_minus1,
                        sub_pic_cpb_params_in_pic_timing_sei_flag,
                        dpb_output_delay_du_length_minus1,
                        cpb_size_du_scale: 0, // replaced below
                    });
                }

                common_inf.bit_rate_scale = Some(bit_reader.read_bits(4)? as u8);
                common_inf.cpb_size_scale = Some(bit_reader.read_bits(4)? as u8);

                if sub_pic_hrd_params_present_flag {
                    let cpb_size_du_scale = bit_reader.read_bits(4)? as u8;

                    // set the cpb_size_du_scale in sub_pic_hrd_params
                    if let Some(ref mut sub_pic_hrd_params) = common_inf.sub_pic_hrd_params {
                        sub_pic_hrd_params.cpb_size_du_scale = cpb_size_du_scale;
                    }
                }

                common_inf.initial_cpb_removal_delay_length_minus1 = bit_reader.read_bits(5)? as u8;
                common_inf.au_cpb_removal_delay_length_minus1 = bit_reader.read_bits(5)? as u8;
                common_inf.dpb_output_delay_length_minus1 = bit_reader.read_bits(5)? as u8;
            }
        }

        let mut sub_layers = Vec::with_capacity(max_num_sub_layers_minus1 as usize + 1);

        for _ in 0..=max_num_sub_layers_minus1 {
            sub_layers.push(HrdParametersSubLayer::parse(
                bit_reader,
                common_inf.sub_pic_hrd_params.is_some(),
                nal_hrd_parameters_present_flag,
                vcl_hrd_parameters_present_flag,
            )?);
        }

        Ok(HrdParameters { common_inf, sub_layers })
    }
}

/// Directly part of [`HrdParameters`].
#[derive(Debug, Clone, PartialEq)]
pub struct CommonInf {
    /// Sub-picture HRD parameters, if `sub_pic_hrd_params_present_flag` is `true`.
    pub sub_pic_hrd_params: Option<SubPicHrdParams>,
    /// Specifies (together with [`bit_rate_value_minus1[i]`](SubLayerHrdParameters::bit_rate_value_minus1)) the maximum
    /// input bit rate of the i-th CPB.
    pub bit_rate_scale: Option<u8>,
    /// Specifies ((together with [`cpb_size_value_minus1[i]`](SubLayerHrdParameters::cpb_size_value_minus1))) the CPB size
    /// of the i-th CPB when the CPB operates at the access unit level.
    pub cpb_size_scale: Option<u8>,
    /// This value plus 1 specifies the length, in bits, of the
    /// `nal_initial_cpb_removal_delay[i]`, `nal_initial_cpb_removal_offset[i]`, `vcl_initial_cpb_removal_delay[i]`,
    /// and `vcl_initial_cpb_removal_offset[i]` syntax elements of the buffering period SEI message.
    pub initial_cpb_removal_delay_length_minus1: u8,
    /// This value plus 1 specifies the length, in bits, of the cpb_delay_offset syntax
    /// element in the buffering period SEI message and the au_cpb_removal_delay_minus1 syntax element in
    /// the picture timing SEI message.
    pub au_cpb_removal_delay_length_minus1: u8,
    /// This value plus 1 specifies the length, in bits, of the dpb_delay_offset syntax
    /// element in the buffering period SEI message and the pic_dpb_output_delay syntax element in the picture
    /// timing SEI message.
    pub dpb_output_delay_length_minus1: u8,
}

impl Default for CommonInf {
    fn default() -> Self {
        Self {
            sub_pic_hrd_params: None,
            bit_rate_scale: None,
            cpb_size_scale: None,
            initial_cpb_removal_delay_length_minus1: 23,
            au_cpb_removal_delay_length_minus1: 23,
            dpb_output_delay_length_minus1: 23,
        }
    }
}

/// Directly part of [`HrdParameters`].
#[derive(Debug, Clone, PartialEq)]
pub struct SubPicHrdParams {
    /// Used to specify the clock sub-tick. A clock sub-tick is the minimum interval of
    /// time that can be represented in the coded data.
    pub tick_divisor_minus2: u8,
    /// This value plus 1 specifies the length, in bits, of the
    /// `du_cpb_removal_delay_increment_minus1[i]` and `du_common_cpb_removal_delay_increment_minus1`
    /// syntax elements of the picture timing SEI message and the `du_spt_cpb_removal_delay_increment` syntax
    /// element in the decoding unit information SEI message.
    pub du_cpb_removal_delay_increment_length_minus1: u8,
    /// Equal to `true` specifies that sub-picture level CPB removal
    /// delay parameters are present in picture timing SEI messages and no decoding unit information SEI
    /// message is available (in the CVS or provided through external means not specified in this document).
    ///
    /// Equal to `false` specifies that sub-picture level CPB removal delay
    /// parameters are present in decoding unit information SEI messages and picture timing SEI messages do
    /// not include sub-picture level CPB removal delay parameters.
    pub sub_pic_cpb_params_in_pic_timing_sei_flag: bool,
    /// This value plus 1 specifies the length, in bits, of
    /// `pic_dpb_output_du_delay` syntax element in the picture timing SEI message and
    /// `pic_spt_dpb_output_du_delay` syntax element in the decoding unit information SEI message.
    pub dpb_output_delay_du_length_minus1: u8,
    /// Specifies (together with [`cpb_size_du_value_minus1[i]`](SubLayerHrdParameters::cpb_size_du_value_minus1))
    /// the CPB size of the i-th CPB when the CPB operates at sub-picture level.
    pub cpb_size_du_scale: u8,
}

/// Directly part of [`HrdParameters`].
#[derive(Debug, Clone, PartialEq)]
pub struct HrdParametersSubLayer {
    /// Equal to `true` indicates that, when `HighestTid` is equal to `i`, the temporal
    /// distance between the HRD output times of consecutive pictures in output order is constrained as specified.
    ///
    /// Equal to `false` indicates that this constraint may not apply.
    pub fixed_pic_rate_general_flag: bool,
    /// Equal to `true` indicates that, when `HighestTid` is equal to `i`, the temporal
    /// distance between the HRD output times of consecutive pictures in output order is constrained as specified.
    ///
    /// Equal to `false` indicates that this constraint may not apply.
    pub fixed_pic_rate_within_cvs_flag: bool,
    /// This value plus 1 (when present) specifies, when `HighestTid` is equal to `i`,
    /// the temporal distance, in clock ticks, between the elemental units that specify the HRD output times of
    /// consecutive pictures in output order as specified.
    ///
    /// The value is in range \[0, 2047\].
    pub elemental_duration_in_tc_minus1: Option<u64>,
    /// Specifies the HRD operational mode, when `HighestTid` is equal to `i`, as specified in
    /// ISO/IEC 23008-2 Annex C or ISO/IEC 23008-2 F.13.
    pub low_delay_hrd_flag: bool,
    /// This value plus 1 specifies the number of alternative CPB specifications in the bitstream of the
    /// CVS when `HighestTid` is equal to `i`.
    ///
    /// The value is in range \[0, 31\].
    pub cpb_cnt_minus1: u64,
    /// Sub-layer HRD parameters.
    pub sub_layer_parameters: Vec<SubLayerHrdParameters>,
}

impl HrdParametersSubLayer {
    fn parse(
        bit_reader: &mut BitReader<impl io::Read>,
        sub_pic_hrd_params_present_flag: bool,
        nal_hrd_parameters_present_flag: bool,
        vcl_hrd_parameters_present_flag: bool,
    ) -> io::Result<Self> {
        let mut fixed_pic_rate_within_cvs_flag = true;

        let fixed_pic_rate_general_flag = bit_reader.read_bit()?;
        if !fixed_pic_rate_general_flag {
            fixed_pic_rate_within_cvs_flag = bit_reader.read_bit()?;
        }

        let mut elemental_duration_in_tc_minus1_value = None;
        let mut low_delay_hrd_flag = false;
        if fixed_pic_rate_within_cvs_flag {
            let elemental_duration_in_tc_minus1 = bit_reader.read_exp_golomb()?;
            range_check!(elemental_duration_in_tc_minus1, 0, 2047)?;
            elemental_duration_in_tc_minus1_value = Some(elemental_duration_in_tc_minus1);
        } else {
            low_delay_hrd_flag = bit_reader.read_bit()?;
        }

        let mut cpb_cnt_minus1 = 0;
        if !low_delay_hrd_flag {
            cpb_cnt_minus1 = bit_reader.read_exp_golomb()?;
            range_check!(cpb_cnt_minus1, 0, 31)?;
        }

        let mut sub_layer_parameters = Vec::new();

        if nal_hrd_parameters_present_flag {
            sub_layer_parameters.append(&mut SubLayerHrdParameters::parse(
                bit_reader,
                true,
                cpb_cnt_minus1 + 1,
                sub_pic_hrd_params_present_flag,
            )?);
        }

        if vcl_hrd_parameters_present_flag {
            sub_layer_parameters.append(&mut SubLayerHrdParameters::parse(
                bit_reader,
                false,
                cpb_cnt_minus1 + 1,
                sub_pic_hrd_params_present_flag,
            )?);
        }

        Ok(Self {
            fixed_pic_rate_general_flag,
            fixed_pic_rate_within_cvs_flag,
            elemental_duration_in_tc_minus1: elemental_duration_in_tc_minus1_value,
            low_delay_hrd_flag,
            cpb_cnt_minus1,
            sub_layer_parameters,
        })
    }
}

/// Sub-layer HRD parameters.
///
/// `sub_layer_hrd_parameters(subLayerId)`
///
/// - ISO/IEC 23008-2 - E.2.3
/// - ISO/IEC 23008-2 - E.3.3
#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerHrdParameters {
    /// Internal field to store if this is a NAL or VCL HRD
    nal_hrd: bool,
    /// Specifies (together with [`bit_rate_scale`](CommonInf::bit_rate_scale)) the maximum input bit rate
    /// for the i-th CPB when the CPB operates at the access unit level.
    ///
    /// For any `i > 0`, `bit_rate_value_minus1[i]` is greater than `bit_rate_value_minus1[i − 1]`.
    ///
    /// The value is in range \[0, 2^32 - 2\].
    ///
    /// Defines [`BitRate[i]`](SubLayerHrdParameters::bit_rate).
    pub bit_rate_value_minus1: u32,
    /// Used together with [`cpb_size_scale`](CommonInf::cpb_size_scale) to specify
    /// the i-th CPB size when the CPB operates at the access unit level.
    ///
    /// For any `i > 0`, `cpb_size_value_minus1[i]` is less than or equal to `cpb_size_value_minus1[i − 1]`.
    ///
    /// The value is in range \[0, 2^32 - 2\].
    ///
    /// Defines [`CpbSize[i]`](SubLayerHrdParameters::cpb_size).
    pub cpb_size_value_minus1: u32,
    /// Used together with [`cpb_size_du_scale`](SubPicHrdParams::cpb_size_du_scale) to specify
    /// the i-th CPB size when the CPB operates at sub-picture level.
    ///
    /// For any `i > 0`, `cpb_size_du_value_minus1[i]` is less than or equal to `cpb_size_du_value_minus1[i − 1]`.
    ///
    /// The value is in range \[0, 2^32 - 2\].
    ///
    /// Defines [`CpbSize[i]`](SubLayerHrdParameters::cpb_size).
    pub cpb_size_du_value_minus1: Option<u64>,
    /// Specifies (together with [`bit_rate_scale`](CommonInf::bit_rate_scale)) the maximum input bit rate for
    /// the i-th CPB when the CPB operates at the sub-picture level.
    ///
    /// For any `i > 0`, `bit_rate_du_value_minus1[i]` shall be greater than `bit_rate_du_value_minus1[i − 1]`.
    ///
    /// The value is in range \[0, 2^32 - 2\].
    ///
    /// Defines [`BitRate[i]`](SubLayerHrdParameters::bit_rate).
    pub bit_rate_du_value_minus1: Option<u64>,
    /// Equal to `false` specifies that to decode this CVS by the HRD using the i-th CPB specification, the
    /// hypothetical stream scheduler (HSS) operates in an intermittent bit rate mode.
    ///
    /// Equal to `true` specifies that the HSS operates in a constant bit rate (CBR) mode.
    pub cbr_flag: bool,
}

impl SubLayerHrdParameters {
    fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        nal_hrd: bool,
        cpb_cnt: u64,
        sub_pic_hrd_params_present_flag: bool,
    ) -> io::Result<Vec<Self>> {
        let mut parameters: Vec<Self> = Vec::with_capacity(cpb_cnt as usize);

        for i in 0..cpb_cnt as usize {
            let bit_rate_value_minus1 = bit_reader.read_exp_golomb()?;
            range_check!(bit_rate_value_minus1, 0, 2u64.pow(32) - 2)?;
            let bit_rate_value_minus1 = bit_rate_value_minus1 as u32;
            if i > 0 && bit_rate_value_minus1 <= parameters[i - 1].bit_rate_value_minus1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "bit_rate_value_minus1 must be greater than the previous value",
                ));
            }

            let cpb_size_value_minus1 = bit_reader.read_exp_golomb()?;
            range_check!(cpb_size_value_minus1, 0, 2u64.pow(32) - 2)?;
            let cpb_size_value_minus1 = cpb_size_value_minus1 as u32;
            if i > 0 && cpb_size_value_minus1 > parameters[i - 1].cpb_size_value_minus1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "cpb_size_value_minus1 must be less than or equal to the previous value",
                ));
            }

            let mut cpb_size_du_value_minus1 = None;
            let mut bit_rate_du_value_minus1 = None;
            if sub_pic_hrd_params_present_flag {
                cpb_size_du_value_minus1 = Some(bit_reader.read_exp_golomb()?);
                bit_rate_du_value_minus1 = Some(bit_reader.read_exp_golomb()?);
            }

            let cbr_flag = bit_reader.read_bit()?;

            parameters.push(Self {
                nal_hrd,
                bit_rate_value_minus1,
                cpb_size_value_minus1,
                cpb_size_du_value_minus1,
                bit_rate_du_value_minus1,
                cbr_flag,
            });
        }

        Ok(parameters)
    }

    /// When `SubPicHrdFlag` is equal to `false`, the bit rate in bits per second is given by:
    /// `BitRate[i] = (bit_rate_value_minus1[ i ] + 1) * 2^(6 + bit_rate_scale)` (E-77)
    ///
    /// When `SubPicHrdFlag` is equal to `true`, the bit rate in bits per second is given by:
    /// `BitRate[i] = (bit_rate_du_value_minus1[ i ] + 1) * 2^(6 + bit_rate_scale)` (E-80)
    ///
    /// When `SubPicHrdFlag` is equal to `true` and the `bit_rate_du_value_minus1[i]` syntax element is not present,
    /// the value of `BitRate[i]` is inferred to be equal to `BrVclFactor * MaxBR` for VCL HRD parameters and to be
    /// equal to `BrNalFactor * MaxBR` for NAL HRD parameters, where `MaxBR`, `BrVclFactor` and `BrNalFactor` are
    /// specified in ISO/IEC 23008-2 - A.4.
    pub fn bit_rate(
        &self,
        sub_pic_hrd_flag: bool,
        bit_rate_scale: u8,
        br_vcl_factor: u64,
        br_nal_factor: u64,
        max_br: u64,
    ) -> u64 {
        let value = if !sub_pic_hrd_flag {
            self.bit_rate_value_minus1 as u64
        } else {
            self.bit_rate_du_value_minus1.unwrap_or_else(|| {
                if self.nal_hrd {
                    br_nal_factor * max_br
                } else {
                    br_vcl_factor * max_br
                }
            })
        };
        (value + 1) * 2u64.pow(6 + bit_rate_scale as u32)
    }

    /// When `SubPicHrdFlag` is equal to `false`, the CPB size in bits is given by:
    /// `CpbSize[i] = (cpb_size_value_minus1[ i ] + 1) * 2^(4 + cpb_size_scale)` (E-78)
    ///
    /// When `SubPicHrdFlag` is equal to `true`, the CPB size in bits is given by:
    /// `CpbSize[i] = (cpb_size_du_value_minus1[ i ] + 1) * 2^(4 + cpb_size_du_scale)` (E-79)
    ///
    /// When `SubPicHrdFlag` is equal to `true` and the `cpb_size_du_value_minus1[i]` syntax element is not present,
    /// the value of `CpbSize[i]` is inferred to be equal to `CpbVclFactor * MaxCPB` for VCL HRD parameters and to
    /// be equal to `CpbNalFactor * MaxCPB` for NAL HRD parameters, where `MaxCPB`, `CpbVclFactor` and
    /// `CpbNalFactor` are specified in ISO/IEC 23008-2 - A.4.
    pub fn cpb_size(
        &self,
        sub_pic_hrd_flag: bool,
        cpb_size_scale: u8,
        cpb_vcl_factor: u64,
        cpb_nal_factor: u64,
        max_cpb: u64,
    ) -> u64 {
        let value = if !sub_pic_hrd_flag {
            self.bit_rate_value_minus1 as u64
        } else {
            self.bit_rate_du_value_minus1.unwrap_or_else(|| {
                if self.nal_hrd {
                    cpb_nal_factor * max_cpb
                } else {
                    cpb_vcl_factor * max_cpb
                }
            })
        };
        (value + 1) * 2u64.pow(4 + cpb_size_scale as u32)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use byteorder::WriteBytesExt;
    use scuffle_bytes_util::{BitReader, BitWriter};
    use scuffle_expgolomb::BitWriterExpGolombExt;

    use super::HrdParameters;

    #[test]
    fn test_parse() {
        let mut data = Vec::new();
        let mut bit_writer = BitWriter::new(&mut data);

        bit_writer.write_bit(true).unwrap(); // nal_hrd_parameters_present_flag
        bit_writer.write_bit(true).unwrap(); // vcl_hrd_parameters_present_flag

        bit_writer.write_bit(false).unwrap(); // sub_pic_hrd_params_present_flag
        bit_writer.write_bits(0, 4).unwrap(); // bit_rate_scale
        bit_writer.write_bits(0, 4).unwrap(); // cpb_size_scale
        bit_writer.write_bits(0, 5).unwrap(); // initial_cpb_removal_delay_length_minus1
        bit_writer.write_bits(0, 5).unwrap(); // au_cpb_removal_delay_length_minus1
        bit_writer.write_bits(0, 5).unwrap(); // dpb_output_delay_length_minus1

        // Sub-layers
        bit_writer.write_bit(true).unwrap(); // fixed_pic_rate_general_flag

        bit_writer.write_exp_golomb(0).unwrap(); // elemental_duration_in_tc_minus1

        bit_writer.write_exp_golomb(0).unwrap(); // cpb_cnt_minus1

        // SubLayerHrdParameters
        bit_writer.write_exp_golomb(0).unwrap(); // bit_rate_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // cpb_size_value_minus1
        bit_writer.write_bit(false).unwrap(); // cbr_flag

        // SubLayerHrdParameters
        bit_writer.write_exp_golomb(0).unwrap(); // bit_rate_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // cpb_size_value_minus1
        bit_writer.write_bit(false).unwrap(); // cbr_flag

        bit_writer.write_bits(0, 8).unwrap(); // fill remaining bits

        let mut bit_reader = BitReader::new(&data[..]);
        let hrd_parameters = HrdParameters::parse(&mut bit_reader, true, 0).unwrap();
        assert_eq!(hrd_parameters.common_inf.bit_rate_scale, Some(0));
        assert_eq!(hrd_parameters.common_inf.cpb_size_scale, Some(0));
        assert_eq!(hrd_parameters.common_inf.initial_cpb_removal_delay_length_minus1, 0);
        assert_eq!(hrd_parameters.common_inf.au_cpb_removal_delay_length_minus1, 0);
        assert_eq!(hrd_parameters.common_inf.dpb_output_delay_length_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers.len(), 1);
        assert!(hrd_parameters.sub_layers[0].fixed_pic_rate_general_flag);
        assert!(hrd_parameters.sub_layers[0].fixed_pic_rate_within_cvs_flag);
        assert_eq!(hrd_parameters.sub_layers[0].elemental_duration_in_tc_minus1, Some(0));
        assert!(!hrd_parameters.sub_layers[0].low_delay_hrd_flag);
        assert_eq!(hrd_parameters.sub_layers[0].cpb_cnt_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters.len(), 2);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate_value_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size_value_minus1, 0);
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size_du_value_minus1,
            None
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate_du_value_minus1,
            None
        );
        assert!(!hrd_parameters.sub_layers[0].sub_layer_parameters[0].cbr_flag);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate_value_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size_value_minus1, 0);
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size_du_value_minus1,
            None
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate_du_value_minus1,
            None
        );
        assert!(!hrd_parameters.sub_layers[0].sub_layer_parameters[1].cbr_flag);

        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate(false, 0, 0, 0, 0),
            64
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate(true, 0, 0, 0, 0),
            64
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate(false, 0, 0, 0, 0),
            64
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate(true, 0, 0, 0, 0),
            64
        );

        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size(false, 0, 0, 0, 0),
            16
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size(true, 0, 0, 0, 0),
            16
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size(false, 0, 0, 0, 0),
            16
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size(true, 0, 0, 0, 0),
            16
        );
    }

    #[test]
    fn test_parse_sub_pic_hrd_params_present_flag() {
        let mut data = Vec::new();
        let mut bit_writer = BitWriter::new(&mut data);

        bit_writer.write_bit(true).unwrap(); // nal_hrd_parameters_present_flag
        bit_writer.write_bit(true).unwrap(); // vcl_hrd_parameters_present_flag

        bit_writer.write_bit(true).unwrap(); // sub_pic_hrd_params_present_flag
        bit_writer.write_u8(42).unwrap(); // tick_divisor_minus2
        bit_writer.write_bits(0, 5).unwrap(); // du_cpb_removal_delay_increment_length_minus1
        bit_writer.write_bit(false).unwrap(); // sub_pic_cpb_params_in_pic_timing_sei_flag
        bit_writer.write_bits(0, 5).unwrap(); // dpb_output_delay_du_length_minus1
        bit_writer.write_bits(0, 4).unwrap(); // bit_rate_scale
        bit_writer.write_bits(0, 4).unwrap(); // cpb_size_scale
        bit_writer.write_bits(0, 4).unwrap(); // cpb_size_du_scale
        bit_writer.write_bits(0, 5).unwrap(); // initial_cpb_removal_delay_length_minus1
        bit_writer.write_bits(0, 5).unwrap(); // au_cpb_removal_delay_length_minus1
        bit_writer.write_bits(0, 5).unwrap(); // dpb_output_delay_length_minus1

        // Sub-layers
        bit_writer.write_bit(true).unwrap(); // fixed_pic_rate_general_flag

        bit_writer.write_exp_golomb(0).unwrap(); // elemental_duration_in_tc_minus1

        bit_writer.write_exp_golomb(0).unwrap(); // cpb_cnt_minus1

        // SubLayerHrdParameters
        bit_writer.write_exp_golomb(0).unwrap(); // bit_rate_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // cpb_size_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // cpb_size_du_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // bit_rate_du_value_minus1
        bit_writer.write_bit(false).unwrap(); // cbr_flag

        // SubLayerHrdParameters
        bit_writer.write_exp_golomb(0).unwrap(); // bit_rate_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // cpb_size_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // cpb_size_du_value_minus1
        bit_writer.write_exp_golomb(0).unwrap(); // bit_rate_du_value_minus1
        bit_writer.write_bit(false).unwrap(); // cbr_flag

        bit_writer.write_bits(0, 8).unwrap(); // fill remaining bits

        let mut bit_reader = BitReader::new(&data[..]);
        let hrd_parameters = HrdParameters::parse(&mut bit_reader, true, 0).unwrap();
        assert_eq!(hrd_parameters.common_inf.bit_rate_scale, Some(0));
        assert_eq!(hrd_parameters.common_inf.cpb_size_scale, Some(0));
        assert_eq!(hrd_parameters.common_inf.initial_cpb_removal_delay_length_minus1, 0);
        assert_eq!(hrd_parameters.common_inf.au_cpb_removal_delay_length_minus1, 0);
        assert_eq!(hrd_parameters.common_inf.dpb_output_delay_length_minus1, 0);
        assert!(hrd_parameters.common_inf.sub_pic_hrd_params.is_some());
        assert_eq!(hrd_parameters.sub_layers.len(), 1);
        assert!(hrd_parameters.sub_layers[0].fixed_pic_rate_general_flag);
        assert!(hrd_parameters.sub_layers[0].fixed_pic_rate_within_cvs_flag);
        assert_eq!(hrd_parameters.sub_layers[0].elemental_duration_in_tc_minus1, Some(0));
        assert!(!hrd_parameters.sub_layers[0].low_delay_hrd_flag);
        assert_eq!(hrd_parameters.sub_layers[0].cpb_cnt_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters.len(), 2);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate_value_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size_value_minus1, 0);
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size_du_value_minus1,
            Some(0)
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate_du_value_minus1,
            Some(0)
        );
        assert!(!hrd_parameters.sub_layers[0].sub_layer_parameters[0].cbr_flag);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate_value_minus1, 0);
        assert_eq!(hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size_value_minus1, 0);
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size_du_value_minus1,
            Some(0)
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate_du_value_minus1,
            Some(0)
        );
        assert!(!hrd_parameters.sub_layers[0].sub_layer_parameters[1].cbr_flag);

        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate(false, 0, 0, 0, 0),
            64
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].bit_rate(true, 0, 0, 0, 0),
            64
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate(false, 0, 0, 0, 0),
            64
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].bit_rate(true, 0, 0, 0, 0),
            64
        );

        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size(false, 0, 0, 0, 0),
            16
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[0].cpb_size(true, 0, 0, 0, 0),
            16
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size(false, 0, 0, 0, 0),
            16
        );
        assert_eq!(
            hrd_parameters.sub_layers[0].sub_layer_parameters[1].cpb_size(true, 0, 0, 0, 0),
            16
        );
    }
}
