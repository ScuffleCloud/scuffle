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

    pub fn coeff_min_y(&self, bit_depth_y: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_y + 6)
        } else {
            15
        };
        -(1 << n)
    }

    pub fn coeff_min_c(&self, bit_depth_c: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_c + 6)
        } else {
            15
        };
        -(1 << n)
    }

    pub fn coeff_max_y(&self, bit_depth_y: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_y + 6)
        } else {
            15
        };
        (1 << n) - 1
    }

    pub fn coeff_max_c(&self, bit_depth_c: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_c + 6)
        } else {
            15
        };
        (1 << n) - 1
    }

    pub fn wp_offset_bd_shift_y(&self, bit_depth_y: u8) -> i8 {
        if self.high_precision_offsets_enabled_flag {
            0
        } else {
            bit_depth_y as i8 - 8
        }
    }

    pub fn wp_offset_bd_shift_c(&self, bit_depth_c: u8) -> i8 {
        if self.high_precision_offsets_enabled_flag {
            0
        } else {
            bit_depth_c as i8 - 8
        }
    }

    pub fn wp_offset_half_range_y(&self, bit_depth_y: u8) -> i8 {
        let n = if self.high_precision_offsets_enabled_flag {
            bit_depth_y.saturating_sub(1)
        } else {
            7
        };
        1 << n
    }

    pub fn wp_offset_half_range_c(&self, bit_depth_c: u8) -> i8 {
        let n = if self.high_precision_offsets_enabled_flag {
            bit_depth_c.saturating_sub(1)
        } else {
            7
        };
        1 << n
    }
}
