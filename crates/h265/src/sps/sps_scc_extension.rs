use std::io;

use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Sequence parameter set screen content coding extension.
///
/// `sps_scc_extension()`
///
/// - ISO/IEC 23008-2 - 7.3.2.2.3
/// - ISO/IEC 23008-2 - 7.4.3.2.3
#[derive(Debug, Clone, PartialEq)]
pub struct SpsSccExtension {
    /// Equal to `true` specifies that a picture in the CVS may be included in a
    /// reference picture list of a slice of the picture itself.
    ///
    /// Equal to `false` specifies that a picture in the CVS is never included in a
    /// reference picture list of a slice of the picture itself.
    pub sps_curr_pic_ref_enabled_flag: bool,
    /// Palette mode information, if `palette_mode_enabled_flag` is `true`.
    pub palette_mode: Option<SpsSccExtensionPaletteMode>,
    /// Controls the presence and inference of the `use_integer_mv_flag`
    /// that specifies the resolution of motion vectors for inter prediction.
    ///
    /// The value is in range \[0, 2\].
    pub motion_vector_resolution_control_idc: u8,
    /// Equal to `true` specifies that the intra boundary filtering process is
    /// unconditionally disabled for intra prediction.
    ///
    /// Equal to `false` specifies that the intra boundary filtering process may be used.
    pub intra_boundary_filtering_disabled_flag: bool,
}

impl SpsSccExtension {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        chroma_format_idc: u8,
        bit_depth_y: u8,
        bit_depth_c: u8,
    ) -> io::Result<Self> {
        let sps_curr_pic_ref_enabled_flag = bit_reader.read_bit()?;

        let mut palette_mode = None;
        let palette_mode_enabled_flag = bit_reader.read_bit()?;
        if palette_mode_enabled_flag {
            let palette_max_size = bit_reader.read_exp_golomb()?;
            let delta_palette_max_predictor_size = bit_reader.read_exp_golomb()?;

            if palette_max_size == 0 && delta_palette_max_predictor_size != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "delta_palette_max_predictor_size must be 0 when palette_max_size is 0",
                ));
            }

            let sps_palette_predictor_initializers_present_flag = bit_reader.read_bit()?;
            if palette_max_size == 0 && !sps_palette_predictor_initializers_present_flag {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "sps_palette_predictor_initializers_present_flag must be 0 when palette_max_size is 0",
                ));
            }

            let mut sps_palette_predictor_initializers = None;
            if sps_palette_predictor_initializers_present_flag {
                let sps_num_palette_predictor_initializers_minus1 = bit_reader.read_exp_golomb()?;

                if sps_num_palette_predictor_initializers_minus1 >= palette_max_size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "sps_num_palette_predictor_initializers_minus1 + 1 must be less than or equal to palette_max_size",
                    ));
                }

                let num_comps = if chroma_format_idc == 0 { 1 } else { 3 };

                let mut initializers = vec![vec![0; sps_num_palette_predictor_initializers_minus1 as usize + 1]; num_comps];
                for (comp, initializer) in initializers.iter_mut().enumerate().take(num_comps) {
                    for sps_palette_predictor_initializer in initializer.iter_mut() {
                        let bit_depth = if comp == 0 { bit_depth_y } else { bit_depth_c };
                        *sps_palette_predictor_initializer = bit_reader.read_bits(bit_depth)?;
                    }
                }

                sps_palette_predictor_initializers = Some(initializers);
            }

            palette_mode = Some(SpsSccExtensionPaletteMode {
                palette_max_size,
                delta_palette_max_predictor_size,
                sps_palette_predictor_initializers,
            });
        }

        let motion_vector_resolution_control_idc = bit_reader.read_bits(2)? as u8;
        range_check!(motion_vector_resolution_control_idc, 0, 2)?; // 3 is reserved

        let intra_boundary_filtering_disabled_flag = bit_reader.read_bit()?;

        Ok(Self {
            sps_curr_pic_ref_enabled_flag,
            palette_mode,
            motion_vector_resolution_control_idc,
            intra_boundary_filtering_disabled_flag,
        })
    }
}

