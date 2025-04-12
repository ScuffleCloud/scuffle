use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

#[derive(Debug, Clone, PartialEq)]
pub struct HrdParameters {
    pub common_inf: Option<CommonInf>,
    pub sub_layers: Vec<Vec<SubLayerHrdParameters>>,
}

impl HrdParameters {
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        common_inf_present_flag: bool,
        max_num_sub_layers_minus1: u8,
    ) -> io::Result<Self> {
        let mut common_inf = None;

        let mut nal_hrd_parameters_present_flag = false;
        let mut vcl_hrd_parameters_present_flag = false;

        if common_inf_present_flag {
            nal_hrd_parameters_present_flag = bit_reader.read_bit()?;
            vcl_hrd_parameters_present_flag = bit_reader.read_bit()?;

            if nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag {
                let mut sub_pic_hrd_params = None;

                let sub_pic_hrd_params_present_flag = bit_reader.read_bit()?;
                if sub_pic_hrd_params_present_flag {
                    let tick_divisor_minus2 = bit_reader.read_u8()?;
                    let du_cpb_removal_delay_increment_length_minus1 = bit_reader.read_bits(5)? as u8;
                    let sub_pic_cpb_params_in_pic_timing_sei_flag = bit_reader.read_bit()?;
                    let dpb_output_delay_du_length_minus1 = bit_reader.read_bits(5)? as u8;

                    sub_pic_hrd_params = Some(SubPicHrdParams {
                        tick_divisor_minus2,
                        du_cpb_removal_delay_increment_length_minus1,
                        sub_pic_cpb_params_in_pic_timing_sei_flag,
                        dpb_output_delay_du_length_minus1,
                        cpb_size_du_scale: 0, // will be replaced
                    });
                }

                let bit_rate_scale = bit_reader.read_bits(4)? as u8;
                let cpb_size_scale = bit_reader.read_bits(4)? as u8;

                if sub_pic_hrd_params_present_flag {
                    let cpb_size_du_scale = bit_reader.read_bits(4)? as u8;

                    // set the cpb_size_du_scale in sub_pic_hrd_params
                    if let Some(ref mut sub_pic_hrd_params) = sub_pic_hrd_params {
                        sub_pic_hrd_params.cpb_size_du_scale = cpb_size_du_scale;
                    }
                }

                let initial_cpb_removal_delay_length_minus1 = bit_reader.read_bits(5)? as u8;
                let au_cpb_removal_delay_length_minus1 = bit_reader.read_bits(5)? as u8;
                let dpb_output_delay_length_minus1 = bit_reader.read_bits(5)? as u8;

                common_inf = Some(CommonInf {
                    sub_pic_hrd_params,
                    bit_rate_scale,
                    cpb_size_scale,
                    initial_cpb_removal_delay_length_minus1,
                    au_cpb_removal_delay_length_minus1,
                    dpb_output_delay_length_minus1,
                });
            }
        }

        let mut sub_layers = Vec::with_capacity(max_num_sub_layers_minus1 as usize);

        for _ in 0..max_num_sub_layers_minus1 {
            let mut fixed_pic_rate_within_cvs_flag = true;

            let fixed_pic_rate_general_flag = bit_reader.read_bit()?;
            if !fixed_pic_rate_general_flag {
                fixed_pic_rate_within_cvs_flag = bit_reader.read_bit()?;
            }

            let mut low_delay_hrd_flag = false;
            if fixed_pic_rate_within_cvs_flag {
                bit_reader.read_exp_golomb()?; // elemental_duration_in_tc_minus1
            } else {
                low_delay_hrd_flag = bit_reader.read_bit()?;
            }

            let mut cpb_cnt_minus1 = 0;
            if !low_delay_hrd_flag {
                cpb_cnt_minus1 = bit_reader.read_exp_golomb()?;
            }

            let mut sub_layer_parameters = Vec::new();

            let sub_pic_hrd_params_present_flag = common_inf.as_ref().is_some_and(|i| i.sub_pic_hrd_params.is_some());

            if nal_hrd_parameters_present_flag {
                sub_layer_parameters.append(&mut SubLayerHrdParameters::parse(
                    bit_reader,
                    cpb_cnt_minus1 + 1,
                    sub_pic_hrd_params_present_flag,
                )?);
            }

            if vcl_hrd_parameters_present_flag {
                sub_layer_parameters.append(&mut SubLayerHrdParameters::parse(
                    bit_reader,
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
    pub bit_rate_scale: u8,
    pub cpb_size_scale: u8,
    pub initial_cpb_removal_delay_length_minus1: u8,
    pub au_cpb_removal_delay_length_minus1: u8,
    pub dpb_output_delay_length_minus1: u8,
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
pub struct SubLayer {
    pub parameters: SubLayerHrdParameters,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerHrdParameters {
    pub bit_rate_value_minus1: u64,
    pub cpb_size_value_minus1: u64,
    pub cpb_size_du_value_minus1: Option<u64>,
    pub bit_rate_du_value_minus1: Option<u64>,
    pub cbr_flag: bool,
}

impl SubLayerHrdParameters {
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        cpb_cnt: u64,
        sub_pic_hrd_params_present_flag: bool,
    ) -> io::Result<Vec<Self>> {
        let mut parameters = Vec::with_capacity(cpb_cnt as usize);

        for _ in 0..cpb_cnt {
            let bit_rate_value_minus1 = bit_reader.read_exp_golomb()?;
            let cpb_size_value_minus1 = bit_reader.read_exp_golomb()?;

            let mut cpb_size_du_value_minus1 = None;
            let mut bit_rate_du_value_minus1 = None;
            if sub_pic_hrd_params_present_flag {
                cpb_size_du_value_minus1 = Some(bit_reader.read_exp_golomb()?);
                bit_rate_du_value_minus1 = Some(bit_reader.read_exp_golomb()?);
            }

            let cbr_flag = bit_reader.read_bit()?;

            parameters.push(Self {
                bit_rate_value_minus1,
                cpb_size_value_minus1,
                cpb_size_du_value_minus1,
                bit_rate_du_value_minus1,
                cbr_flag,
            });
        }

        Ok(parameters)
    }
}
