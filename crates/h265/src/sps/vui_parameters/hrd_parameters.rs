use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::range_check::range_check;

#[derive(Debug, Clone, PartialEq)]
pub struct HrdParameters {
    pub common_inf: CommonInf,
    pub sub_layers: Vec<Vec<SubLayerHrdParameters>>,
}

impl HrdParameters {
    pub fn parse<R: io::Read>(
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
            let mut fixed_pic_rate_within_cvs_flag = true;

            let fixed_pic_rate_general_flag = bit_reader.read_bit()?;
            if !fixed_pic_rate_general_flag {
                fixed_pic_rate_within_cvs_flag = bit_reader.read_bit()?;
            }

            let mut low_delay_hrd_flag = false;
            if fixed_pic_rate_within_cvs_flag {
                let elemental_duration_in_tc_minus1 = bit_reader.read_exp_golomb()?;
                range_check!(elemental_duration_in_tc_minus1, 0, 2047)?;
            } else {
                low_delay_hrd_flag = bit_reader.read_bit()?;
            }

            let mut cpb_cnt_minus1 = 0;
            if !low_delay_hrd_flag {
                cpb_cnt_minus1 = bit_reader.read_exp_golomb()?;
            }

            let mut sub_layer_parameters = Vec::new();

            let sub_pic_hrd_params_present_flag = common_inf.sub_pic_hrd_params.is_some();

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

            sub_layers.push(sub_layer_parameters);
        }

        Ok(HrdParameters { common_inf, sub_layers })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonInf {
    pub sub_pic_hrd_params: Option<SubPicHrdParams>,
    pub bit_rate_scale: Option<u8>,
    pub cpb_size_scale: Option<u8>,
    pub initial_cpb_removal_delay_length_minus1: u8,
    pub au_cpb_removal_delay_length_minus1: u8,
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

#[derive(Debug, Clone, PartialEq)]
pub struct SubPicHrdParams {
    pub tick_divisor_minus2: u8,
    pub du_cpb_removal_delay_increment_length_minus1: u8,
    pub sub_pic_cpb_params_in_pic_timing_sei_flag: bool,
    pub dpb_output_delay_du_length_minus1: u8,
    pub cpb_size_du_scale: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerHrdParameters {
    /// Internal field to store if this is a NAL or VCL HRD
    nal_hrd: bool,
    pub bit_rate_value_minus1: u32,
    pub cpb_size_value_minus1: u32,
    pub cpb_size_du_value_minus1: Option<u64>,
    pub bit_rate_du_value_minus1: Option<u64>,
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
