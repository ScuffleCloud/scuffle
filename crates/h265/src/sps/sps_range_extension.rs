use std::io;

use scuffle_bytes_util::BitReader;

/// Sequence parameter set range extension.
///
/// `sps_range_extension()`
///
/// - ISO/IEC 23008-2 - 7.3.2.2.2
/// - ISO/IEC 23008-2 - 7.4.3.2.2
#[derive(Debug, Clone, PartialEq)]
pub struct SpsRangeExtension {
    /// Equal to `true` specifies that a rotation is applied to the residual data
    /// block for intra 4x4 blocks coded using a transform skip operation.
    ///
    /// Equal to `false` specifies that this rotation is not applied.
    pub transform_skip_rotation_enabled_flag: bool,
    /// Equal to `true` specifies that a particular context is used for the
    /// parsing of the `sig_coeff_flag` for transform blocks with a skipped transform.
    ///
    /// Equal to `false` specifies that the presence or absence of transform
    /// skipping or a transform bypass for transform blocks is not used in the context selection for this flag.
    pub transform_skip_context_enabled_flag: bool,
    /// Equal to `true` specifies that the residual modification process for blocks using
    /// a transform bypass may be used for intra blocks in the CVS.
    ///
    /// Equal to `false` specifies that the residual modification process is not used for intra blocks in the CVS.
    pub implicit_rdpcm_enabled_flag: bool,
    /// Equal to `true` specifies that the residual modification process for blocks using
    /// a transform bypass may be used for inter blocks in the CVS.
    ///
    /// Equal to `false` specifies that the residual modification process is not used for inter blocks in the CVS.
    pub explicit_rdpcm_enabled_flag: bool,
    /// Equal to `true` specifies that an extended dynamic range is used for
    /// transform coefficients and transform processing.
    ///
    /// Equal to `false` specifies that the extended dynamic range is not used.
    ///
    /// Defines [`CoeffMinY`](SpsRangeExtension::coeff_min_y), [`CoeffMinC`](SpsRangeExtension::coeff_min_c),
    /// [`CoeffMaxY`](SpsRangeExtension::coeff_max_y), and [`CoeffMaxC`](SpsRangeExtension::coeff_max_c).
    pub extended_precision_processing_flag: bool,
    /// Equal to `true` specifies that the filtering process of neighbouring samples is
    /// unconditionally disabled for intra prediction.
    ///
    /// Equal to `false` specifies that the filtering process of neighbouring samples is not disabled.
    pub intra_smoothing_disabled_flag: bool,
    /// Equal to `true` specifies that weighted prediction offset values are
    /// signalled using a bit-depth-dependent precision.
    ///
    /// Equal to `false` specifies that weighted prediction offset values are signalled with
    /// a precision equivalent to eight bit processing.
    ///
    /// Defines [`WpOffsetBdShiftY`](SpsRangeExtension::wp_offset_bd_shift_y),
    /// [`WpOffsetBdShiftC`](SpsRangeExtension::wp_offset_bd_shift_c),
    /// [`WpOffsetHalfRangeY`](SpsRangeExtension::wp_offset_half_range_y), and
    /// [`WpOffsetHalfRangeC`](SpsRangeExtension::wp_offset_half_range_c).
    pub high_precision_offsets_enabled_flag: bool,
    /// Equal to `true` specifies that the Rice parameter derivation for the
    /// binarization of `coeff_abs_level_remaining[]` is initialized at the start of each sub-block using mode
    /// dependent statistics accumulated from previous sub-blocks.
    ///
    /// Equal to `false` specifies that no previous sub-block state is used in Rice parameter derivation.
    pub persistent_rice_adaptation_enabled_flag: bool,
    /// Equal to `true` specifies that a CABAC alignment process is used
    /// prior to bypass decoding of the syntax elements `coeff_sign_flag[]` and `coeff_abs_level_remaining[]`.
    ///
    /// Equal to `false` specifies that no CABAC alignment process is used prior to bypass decoding.
    pub cabac_bypass_alignment_enabled_flag: bool,
}

impl SpsRangeExtension {
    pub(crate) fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
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

    /// `CoeffMinY = −(1 << (extended_precision_processing_flag ? Max(15, BitDepthY + 6) : 15))` (7-27)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn coeff_min_y(&self, bit_depth_y: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_y + 6)
        } else {
            15
        };
        -(1 << n)
    }

    /// `CoeffMinC = −(1 << (extended_precision_processing_flag ? Max(15, BitDepthC + 6) : 15))` (7-28)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn coeff_min_c(&self, bit_depth_c: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_c + 6)
        } else {
            15
        };
        -(1 << n)
    }

    /// `CoeffMaxY = (1 << (extended_precision_processing_flag ? Max(15, BitDepthY + 6) : 15)) - 1` (7-29)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn coeff_max_y(&self, bit_depth_y: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_y + 6)
        } else {
            15
        };
        (1 << n) - 1
    }

    /// `CoeffMaxC = (1 << (extended_precision_processing_flag ? Max(15, BitDepthC + 6) : 15)) − 1` (7-30)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn coeff_max_c(&self, bit_depth_c: u8) -> i64 {
        let n = if self.extended_precision_processing_flag {
            15.max(bit_depth_c + 6)
        } else {
            15
        };
        (1 << n) - 1
    }

    /// `WpOffsetBdShiftY = high_precision_offsets_enabled_flag ? 0 : (BitDepthY − 8)` (7-31)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn wp_offset_bd_shift_y(&self, bit_depth_y: u8) -> i8 {
        if self.high_precision_offsets_enabled_flag {
            0
        } else {
            bit_depth_y as i8 - 8
        }
    }

    /// `WpOffsetBdShiftC = high_precision_offsets_enabled_flag ? 0 : (BitDepthC − 8)` (7-32)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn wp_offset_bd_shift_c(&self, bit_depth_c: u8) -> i8 {
        if self.high_precision_offsets_enabled_flag {
            0
        } else {
            bit_depth_c as i8 - 8
        }
    }

    /// `WpOffsetHalfRangeY = 1 << (high_precision_offsets_enabled_flag ? (BitDepthY − 1) : 7)` (7-33)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn wp_offset_half_range_y(&self, bit_depth_y: u8) -> i8 {
        let n = if self.high_precision_offsets_enabled_flag {
            bit_depth_y.saturating_sub(1)
        } else {
            7
        };
        1 << n
    }

    /// `WpOffsetHalfRangeC = 1 << (high_precision_offsets_enabled_flag ? (BitDepthC − 1) : 7)` (7-34)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2.2
    pub fn wp_offset_half_range_c(&self, bit_depth_c: u8) -> i8 {
        let n = if self.high_precision_offsets_enabled_flag {
            bit_depth_c.saturating_sub(1)
        } else {
            7
        };
        1 << n
    }
}
