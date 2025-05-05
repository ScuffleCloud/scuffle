use std::fmt::Debug;
use std::io;

use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Short-term reference picture set syntax.
///
/// `st_ref_pic_set(stRpsIdx)`
///
/// - ISO/IEC 23008-2 - 7.3.7
/// - ISO/IEC 23008-2 - 7.4.8
#[derive(Debug, Clone, PartialEq)]
pub struct ShortTermRefPicSets {
    /// `NumDeltaPocs[stRpsIdx]`
    pub num_delta_pocs: Vec<u64>,
    /// `NumPositivePics[stRpsIdx]`
    pub num_positive_pics: Vec<u64>,
    /// `NumNegativePics[stRpsIdx]`
    pub num_negative_pics: Vec<u64>,
    /// `DeltaPocS1[stRpsIdx][j]`
    pub delta_poc_s1: Vec<Vec<i64>>,
    /// `DeltaPocS0[stRpsIdx][j]`
    pub delta_poc_s0: Vec<Vec<i64>>,
    /// `UsedByCurrPicS0[stRpsIdx][j]`
    pub used_by_curr_pic_s0: Vec<Vec<bool>>,
    /// `UsedByCurrPicS1[stRpsIdx][j]`
    pub used_by_curr_pic_s1: Vec<Vec<bool>>,
}

