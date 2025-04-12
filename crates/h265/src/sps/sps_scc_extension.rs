use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

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
        chroma_format_idc: u64,
        bit_depth_y: u64,
        bit_depth_c: u64,
    ) -> io::Result<Self> {
        let sps_curr_pic_ref_enabled_flag = bit_reader.read_bit()?;

        let mut palette_mode = None;
        let palette_mode_enabled_flag = bit_reader.read_bit()?;
        if palette_mode_enabled_flag {
            let palette_max_size = bit_reader.read_exp_golomb()?;
            let delta_palette_max_predictor_size = bit_reader.read_exp_golomb()?;

            let mut sps_palette_predictor_initializers = None;
            let sps_palette_predictor_initializers_present_flag = bit_reader.read_bit()?;
            if sps_palette_predictor_initializers_present_flag {
                let sps_num_palette_predictor_initializers_minus1 = bit_reader.read_exp_golomb()?;

                let num_comps = if chroma_format_idc == 0 { 1 } else { 3 };

                let mut initializers = vec![vec![0; sps_num_palette_predictor_initializers_minus1 as usize]; num_comps];
                for (comp, initializer) in initializers.iter_mut().enumerate().take(num_comps) {
                    for sps_palette_predictor_initializer in initializer
                        .iter_mut()
                        .take(sps_num_palette_predictor_initializers_minus1 as usize)
                    {
                        let bit_depth = if comp == 0 { bit_depth_y } else { bit_depth_c }
                            .try_into()
                            .unwrap_or(u8::MAX);
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
