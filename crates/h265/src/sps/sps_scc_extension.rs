use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::range_check::range_check;

#[derive(Debug, Clone, PartialEq)]
pub struct SpsSccExtension {
    pub sps_curr_pic_ref_enabled_flag: bool,
    pub palette_mode: Option<SpsSccExtensionPaletteMode>,
    pub motion_vector_resolution_control_idc: u8,
    pub intra_boundary_filtering_disabled_flag: bool,
}

impl SpsSccExtension {
    pub fn parse<R: io::Read>(
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

                let mut initializers = vec![vec![0; sps_num_palette_predictor_initializers_minus1 as usize]; num_comps];
                for (comp, initializer) in initializers.iter_mut().enumerate().take(num_comps) {
                    for sps_palette_predictor_initializer in initializer
                        .iter_mut()
                        .take(sps_num_palette_predictor_initializers_minus1 as usize)
                    {
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

#[derive(Debug, Clone, PartialEq)]
pub struct SpsSccExtensionPaletteMode {
    pub palette_max_size: u64,
    pub delta_palette_max_predictor_size: u64,
    pub sps_palette_predictor_initializers: Option<Vec<Vec<u64>>>,
}

impl SpsSccExtensionPaletteMode {
    pub fn palette_max_predictor_size(&self) -> u64 {
        self.palette_max_size + self.delta_palette_max_predictor_size
    }
}
