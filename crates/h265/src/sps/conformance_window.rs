use std::io;

use scuffle_bytes_util::{BitReader, BitWriter};
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt};

/// `ConfWindowInfo` contains the frame cropping info.
///
/// This includes `conf_win_left_offset`, `conf_win_right_offset`, `conf_win_top_offset`,
/// and `conf_win_bottom_offset`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceWindow {
    /// The `conf_win_left_offset` is the the left crop offset which is used to compute the width:
    ///
    /// `width = pic_width_in_luma_samples - sub_width_c * (conf_win_left_offset + conf_win_right_offset)`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub conf_win_left_offset: u64,

    /// The `conf_win_right_offset` is the the right crop offset which is used to compute the width:
    ///
    /// `width = pic_width_in_luma_samples - sub_width_c * (conf_win_left_offset + conf_win_right_offset)`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub conf_win_right_offset: u64,

    /// The `conf_win_top_offset` is the the top crop offset which is used to compute the height:
    ///
    /// `height = pic_height_in_luma_samples - sub_height_c * (conf_win_top_offset + conf_win_bottom_offset)`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub conf_win_top_offset: u64,

    /// The `conf_win_bottom_offset` is the the bottom crop offset which is used to compute the height:
    ///
    /// `height = pic_height_in_luma_samples - sub_height_c * (conf_win_top_offset + conf_win_bottom_offset)`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// ISO/IEC-23008-2-2020 - 7.4.3.2.1
    pub conf_win_bottom_offset: u64,
}

impl ConformanceWindow {
    /// Parses the fields defined when the `conformance_window_flag == 1` from a bitstream.
    /// Returns a `ConformanceWindow` struct.
    pub fn parse<R: io::Read>(reader: &mut BitReader<R>) -> io::Result<Self> {
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

    /// Builds the ConformanceWindow struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<W: io::Write>(&self, writer: &mut BitWriter<W>) -> io::Result<()> {
        writer.write_exp_golomb(self.conf_win_left_offset)?;
        writer.write_exp_golomb(self.conf_win_right_offset)?;
        writer.write_exp_golomb(self.conf_win_top_offset)?;
        writer.write_exp_golomb(self.conf_win_bottom_offset)?;
        Ok(())
    }

    // /// Returns the total bits of the ConformanceWindow struct.
    // ///
    // /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    // ///
    // pub fn bitsize(&self) -> u64 {
    //     size_of_exp_golomb(self.conf_win_left_offset)
    //         + size_of_exp_golomb(self.conf_win_right_offset)
    //         + size_of_exp_golomb(self.conf_win_top_offset)
    //         + size_of_exp_golomb(self.conf_win_bottom_offset)
    // }

    // /// Returns the total bytes of the ConformanceWindow struct.
    // ///
    // /// Note that this calls [`ConformanceWindow::bitsize()`] and calculates the number of bytes
    // /// including any necessary padding such that the bitstream is byte aligned.
    // ///
    // pub fn bytesize(&self) -> u64 {
    //     self.bitsize().div_ceil(8)
    // }
}
