use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerOrderingInfo {
    pub sps_max_dec_pic_buffering_minus1: Vec<u64>,
    pub sps_max_num_reorder_pics: Vec<u64>,
    pub sps_max_latency_increase_plus1: Vec<u64>,
}

impl SubLayerOrderingInfo {
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        sps_sub_layer_ordering_info_present_flag: bool,
        sps_max_sub_layers_minus1: u8,
    ) -> io::Result<Self> {
        let len = if sps_sub_layer_ordering_info_present_flag {
            (sps_max_sub_layers_minus1 + 1) as usize
        } else {
            1
        };

        let mut sps_max_dec_pic_buffering_minus1 = Vec::with_capacity(len);
        let mut sps_max_num_reorder_pics = Vec::with_capacity(len);
        let mut sps_max_latency_increase_plus1 = Vec::with_capacity(len);

        for _ in 0..len {
            sps_max_dec_pic_buffering_minus1.push(bit_reader.read_exp_golomb()?);
            sps_max_num_reorder_pics.push(bit_reader.read_exp_golomb()?);
            sps_max_latency_increase_plus1.push(bit_reader.read_exp_golomb()?);
        }

        Ok(SubLayerOrderingInfo {
            sps_max_dec_pic_buffering_minus1,
            sps_max_num_reorder_pics,
            sps_max_latency_increase_plus1,
        })
    }
}
