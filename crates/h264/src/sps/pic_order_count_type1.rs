use std::io;

use scuffle_bytes_util::{BitReader, BitWriter, range_check};
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt, size_of_exp_golomb, size_of_signed_exp_golomb};

/// `PicOrderCountType1` contains the fields that are set when `pic_order_cnt_type == 1`.
///
/// This contains the following fields: `delta_pic_order_always_zero_flag`,
/// `offset_for_non_ref_pic`, `offset_for_top_to_bottom_field`, and
/// `offset_for_ref_frame`.
#[derive(Debug, Clone, PartialEq)]
pub struct PicOrderCountType1 {
    /// The `delta_pic_order_always_zero_flag` is a single bit.
    ///
    /// 0 means the `delta_pic_order_cnt[0]` is in the slice headers and `delta_pic_order_cnt[1]`
    /// might not be in the slice headers.
    ///
    /// 1 means the `delta_pic_order_cnt[0]` and `delta_pic_order_cnt[1]` are NOT in the slice headers
    /// and will be set to 0 by default.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub delta_pic_order_always_zero_flag: bool,

    /// The `offset_for_non_ref_pic` is used to calculate the pic order count for a non-reference picture
    /// from subclause 8.2.1.
    ///
    /// The value of this ranges from \[-2^(31) + 1, 2^(31) - 1\].
    ///
    /// This is a variable number of bits as it is encoded by a SIGNED exp golomb.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub offset_for_non_ref_pic: i64,

    /// The `offset_for_top_to_bottom_field` is used to calculate the pic order count of a bottom field from
    /// subclause 8.2.1.
    ///
    /// The value of this ranges from \[-2^(31) + 1, 2^(31) - 1\].
    ///
    /// This is a variable number of bits as it is encoded by a SIGNED exp golomb.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub offset_for_top_to_bottom_field: i64,

    /// The `num_ref_frames_in_pic_order_cnt_cycle` is used in the decoding process for the picture order
    /// count in 8.2.1.
    ///
    /// The value of this ranges from \[0, 255\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `255` which is encoded as `0 0000 0001 0000 0000`, which is 17 bits.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub num_ref_frames_in_pic_order_cnt_cycle: u64,

    /// The `offset_for_ref_frame` is a vec where each value used in decoding the picture order count
    /// from subclause 8.2.1.
    ///
    /// When `pic_order_cnt_type == 1`, `ExpectedDeltaPerPicOrderCntCycle` can be derived by:
    /// ```python
    /// ExpectedDeltaPerPicOrderCntCycle = sum(offset_for_ref_frame)
    /// ```
    ///
    /// The value of this ranges from \[-2^(31) + 1, 2^(31) - 1\].
    ///
    /// This is a variable number of bits as it is encoded by a SIGNED exp golomb.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub offset_for_ref_frame: Vec<i64>,
}

impl PicOrderCountType1 {
    /// Parses the fields defined when the `pic_order_count_type == 1` from a bitstream.
    /// Returns a `PicOrderCountType1` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let delta_pic_order_always_zero_flag = reader.read_bit()?;
        let offset_for_non_ref_pic = reader.read_signed_exp_golomb()?;
        range_check!(offset_for_non_ref_pic, i32::MIN as i64 + 1, i32::MAX as i64)?;
        let offset_for_top_to_bottom_field = reader.read_signed_exp_golomb()?;
        range_check!(offset_for_top_to_bottom_field, i32::MIN as i64 + 1, i32::MAX as i64)?;
        let num_ref_frames_in_pic_order_cnt_cycle = reader.read_exp_golomb()?;
        range_check!(num_ref_frames_in_pic_order_cnt_cycle, 0, 255)?;

        let mut offset_for_ref_frame = vec![];
        for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
            offset_for_ref_frame.push(reader.read_signed_exp_golomb()?);
            range_check!(
                offset_for_ref_frame.last().copied().unwrap(),
                i32::MIN as i64 + 1,
                i32::MAX as i64
            )?;
        }

        Ok(PicOrderCountType1 {
            delta_pic_order_always_zero_flag,
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frame,
        })
    }

    /// Builds the PicOrderCountType1 struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_bit(self.delta_pic_order_always_zero_flag)?;
        writer.write_signed_exp_golomb(self.offset_for_non_ref_pic)?;
        writer.write_signed_exp_golomb(self.offset_for_top_to_bottom_field)?;
        writer.write_exp_golomb(self.num_ref_frames_in_pic_order_cnt_cycle)?;

        for num in &self.offset_for_ref_frame {
            writer.write_signed_exp_golomb(*num)?;
        }
        Ok(())
    }

    /// Returns the total bits of the PicOrderCountType1 struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        1 + // delta_pic_order_always_zero_flag
        size_of_signed_exp_golomb(self.offset_for_non_ref_pic) +
        size_of_signed_exp_golomb(self.offset_for_top_to_bottom_field) +
        size_of_exp_golomb(self.num_ref_frames_in_pic_order_cnt_cycle) +
        self.offset_for_ref_frame.iter().map(|x| size_of_signed_exp_golomb(*x)).sum::<u64>()
    }

    /// Returns the total bytes of the PicOrderCountType1 struct.
    ///
    /// Note that this calls [`PicOrderCountType1::bitsize()`] and calculates the number of bytes
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

    use crate::sps::PicOrderCountType1;

    #[test]
    fn test_build_size_pic_order() {
        // create bitstream for pic_order_count_type1
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_bit(true).unwrap();
        writer.write_signed_exp_golomb(3).unwrap();
        writer.write_signed_exp_golomb(7).unwrap();
        writer.write_exp_golomb(2).unwrap();

        // loop
        writer.write_signed_exp_golomb(4).unwrap();
        writer.write_signed_exp_golomb(8).unwrap();

        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let pic_order_count_type1 = PicOrderCountType1::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        pic_order_count_type1.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_pic_order_count_type1 = PicOrderCountType1::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_pic_order_count_type1.bitsize(), pic_order_count_type1.bitsize());
        assert_eq!(rebuilt_pic_order_count_type1.bytesize(), pic_order_count_type1.bytesize());
    }
}