/// Directly part of [`SpsSccExtension`].
#[derive(Debug, Clone, PartialEq)]
pub struct SpsSccExtensionPaletteMode {
    /// Specifies the maximum allowed palette size.
    pub palette_max_size: u64,
    /// Specifies the difference between the maximum allowed palette predictor size and the maximum allowed palette size.
    ///
    /// Defines [`PaletteMaxPredictorSize`](SpsSccExtensionPaletteMode::palette_max_predictor_size).
    pub delta_palette_max_predictor_size: u64,
    /// `sps_palette_predictor_initializer[comp][i]`, if `sps_palette_predictor_initializers_present_flag` is `true`.
    ///
    /// Specifies the value of the `comp`-th component of the `i`-th
    /// palette entry in the SPS that is used to initialize the array PredictorPaletteEntries.
    ///
    /// The value of `sps_palette_predictor_initializer[0][i]` is in range \[0, `(1 << BitDepthY) − 1`\].
    /// See [`BitDepthY`](crate::SpsRbsp::bit_depth_y).
    ///
    /// The values of `sps_palette_predictor_initializer[1][i]` and `sps_palette_predictor_initializer[2][i]`
    /// is in range \[0, `(1 << BitDepthC) − 1`\].
    /// See [`BitDepthC`](crate::SpsRbsp::bit_depth_c).
    pub sps_palette_predictor_initializers: Option<Vec<Vec<u64>>>,
}

impl SpsSccExtensionPaletteMode {
    /// `PaletteMaxPredictorSize = palette_max_size + delta_palette_max_predictor_size` (7-35)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.3
    pub fn palette_max_predictor_size(&self) -> u64 {
        self.palette_max_size + self.delta_palette_max_predictor_size
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use byteorder::WriteBytesExt;
    use scuffle_bytes_util::BitWriter;
    use scuffle_expgolomb::BitWriterExpGolombExt;

    #[test]
    fn test_parse() {
        let mut data = Vec::new();
        let mut bit_writer = BitWriter::new(&mut data);

        bit_writer.write_bit(true).unwrap(); // sps_curr_pic_ref_enabled_flag
        bit_writer.write_bit(true).unwrap(); // palette_mode_enabled_flag

        bit_writer.write_exp_golomb(5).unwrap(); // palette_max_size
        bit_writer.write_exp_golomb(2).unwrap(); // delta_palette_max_predictor_size
        bit_writer.write_bit(true).unwrap(); // sps_palette_predictor_initializers_present_flag

        bit_writer.write_exp_golomb(1).unwrap(); // sps_num_palette_predictor_initializers_minus1

        bit_writer.write_u8(1).unwrap(); // sps_palette_predictor_initializer[0][0]
        bit_writer.write_u8(2).unwrap(); // sps_palette_predictor_initializer[0][1]
        bit_writer.write_u8(3).unwrap(); // sps_palette_predictor_initializer[1][0]
        bit_writer.write_u8(4).unwrap(); // sps_palette_predictor_initializer[1][1]
        bit_writer.write_u8(5).unwrap(); // sps_palette_predictor_initializer[2][0]
        bit_writer.write_u8(6).unwrap(); // sps_palette_predictor_initializer[2][1]

        bit_writer.write_bits(0, 2).unwrap(); // motion_vector_resolution_control_idc
        bit_writer.write_bit(false).unwrap(); // intra_boundary_filtering_disabled_flag

        bit_writer.write_bits(0, 8).unwrap(); // fill the last byte

        let scc_extension = super::SpsSccExtension::parse(
            &mut scuffle_bytes_util::BitReader::new(&data[..]),
            1, // chroma_format_idc
            8, // bit_depth_y
            8, // bit_depth_c
        )
        .unwrap();

        assert!(scc_extension.sps_curr_pic_ref_enabled_flag);

        assert!(scc_extension.palette_mode.is_some());
        let palette_mode = scc_extension.palette_mode.unwrap();
        assert_eq!(palette_mode.palette_max_size, 5);
        assert_eq!(palette_mode.delta_palette_max_predictor_size, 2);
        assert_eq!(palette_mode.palette_max_predictor_size(), 7);

        assert!(palette_mode.sps_palette_predictor_initializers.is_some());
        let initializers = palette_mode.sps_palette_predictor_initializers.unwrap();
        assert_eq!(initializers.len(), 3);
        assert_eq!(initializers[0].len(), 2);
        assert_eq!(initializers[0][0], 1);
        assert_eq!(initializers[0][1], 2);
        assert_eq!(initializers[1].len(), 2);
        assert_eq!(initializers[1][0], 3);
        assert_eq!(initializers[1][1], 4);
        assert_eq!(initializers[2].len(), 2);
        assert_eq!(initializers[2][0], 5);
        assert_eq!(initializers[2][1], 6);

        assert_eq!(scc_extension.motion_vector_resolution_control_idc, 0);
        assert!(!scc_extension.intra_boundary_filtering_disabled_flag);
    }
}
