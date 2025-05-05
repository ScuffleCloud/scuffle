use std::io;

use scuffle_bytes_util::{BitReader, BitWriter, range_check};
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt, size_of_exp_golomb, size_of_signed_exp_golomb};

/// The Sequence Parameter Set extension.
/// ISO/IEC-14496-10-2022 - 7.3.2
#[derive(Debug, Clone, PartialEq)]
pub struct SpsExtended {
    /// The `chroma_format_idc` as a u8. This is the chroma sampling relative
    /// to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 3\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `3` which is encoded as `0 0100`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_format_idc: u8,

    /// The `separate_colour_plane_flag` is a single bit.
    ///
    /// 0 means the the color components aren't coded separately and `ChromaArrayType` is set to `chroma_format_idc`.
    ///
    /// 1 means the 3 color components of the 4:4:4 chroma format are coded separately and
    /// `ChromaArrayType` is set to 0.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub separate_color_plane_flag: bool,

    /// The `bit_depth_luma_minus8` as a u8. This is the chroma sampling relative
    /// to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 6\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `6` which is encoded as `0 0111`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub bit_depth_luma_minus8: u8,

    /// The `bit_depth_chroma_minus8` as a u8. This is the chroma sampling
    /// relative to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 6\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `6` which is encoded as `0 0111`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub bit_depth_chroma_minus8: u8,

    /// The `qpprime_y_zero_transform_bypass_flag` is a single bit.
    ///
    /// 0 means the transform coefficient decoding and picture construction processes wont
    /// use the transform bypass operation.
    ///
    /// 1 means that when QP'_Y is 0 then a transform bypass operation for the transform
    /// coefficient decoding and picture construction processes will be applied before
    /// the deblocking filter process from subclause 8.5.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub qpprime_y_zero_transform_bypass_flag: bool,

    /// The `scaling_matrix`. If the length is nonzero, then
    /// `seq_scaling_matrix_present_flag` must have been set.
    pub scaling_matrix: Vec<Vec<i64>>,
}

impl Default for SpsExtended {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SpsExtended {
    // default values defined in 7.4.2.1.1
    const DEFAULT: SpsExtended = SpsExtended {
        chroma_format_idc: 1,
        separate_color_plane_flag: false,
        bit_depth_luma_minus8: 0,
        bit_depth_chroma_minus8: 0,
        qpprime_y_zero_transform_bypass_flag: false,
        scaling_matrix: vec![],
    };

    /// Parses an extended SPS from a bitstream.
    /// Returns an `SpsExtended` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let chroma_format_idc = reader.read_exp_golomb()?;
        range_check!(chroma_format_idc, 0, 3)?;
        let chroma_format_idc = chroma_format_idc as u8;
        // Defaults to false: ISO/IEC-14496-10-2022 - 7.4.2.1.1
        let mut separate_color_plane_flag = false;
        if chroma_format_idc == 3 {
            separate_color_plane_flag = reader.read_bit()?;
        }

        let bit_depth_luma_minus8 = reader.read_exp_golomb()?;
        range_check!(bit_depth_luma_minus8, 0, 6)?;
        let bit_depth_luma_minus8 = bit_depth_luma_minus8 as u8;

        let bit_depth_chroma_minus8 = reader.read_exp_golomb()?;
        range_check!(bit_depth_chroma_minus8, 0, 6)?;
        let bit_depth_chroma_minus8 = bit_depth_chroma_minus8 as u8;

        let qpprime_y_zero_transform_bypass_flag = reader.read_bit()?;
        let seq_scaling_matrix_present_flag = reader.read_bit()?;
        let mut scaling_matrix: Vec<Vec<i64>> = vec![];

        if seq_scaling_matrix_present_flag {
            // We need to read the scaling matrices here, but we don't need them
            // for decoding, so we just skip them.
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for i in 0..count {
                let bit = reader.read_bit()?;
                scaling_matrix.push(vec![]);
                if bit {
                    let size = if i < 6 { 16 } else { 64 };
                    let mut next_scale = 8;
                    for _ in 0..size {
                        let delta_scale = reader.read_signed_exp_golomb()?;
                        scaling_matrix[i].push(delta_scale);
                        next_scale = (next_scale + delta_scale + 256) % 256;
                        if next_scale == 0 {
                            break;
                        }
                    }
                }
            }
        }

        Ok(SpsExtended {
            chroma_format_idc,
            separate_color_plane_flag,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag,
            scaling_matrix,
        })
    }

    /// Builds the SPSExtended struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_exp_golomb(self.chroma_format_idc as u64)?;

        if self.chroma_format_idc == 3 {
            writer.write_bit(self.separate_color_plane_flag)?;
        }

        writer.write_exp_golomb(self.bit_depth_luma_minus8 as u64)?;
        writer.write_exp_golomb(self.bit_depth_chroma_minus8 as u64)?;
        writer.write_bit(self.qpprime_y_zero_transform_bypass_flag)?;