impl ShortTermRefPicSets {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        num_short_term_ref_pic_sets: usize,
        nuh_layer_id: u8,
        sps_max_dec_pic_buffering_minus1_at_sps_max_sub_layers_minus1: u64,
    ) -> io::Result<Self> {
        let mut num_delta_pocs = Vec::with_capacity(num_short_term_ref_pic_sets);

        // num_short_term_ref_pic_sets is bound above by 64
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
                    range_check!(delta_idx_minus1, 0, st_rps_idx - 1)?;
                }

                // (7-59)
                let ref_rps_idx = st_rps_idx - (delta_idx_minus1 + 1);

                let delta_rps_sign = bit_reader.read_bit()?;
                let abs_delta_rps_minus1 = bit_reader.read_exp_golomb()?;
                range_check!(abs_delta_rps_minus1, 0, 2u64.pow(15) - 1)?;
                // (7-60)
                let delta_rps = (1 - 2 * delta_rps_sign as i64) * (abs_delta_rps_minus1 + 1) as i64;

                // num_delta_pocs is bound above by 32 ((7-71) see below)
                let len = num_delta_pocs[ref_rps_idx] as usize + 1;
                let mut used_by_curr_pic_flag = vec![false; len];
                let mut use_delta_flag = vec![true; len];
                for j in 0..len {
                    used_by_curr_pic_flag[j] = bit_reader.read_bit()?;
                    if !used_by_curr_pic_flag[j] {
                        use_delta_flag[j] = bit_reader.read_bit()?;
                    }
                }

                delta_poc_s0.push(vec![0; len]);
                delta_poc_s1.push(vec![0; len]);
                used_by_curr_pic_s0.push(vec![false; len]);
                used_by_curr_pic_s1.push(vec![false; len]);

                // Calculate derived values as defined as (7-61) and (7-62) by the spec
                let mut i = 0;
                if let Some(start) = num_positive_pics[ref_rps_idx].checked_sub(1).map(|s| s as usize) {
                    for j in (0..=start).rev() {
                        let d_poc = delta_poc_s1[ref_rps_idx][j] + delta_rps;
                        if d_poc < 0 && use_delta_flag[num_negative_pics[ref_rps_idx] as usize + j] {
                            delta_poc_s0[st_rps_idx][i] = d_poc;
                            used_by_curr_pic_s0[st_rps_idx][i] =
                                used_by_curr_pic_flag[num_negative_pics[ref_rps_idx] as usize + j];
                            i += 1;
                        }
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
                // This is a sanity check just for safety, it should be unreachable
                // num_negative_pics is said to be bound by
                // sps_max_dec_pic_buffering_minus1[sps_max_sub_layers_minus1]
                // which itself is bound by 16
                range_check!(num_negative_pics[st_rps_idx], 0, 16)?;

                i = 0;
                if let Some(start) = num_negative_pics[ref_rps_idx].checked_sub(1).map(|s| s as usize) {
                    for j in (0..=start).rev() {
                        let d_poc = delta_poc_s0[ref_rps_idx][j] + delta_rps;
                        if d_poc > 0 && use_delta_flag[j] {
                            delta_poc_s1[st_rps_idx][i] = d_poc;
                            used_by_curr_pic_s1[st_rps_idx][i] = used_by_curr_pic_flag[j];
                            i += 1;
                        }
                    }
                }

                if delta_rps > 0 && use_delta_flag[num_delta_pocs[ref_rps_idx] as usize] {
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
                // This is a sanity check just for safety, it should be unreachable
                // num_positive_pics is said to be bound by
                // sps_max_dec_pic_buffering_minus1[sps_max_sub_layers_minus1] - num_negative_pics
                // which itself is bound by 16
                range_check!(num_negative_pics[st_rps_idx], 0, 16)?;
            } else {
                num_negative_pics[st_rps_idx] = bit_reader.read_exp_golomb()?;
                num_positive_pics[st_rps_idx] = bit_reader.read_exp_golomb()?;

                let upper_bound = if nuh_layer_id == 0 {
                    // bound above by 16
                    sps_max_dec_pic_buffering_minus1_at_sps_max_sub_layers_minus1
                } else {
                    16
                };
                range_check!(num_negative_pics[st_rps_idx], 0, upper_bound)?;

                let upper_bound = if nuh_layer_id == 0 {
                    // bound above by 16
                    sps_max_dec_pic_buffering_minus1_at_sps_max_sub_layers_minus1
                        .saturating_sub(num_negative_pics[st_rps_idx])
                } else {
                    16
                };
                range_check!(num_positive_pics[st_rps_idx], 0, upper_bound)?;

                delta_poc_s0.push(vec![0; num_negative_pics[st_rps_idx] as usize]);
                used_by_curr_pic_s0.push(vec![false; num_negative_pics[st_rps_idx] as usize]);

                for i in 0..num_negative_pics[st_rps_idx] as usize {
                    let delta_poc_s0_minus1 = bit_reader.read_exp_golomb()?;
                    range_check!(delta_poc_s0_minus1, 0, 2u64.pow(15) - 1)?;
                    if i == 0 {
                        // (7-67)
                        delta_poc_s0[st_rps_idx][i] = -(delta_poc_s0_minus1 as i64 + 1);
                    } else {
                        // (7-69)
                        delta_poc_s0[st_rps_idx][i] = delta_poc_s0[st_rps_idx][i - 1] - (delta_poc_s0_minus1 as i64 + 1);
                    }

                    let used_by_curr_pic_s0_flag = bit_reader.read_bit()?;
                    used_by_curr_pic_s0[st_rps_idx][i] = used_by_curr_pic_s0_flag;
                }

                delta_poc_s1.push(vec![0; num_positive_pics[st_rps_idx] as usize]);
                used_by_curr_pic_s1.push(vec![false; num_positive_pics[st_rps_idx] as usize]);

                for i in 0..num_positive_pics[st_rps_idx] as usize {
                    let delta_poc_s1_minus1 = bit_reader.read_exp_golomb()?;
                    range_check!(delta_poc_s1_minus1, 0, 2u64.pow(15) - 1)?;
                    if i == 0 {
                        // (7-68)
                        delta_poc_s1[st_rps_idx][i] = delta_poc_s1_minus1 as i64 + 1;
                    } else {
                        // (7-70)
                        delta_poc_s1[st_rps_idx][i] = delta_poc_s1[st_rps_idx][i - 1] + delta_poc_s1_minus1 as i64 + 1;
                    }

                    let used_by_curr_pic_s1_flag = bit_reader.read_bit()?;
                    used_by_curr_pic_s1[st_rps_idx][i] = used_by_curr_pic_s1_flag;
                }
            }

            // (7-71)
            num_delta_pocs.push(num_negative_pics[st_rps_idx] + num_positive_pics[st_rps_idx]);
            // both num_negative_pics and num_positive_pics are bound above by 16
            // => num_delta_pocs[st_rps_idx] <= 32
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
