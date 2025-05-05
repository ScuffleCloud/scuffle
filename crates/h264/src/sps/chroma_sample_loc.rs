use std::io;

use scuffle_bytes_util::{BitReader, BitWriter, range_check};
use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt, size_of_exp_golomb};

/// `ChromaSampleLoc` contains the fields that are set when `chroma_loc_info_present_flag == 1`,
///
/// This contains the following fields: `chroma_sample_loc_type_top_field` and `chroma_sample_loc_type_bottom_field`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSampleLoc {
    /// The `chroma_sample_loc_type_top_field` specifies the location of chroma samples.
    ///
    /// The value of this ranges from \[0, 5\]. By default, this value is set to 0.
    ///
    /// See ISO/IEC-14496-10-2022 - E.2.1 Figure E-1 for more info.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `5` which is encoded as `0 0110`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - E.2.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_sample_loc_type_top_field: u8,

    /// The `chroma_sample_loc_type_bottom_field`
    ///
    /// The value of this ranges from \[0, 5\]. By default, this value is set to 0.
    ///
    /// See ISO/IEC-14496-10-2022 - E.2.1 Figure E-1 for more info.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `5` which is encoded as `0 0110`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - E.2.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_sample_loc_type_bottom_field: u8,
}

impl ChromaSampleLoc {
    /// Parses the fields defined when the `chroma_loc_info_present_flag == 1` from a bitstream.
    /// Returns a `ChromaSampleLoc` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let chroma_sample_loc_type_top_field = reader.read_exp_golomb()?;
        range_check!(chroma_sample_loc_type_top_field, 0, 5)?;
        let chroma_sample_loc_type_top_field = chroma_sample_loc_type_top_field as u8;

        let chroma_sample_loc_type_bottom_field = reader.read_exp_golomb()?;
        range_check!(chroma_sample_loc_type_bottom_field, 0, 5)?;
        let chroma_sample_loc_type_bottom_field = chroma_sample_loc_type_bottom_field as u8;

        Ok(ChromaSampleLoc {
            chroma_sample_loc_type_top_field,
            chroma_sample_loc_type_bottom_field,
        })
    }

    /// Builds the ChromaSampleLoc struct into a byte stream.
    /// Returns a built byte stream.
    pub fn build<T: io::Write>(&self, writer: &mut BitWriter<T>) -> io::Result<()> {
        writer.write_exp_golomb(self.chroma_sample_loc_type_top_field as u64)?;
        writer.write_exp_golomb(self.chroma_sample_loc_type_bottom_field as u64)?;
        Ok(())
    }

    /// Returns the total bits of the ChromaSampleLoc struct.
    ///
    /// Note that this isn't the bytesize since aligning it may cause some values to be different.
    pub fn bitsize(&self) -> u64 {
        size_of_exp_golomb(self.chroma_sample_loc_type_top_field as u64)
            + size_of_exp_golomb(self.chroma_sample_loc_type_bottom_field as u64)
    }

    /// Returns the total bytes of the ChromaSampleLoc struct.
    ///
    /// Note that this calls [`ChromaSampleLoc::bitsize()`] and calculates the number of bytes
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

    use crate::sps::ChromaSampleLoc;

    #[test]
    fn test_build_size_chroma_sample() {
        // create bitstream for chroma_sample_loc
        let mut data = Vec::new();
        let mut writer = BitWriter::new(&mut data);

        writer.write_exp_golomb(1).unwrap();
        writer.write_exp_golomb(5).unwrap();
        writer.finish().unwrap();

        // parse bitstream
        let mut reader = BitReader::new_from_slice(&mut data);
        let chroma_sample_loc = ChromaSampleLoc::parse(&mut reader).unwrap();

        // create a writer for the builder
        let mut buf = Vec::new();
        let mut writer2 = BitWriter::new(&mut buf);

        // build from the example result
        chroma_sample_loc.build(&mut writer2).unwrap();
        writer2.finish().unwrap();

        assert_eq!(buf, data);

        // now we re-parse so we can compare the bit sizes.
        // create a reader for the parser
        let mut reader2 = BitReader::new_from_slice(buf);
        let rebuilt_chroma_sample_loc = ChromaSampleLoc::parse(&mut reader2).unwrap();

        // now we can check the size:
        assert_eq!(rebuilt_chroma_sample_loc.bitsize(), chroma_sample_loc.bitsize());
        assert_eq!(rebuilt_chroma_sample_loc.bytesize(), chroma_sample_loc.bytesize());
    }
}