        writer.write_bit(!self.scaling_matrix.is_empty())?;

        for vec in &self.scaling_matrix {
            writer.write_bit(!vec.is_empty())?;

            for expg in vec {
                writer.write_signed_exp_golomb(*expg)?;
            }
        }
        Ok(())
    }

    /// Returns the total bits of the SpsExtended struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        size_of_exp_golomb(self.chroma_format_idc as u64) +
        (self.chroma_format_idc == 3) as u64 +
        size_of_exp_golomb(self.bit_depth_luma_minus8 as u64) +
        size_of_exp_golomb(self.bit_depth_chroma_minus8 as u64) +
        1 + // qpprime_y_zero_transform_bypass_flag
        1 + // scaling_matrix.is_empty()
        // scaling matrix
        self.scaling_matrix.len() as u64 +
        self.scaling_matrix.iter().flat_map(|inner| inner.iter()).map(|&x| size_of_signed_exp_golomb(x)).sum::<u64>()
    }

    /// Returns the total bytes of the SpsExtended struct.
    ///
    /// Note that this calls [`SpsExtended::bitsize()`] and calculates the number of bytes
    /// including any necessary padding such that the bitstream is byte aligned.
    pub fn bytesize(&self) -> u64 {
        self.bitsize().div_ceil(8)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::{BitReader, BitWriter};
    use scuffle_expgolomb::BitWriterExpGolombExt;

    use crate::sps::SpsExtended;

    #[test]
    fn test_build_size_sps_ext_chroma_not_3_and_no_scaling_matrix_and_size() {
        // create data bitstream for sps_ext
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_exp_golomb(1).unwrap();
        writer.write_exp_golomb(2).unwrap();
        writer.write_exp_golomb(4).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let sps_ext = SpsExtended::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        sps_ext.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_sps_ext = SpsExtended::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_sps_ext.bitsize(), sps_ext.bitsize());
        assert_eq!(rebuilt_sps_ext.bytesize(), sps_ext.bytesize());
    }

    #[test]
    fn test_build_size_sps_ext_chroma_3_and_scaling_matrix() {
        // create bitstream for sps_ext
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        // set chroma_format_idc = 3
        writer.write_exp_golomb(3).unwrap();
        // separate_color_plane_flag since chroma_format_idc = 3
        writer.write_bit(true).unwrap();
        writer.write_exp_golomb(2).unwrap();
        writer.write_exp_golomb(4).unwrap();
        writer.write_bit(true).unwrap();
        // set seq_scaling_matrix_present_flag
        writer.write_bit(true).unwrap();

        // scaling matrix loop happens 12 times since chroma_format_idc is 3
        // loop 1 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64
        // next_scale starts as 8, we add 1 so it's 9
        writer.write_signed_exp_golomb(1).unwrap();
        // subloop 2 of 64
        // next_scale is 9, we add 2 so it's 11
        writer.write_signed_exp_golomb(2).unwrap();
        // subloop 3 of 64
        // next_scale is 11, we add 3 so it's 14
        writer.write_signed_exp_golomb(3).unwrap();
        // subloop 4 of 64: we want to break out of the loop now
        // next_scale is 14, we subtract 14 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-14).unwrap();

        // loop 2 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64
        // next_scale starts at 8, we add 3 so it's 11
        writer.write_signed_exp_golomb(3).unwrap();
        // subloop 2 of 64
        // next_scale is 11, we add 5 so it's 16
        writer.write_signed_exp_golomb(5).unwrap();
        // subloop 3 of 64; we want to break out of the loop now
        // next_scale is 16, we subtract 16 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-16).unwrap();

        // loop 3 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64
        // next_scale starts at 8, we add 1 so it's 9
        writer.write_signed_exp_golomb(1).unwrap();
        // subloop 2 of 64; we want to break out of the loop now
        // next_scale is 9, we subtract 9 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-9).unwrap();

        // loop 4 of 12
        writer.write_bit(true).unwrap();
        // subloop 1 of 64; we want to break out of the loop now
        // next scale starts at 8, we subtract 8 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-8).unwrap();

        // loop 5 thru 11: try writing nothing
        writer.write_bits(0, 7).unwrap();

        // loop 12 of 12: try writing something
        writer.write_bit(true).unwrap();
        // subloop 1 of 64; we want to break out of the loop now
        // next scale starts at 8, we subtract 8 so it's 0, triggering a break
        writer.write_signed_exp_golomb(-8).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let sps_ext = SpsExtended::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        sps_ext.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_sps_ext = SpsExtended::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_sps_ext.bitsize(), sps_ext.bitsize());
        assert_eq!(rebuilt_sps_ext.bytesize(), sps_ext.bytesize());
    }
}
