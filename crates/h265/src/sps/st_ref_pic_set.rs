use std::fmt::Debug;
use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::range_check::range_check;

#[derive(Debug, Clone, PartialEq)]
pub struct ShortTermRefPicSets {
    pub num_delta_pocs: Vec<u64>,
    pub num_positive_pics: Vec<u64>,
    pub num_negative_pics: Vec<u64>,
    pub delta_poc_s1: Vec<Vec<i64>>,
    pub delta_poc_s0: Vec<Vec<i64>>,
    pub used_by_curr_pic_s0: Vec<Vec<bool>>,
    pub used_by_curr_pic_s1: Vec<Vec<bool>>,
}

impl ShortTermRefPicSets {
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        num_short_term_ref_pic_sets: usize,
        nuh_layer_id: u8,
        sps_max_dec_pic_buffering_minus1_at_sps_max_sub_layers_minus1: u64,
    ) -> io::Result<Self> {
        let mut num_delta_pocs = Vec::with_capacity(num_short_term_ref_pic_sets);

        let mut num_positive_pics = vec![0u64; num_short_term_ref_pic_sets];
        let mut num_negative_pics = vec![0u64; num_short_term_ref_pic_sets];
        let mut delta_poc_s1 = Vec::with_capacity(num_short_term_ref_pic_sets);
        let mut delta_poc_s0 = Vec::with_capacity(num_short_term_ref_pic_sets);
        let mut used_by_curr_pic_s0 = Vec::with_capacity(num_short_term_ref_pic_sets);
        let mut used_by_curr_pic_s1 = Vec::with_capacity(num_short_term_ref_pic_sets);

        for st_rps_idx in 0..num_short_term_ref_pic_sets {
            let mut inter_ref_pic_set_prediction_flag = false;
            if st_rps_idx != 0 {
                inter_ref_pic_set_prediction_flag = bit_reader.read_bit()?;
            }

            if inter_ref_pic_set_prediction_flag {
                let mut delta_idx_minus1 = 0;
                if st_rps_idx == num_short_term_ref_pic_sets {
                    delta_idx_minus1 = bit_reader.read_exp_golomb()? as usize;

                    if (0..=st_rps_idx - 1).contains(&delta_idx_minus1) {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "delta_idx_minus1 is out of range"));
                    }
                }

                let ref_rps_idx = st_rps_idx - (delta_idx_minus1 + 1);

                let delta_rps_sign = bit_reader.read_bit()?;
                let abs_delta_rps_minus1 = bit_reader.read_exp_golomb()?;
                let delta_rps = (1 - 2 * delta_rps_sign as i64) * (abs_delta_rps_minus1 + 1) as i64;

                let len = num_delta_pocs[ref_rps_idx] as usize;
                let mut used_by_curr_pic_flag = vec![false; len];
                let mut use_delta_flag = vec![true; len];
                for j in 0..len {
                    used_by_curr_pic_flag[j] = bit_reader.read_bit()?;
                    if !used_by_curr_pic_flag[j] {
                        use_delta_flag[j] = bit_reader.read_bit()?;
                    }
                }

                // TODO: Figure out what the actual size of these vectors should be
                // I hope this is enough for all cases
                let pic_sum = num_positive_pics[ref_rps_idx] as usize + num_negative_pics[ref_rps_idx] as usize;
                delta_poc_s0.push(vec![0; pic_sum]);
                delta_poc_s1.push(vec![0; pic_sum]);
                used_by_curr_pic_s0.push(vec![false; pic_sum]);
                used_by_curr_pic_s1.push(vec![false; pic_sum]);

                // Calculate derived values as defined by the spec
                // TODO: This whole code segment is pretty risky because of all the unckecked indexing
                let mut i = 0;
                let start = num_positive_pics[ref_rps_idx] as i64 - 1;
                for j in (0..=start).rev() {
                    let d_poc = delta_poc_s1[ref_rps_idx][j as usize] + delta_rps;
                    if d_poc < 0 && use_delta_flag[num_negative_pics[ref_rps_idx] as usize + j as usize] {
                        delta_poc_s0[st_rps_idx][i] = d_poc;
                        used_by_curr_pic_s0[st_rps_idx][i] =
                            used_by_curr_pic_flag[num_negative_pics[ref_rps_idx] as usize + j as usize];
                        i += 1;
                    }
                }

                if delta_rps < 0 && use_delta_flag[num_delta_pocs[ref_rps_idx] as usize] {
                    delta_poc_s0[st_rps_idx][i] = delta_rps;
                    used_by_curr_pic_s0[st_rps_idx][i] = used_by_curr_pic_flag[num_delta_pocs[ref_rps_idx] as usize];
                    i += 1;
                }

                for j in 0..num_negative_pics[ref_rps_idx] as usize {
                    let d_poc = delta_poc_s0[ref_rps_idx][j] + delta_rps;
                    if d_poc < 0 && use_delta_flag[j] {
                        delta_poc_s0[st_rps_idx][i] = d_poc;
                        used_by_curr_pic_s0[st_rps_idx][i] = used_by_curr_pic_flag[j];
                        i += 1;
                    }
                }

                num_negative_pics[st_rps_idx] = i as u64;

                i = 0;
                let start = num_negative_pics[ref_rps_idx] as i64 - 1;
                for j in (0..=start).rev() {
                    let d_poc = delta_poc_s0[ref_rps_idx][j as usize] + delta_rps;
                    if d_poc > 0 && use_delta_flag[j as usize] {
                        delta_poc_s1[st_rps_idx][i] = d_poc;
                        used_by_curr_pic_s1[st_rps_idx][i] = used_by_curr_pic_flag[j as usize];
                        i += 1;
                    }
                }

                if delta_rps > 0
                    && use_delta_flag
                        .get(num_delta_pocs[ref_rps_idx] as usize)
                        .copied()
                        .unwrap_or(false)
                {
                    delta_poc_s1[st_rps_idx][i] = delta_rps;
                    used_by_curr_pic_s1[st_rps_idx][i] = used_by_curr_pic_flag[num_delta_pocs[ref_rps_idx] as usize];
                    i += 1;
                }

                for j in 0..num_positive_pics[ref_rps_idx] as usize {
                    let d_poc = delta_poc_s1[ref_rps_idx][j] + delta_rps;
                    if d_poc > 0 && use_delta_flag[num_negative_pics[ref_rps_idx] as usize + j] {
                        delta_poc_s1[st_rps_idx][i] = d_poc;
                        used_by_curr_pic_s1[st_rps_idx][i] =
                            used_by_curr_pic_flag[num_negative_pics[ref_rps_idx] as usize + j];
                        i += 1;
                    }
                }

                num_positive_pics[st_rps_idx] = i as u64;

                num_delta_pocs.push(num_negative_pics[st_rps_idx] + num_positive_pics[st_rps_idx]);
            } else {
                let num_negative_pics = bit_reader.read_exp_golomb()?;
                let num_positive_pics = bit_reader.read_exp_golomb()?;

                let upper_bound = if nuh_layer_id == 0 {
                    sps_max_dec_pic_buffering_minus1_at_sps_max_sub_layers_minus1
                } else {
                    // The spec does not limit the value in this case so we set it to a reasonable value
                    1000
                };
                range_check!(num_negative_pics, 0, upper_bound)?;

                let upper_bound = if nuh_layer_id == 0 {
                    sps_max_dec_pic_buffering_minus1_at_sps_max_sub_layers_minus1.saturating_sub(num_negative_pics)
                } else {
                    // The spec does not limit the value in this case so we set it to a reasonable value
                    1000
                };
                range_check!(num_positive_pics, 0, upper_bound)?;

                delta_poc_s0.push(vec![0; num_negative_pics as usize]);
                used_by_curr_pic_s0.push(vec![false; num_negative_pics as usize]);

                for i in 0..num_negative_pics as usize {
                    let delta_poc_s0_minus1 = bit_reader.read_exp_golomb()?;
                    range_check!(delta_poc_s0_minus1, 0, 2u64.pow(15) - 1)?;
                    delta_poc_s0[st_rps_idx][i] = delta_poc_s0_minus1 as i64 + 1;

                    let used_by_curr_pic_s0_flag = bit_reader.read_bit()?;
                    used_by_curr_pic_s0[st_rps_idx][i] = used_by_curr_pic_s0_flag;
                }

                delta_poc_s1.push(vec![0; num_positive_pics as usize]);
                used_by_curr_pic_s1.push(vec![false; num_positive_pics as usize]);

                for i in 0..num_positive_pics as usize {
                    let delta_poc_s1_minus1 = bit_reader.read_exp_golomb()?;
                    range_check!(delta_poc_s1_minus1, 0, 2u64.pow(15) - 1)?;
                    delta_poc_s1[st_rps_idx][i] = delta_poc_s1_minus1 as i64 + 1;

                    let used_by_curr_pic_s1_flag = bit_reader.read_bit()?;
                    used_by_curr_pic_s1[st_rps_idx][i] = used_by_curr_pic_s1_flag;
                }

                num_delta_pocs.push(num_negative_pics + num_positive_pics);
            }
        }

        Ok(Self {
            num_delta_pocs,
            num_positive_pics,
            num_negative_pics,
            delta_poc_s1,
            delta_poc_s0,
            used_by_curr_pic_s0,
            used_by_curr_pic_s1,
        })
    }
}
