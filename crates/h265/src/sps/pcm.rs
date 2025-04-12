use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

#[derive(Debug, Clone, PartialEq)]
pub struct Pcm {
    pub pcm_sample_bit_depth_luma_minus1: u8,
    pub pcm_sample_bit_depth_chroma_minus1: u8,
    pub log2_min_pcm_luma_coding_block_size_minus3: u64,
    pub log2_diff_max_min_pcm_luma_coding_block_size: u64,
    pub pcm_loop_filter_disabled_flag: bool,
}

impl Pcm {
    pub fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        Ok(Self {
            pcm_sample_bit_depth_luma_minus1: bit_reader.read_bits(4)? as u8,
            pcm_sample_bit_depth_chroma_minus1: bit_reader.read_bits(4)? as u8,
            log2_min_pcm_luma_coding_block_size_minus3: bit_reader.read_exp_golomb()?,
            log2_diff_max_min_pcm_luma_coding_block_size: bit_reader.read_exp_golomb()?,
            pcm_loop_filter_disabled_flag: bit_reader.read_bit()?,
        })
    }
}
