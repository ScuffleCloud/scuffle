use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Specifies the samples of the pictures in the CVS that are output from the decoding process, in terms of a rectangular
/// region specified in picture coordinates for output.
///
/// Directly part of [SPS RBSP](crate::SpsRbsp).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConformanceWindow {
    /// The the left crop offset which is used to compute the [`croppedWidth`](crate::SpsRbsp::cropped_width).
    pub conf_win_left_offset: u64,
    /// The the right crop offset which is used to compute the [`croppedWidth`](crate::SpsRbsp::cropped_width).
    pub conf_win_right_offset: u64,
    /// The top crop offset which is used to compute the [`croppedHeight`](crate::SpsRbsp::cropped_height).
    pub conf_win_top_offset: u64,
    /// The bottom crop offset which is used to compute the [`croppedHeight`](crate::SpsRbsp::cropped_height).
    pub conf_win_bottom_offset: u64,
}

impl ConformanceWindow {
    pub(crate) fn parse<R: io::Read>(reader: &mut BitReader<R>) -> io::Result<Self> {
        let conf_win_left_offset = reader.read_exp_golomb()?;
        let conf_win_right_offset = reader.read_exp_golomb()?;
        let conf_win_top_offset = reader.read_exp_golomb()?;
        let conf_win_bottom_offset = reader.read_exp_golomb()?;

        Ok(ConformanceWindow {
            conf_win_left_offset,
            conf_win_right_offset,
            conf_win_top_offset,
            conf_win_bottom_offset,
        })
    }
}
