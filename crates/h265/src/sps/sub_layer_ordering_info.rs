use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::range_check::range_check;

#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerOrderingInfo {
    pub sps_max_dec_pic_buffering_minus1: Vec<u64>,
    pub sps_max_num_reorder_pics: Vec<u64>,
    pub sps_max_latency_increase_plus1: Vec<u32>,
}

impl SubLayerOrderingInfo {
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        sps_sub_layer_ordering_info_present_flag: bool,
        sps_max_sub_layers_minus1: u8,
    ) -> io::Result<Self> {
        let mut sps_max_dec_pic_buffering_minus1 = vec![0; sps_max_sub_layers_minus1 as usize + 1];
        let mut sps_max_num_reorder_pics = vec![0; sps_max_sub_layers_minus1 as usize + 1];
        let mut sps_max_latency_increase_plus1 = vec![0; sps_max_sub_layers_minus1 as usize + 1];

        if sps_sub_layer_ordering_info_present_flag {
            for i in 0..=sps_max_sub_layers_minus1 as usize {
                sps_max_dec_pic_buffering_minus1[i] = bit_reader.read_exp_golomb()?;
                if i > 0 && sps_max_dec_pic_buffering_minus1[i] < sps_max_dec_pic_buffering_minus1[i - 1] {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "sps_max_dec_pic_buffering_minus1[i] must be greater than or equal to sps_max_dec_pic_buffering_minus1[i-1]",
                    ));
                }

                sps_max_num_reorder_pics[i] = bit_reader.read_exp_golomb()?;
                range_check!(sps_max_num_reorder_pics[i], 0, sps_max_dec_pic_buffering_minus1[i])?;
                if i > 0 && sps_max_num_reorder_pics[i] < sps_max_num_reorder_pics[i - 1] {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "sps_max_num_reorder_pics[i] must be greater than or equal to sps_max_num_reorder_pics[i-1]",
                    ));
                }

                let sps_max_latency_increase_plus1_i = bit_reader.read_exp_golomb()?;
                range_check!(sps_max_latency_increase_plus1_i, 0, 2u64.pow(32) - 2)?;
                sps_max_latency_increase_plus1[i] = sps_max_latency_increase_plus1_i as u32;
            }
        } else {
            // From the spec, page 108 and 109:
            // When sps_max_dec_pic_buffering_minus1[i] is not present (...) due to
            // sps_sub_layer_ordering_info_present_flag being equal to 0, it is inferred to be equal to
            // sps_max_dec_pic_buffering_minus1[sps_max_sub_layers_minus1].

            let sps_max_dec_pic_buffering_minus1_i = bit_reader.read_exp_golomb()?;
            sps_max_dec_pic_buffering_minus1.fill(sps_max_dec_pic_buffering_minus1_i);

            let sps_max_num_reorder_pics_i = bit_reader.read_exp_golomb()?;
            range_check!(sps_max_num_reorder_pics_i, 0, sps_max_dec_pic_buffering_minus1_i)?;
            sps_max_num_reorder_pics.fill(sps_max_num_reorder_pics_i);

            let sps_max_latency_increase_plus1_i = bit_reader.read_exp_golomb()?;
            range_check!(sps_max_latency_increase_plus1_i, 0, 2u64.pow(32) - 2)?;
            sps_max_latency_increase_plus1.fill(sps_max_latency_increase_plus1_i as u32);
        }

        Ok(SubLayerOrderingInfo {
            sps_max_dec_pic_buffering_minus1,
            sps_max_num_reorder_pics,
            sps_max_latency_increase_plus1,
        })
    }

    pub fn sps_max_latency_pictures(&self) -> Vec<Option<u64>> {
        self.sps_max_num_reorder_pics
            .iter()
            .zip(self.sps_max_latency_increase_plus1.iter())
            .map(|(reorder, latency)| Some(reorder + latency.checked_sub(1)? as u64))
            .collect()
    }

    pub fn sps_max_latency_pictures_at(&self, i: usize) -> Option<u64> {
        Some(self.sps_max_num_reorder_pics.get(i)? + self.sps_max_latency_increase_plus1.get(i)?.checked_sub(1)? as u64)
    }
}
