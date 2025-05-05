use std::io;

use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Directly part of [SPS RBSP](crate::SpsRbsp).
#[derive(Debug, Clone, PartialEq)]
pub struct Pcm {
    /// Defines [`PcmBitDepth_Y`](Pcm::pcm_bit_depth_y).
    pub pcm_sample_bit_depth_luma_minus1: u8,
    /// Defines [`PcmBitDepth_C`](Pcm::pcm_bit_depth_c).
    pub pcm_sample_bit_depth_chroma_minus1: u8,
    /// This value plus 3 specifies the minimum size of coding blocks with `pcm_flag` equal to `true`.
    ///
    /// Defines [`Log2MinIpcmCbSizeY`](Pcm::log2_min_ipcm_cb_size_y).
    pub log2_min_pcm_luma_coding_block_size_minus3: u64,
    /// Specifies the difference between the maximum and minimum size of coding blocks with `pcm_flag` equal to `true`.
    ///
    /// Defines [`Log2MaxIpcmCbSizeY`](Pcm::log2_max_ipcm_cb_size_y).
    pub log2_diff_max_min_pcm_luma_coding_block_size: u64,
    /// Specifies whether the loop filter process is disabled on reconstructed
    /// samples in a coding unit with `pcm_flag` equal to `true`.
    pub pcm_loop_filter_disabled_flag: bool,
}

impl Pcm {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        bit_depth_y: u8,
        bit_depth_c: u8,
        min_cb_log2_size_y: u64,
        ctb_log2_size_y: u64,
    ) -> io::Result<Self> {
        let pcm_sample_bit_depth_luma_minus1 = bit_reader.read_bits(4)? as u8;
        if pcm_sample_bit_depth_luma_minus1 + 1 > bit_depth_y {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PcmBitDepth_Y must be less than or equal to BitDepth_Y",
            ));
        }

        let pcm_sample_bit_depth_chroma_minus1 = bit_reader.read_bits(4)? as u8;
        if pcm_sample_bit_depth_chroma_minus1 + 1 > bit_depth_c {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PcmBitDepth_C must be less than or equal to BitDepth_C",
            ));
        }

        let log2_min_pcm_luma_coding_block_size_minus3 = bit_reader.read_exp_golomb()?;
        let log2_min_ipcm_cb_size_y = log2_min_pcm_luma_coding_block_size_minus3 + 3;
        range_check!(log2_min_ipcm_cb_size_y, min_cb_log2_size_y.min(5), ctb_log2_size_y.min(5))?;

        let log2_diff_max_min_pcm_luma_coding_block_size = bit_reader.read_exp_golomb()?;
        let log2_max_ipcm_cb_size_y = log2_diff_max_min_pcm_luma_coding_block_size + log2_min_ipcm_cb_size_y;
        if log2_max_ipcm_cb_size_y > ctb_log2_size_y.min(5) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Log2MaxIpcmCbSizeY must be less than or equal to Min(CtbLog2SizeY, 5)",
            ));
        }

        Ok(Self {
            pcm_sample_bit_depth_luma_minus1,
            pcm_sample_bit_depth_chroma_minus1,
            log2_min_pcm_luma_coding_block_size_minus3,
            log2_diff_max_min_pcm_luma_coding_block_size,
            pcm_loop_filter_disabled_flag: bit_reader.read_bit()?,
        })
    }

    /// Specifies the number of bits used to represent each of PCM sample values of the luma component.
    ///
    /// The value of `PcmBitDepthY` is less than or equal to the value of [`BitDepthY`](crate::SpsRbsp::bit_depth_y).
    ///
    /// `PcmBitDepthY = pcm_sample_bit_depth_luma_minus1 + 1` (7-25)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.1
    pub fn pcm_bit_depth_y(&self) -> u8 {
        self.pcm_sample_bit_depth_luma_minus1 + 1
    }

    /// Specifies the number of bits used to represent each of PCM sample values of the chroma components.
    ///
    /// The value of `PcmBitDepthC` is less than or equal to the value of [`BitDepthC`](crate::SpsRbsp::bit_depth_c).
    /// When [`ChromaArrayType`](crate::SpsRbsp::chroma_array_type) is equal to 0, decoders shall ignore its value.
    ///
    /// `PcmBitDepthC = pcm_sample_bit_depth_chroma_minus1 + 1` (7-26)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.1
    pub fn pcm_bit_depth_c(&self) -> u8 {
        self.pcm_sample_bit_depth_chroma_minus1 + 1
    }

    /// The value is range
    /// \[[`Min(MinCbLog2SizeY, 5)`](crate::SpsRbsp::min_cb_log2_size_y), [`Min(CtbLog2SizeY, 5)`](crate::SpsRbsp::ctb_log2_size_y)\].
    ///
    /// `Log2MinIpcmCbSizeY = log2_min_pcm_luma_coding_block_size_minus3 + 3`
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.1
    pub fn log2_min_ipcm_cb_size_y(&self) -> u64 {
        self.log2_min_pcm_luma_coding_block_size_minus3 + 3
    }

    /// The value is less than or equal to [`Min(CtbLog2SizeY, 5)`](crate::SpsRbsp::ctb_log2_size_y).
    ///
    /// `Log2MaxIpcmCbSizeY = log2_diff_max_min_pcm_luma_coding_block_size + Log2MinIpcmCbSizeY`
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.1
    pub fn log2_max_ipcm_cb_size_y(&self) -> u64 {
        self.log2_diff_max_min_pcm_luma_coding_block_size + self.log2_min_ipcm_cb_size_y()
    }
}
