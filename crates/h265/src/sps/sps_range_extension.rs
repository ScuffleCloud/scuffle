use std::io;

use scuffle_bytes_util::BitReader;

#[derive(Debug, Clone, PartialEq)]
pub struct SpsRangeExtension {
    pub transform_skip_rotation_enabled_flag: bool,
    pub transform_skip_context_enabled_flag: bool,
    pub implicit_rdpcm_enabled_flag: bool,
    pub explicit_rdpcm_enabled_flag: bool,
    pub extended_precision_processing_flag: bool,
    pub intra_smoothing_disabled_flag: bool,
    pub high_precision_offsets_enabled_flag: bool,
    pub persistent_rice_adaptation_enabled_flag: bool,
    pub cabac_bypass_alignment_enabled_flag: bool,
}

impl SpsRangeExtension {
    pub fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        Ok(Self {
            transform_skip_rotation_enabled_flag: bit_reader.read_bit()?,
            transform_skip_context_enabled_flag: bit_reader.read_bit()?,
            implicit_rdpcm_enabled_flag: bit_reader.read_bit()?,
            explicit_rdpcm_enabled_flag: bit_reader.read_bit()?,
            extended_precision_processing_flag: bit_reader.read_bit()?,
            intra_smoothing_disabled_flag: bit_reader.read_bit()?,
            high_precision_offsets_enabled_flag: bit_reader.read_bit()?,
            persistent_rice_adaptation_enabled_flag: bit_reader.read_bit()?,
            cabac_bypass_alignment_enabled_flag: bit_reader.read_bit()?,
        })
    }
}
