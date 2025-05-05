use std::io;

use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Directly part of [SPS RBSP](crate::SpsRbsp).
#[derive(Debug, Clone, PartialEq)]
pub struct LongTermRefPics {
    /// Specifies the picture order count modulo `MaxPicOrderCntLsb` of the `i`-th
    /// candidate long-term reference picture specified in the SPS.
    pub lt_ref_pic_poc_lsb_sps: Vec<u64>,
    /// Equal to `false` specifies that the `i`-th candidate long-term reference picture
    /// specified in the SPS is not used for reference by a picture that includes in its long-term RPS the `i`-th
    /// candidate long-term reference picture specified in the SPS.
    pub used_by_curr_pic_lt_sps_flag: Vec<bool>,
}

impl LongTermRefPics {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        log2_max_pic_order_cnt_lsb_minus4: u8,
    ) -> Result<Self, io::Error> {
        let num_long_term_ref_pics_sps = bit_reader.read_exp_golomb()?;
        range_check!(num_long_term_ref_pics_sps, 0, 32)?;

        let mut lt_ref_pic_poc_lsb_sps = Vec::with_capacity(num_long_term_ref_pics_sps as usize);
        let mut used_by_curr_pic_lt_sps_flag = Vec::with_capacity(num_long_term_ref_pics_sps as usize);

        for _ in 0..num_long_term_ref_pics_sps {
            lt_ref_pic_poc_lsb_sps.push(bit_reader.read_bits(log2_max_pic_order_cnt_lsb_minus4 + 4)?);
            used_by_curr_pic_lt_sps_flag.push(bit_reader.read_bit()?);
        }

        Ok(Self {
            lt_ref_pic_poc_lsb_sps,
            used_by_curr_pic_lt_sps_flag,
        })
    }
}
