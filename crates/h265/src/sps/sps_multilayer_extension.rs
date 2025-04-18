use std::io;

use scuffle_bytes_util::BitReader;

/// Sequence parameter set multilayer extension.
///
/// `sps_multilayer_extension()`
///
/// - ISO/IEC 23008-2 - F.7.3.2.2.4
/// - ISO/IEC 23008-2 - F.7.4.3.2.4
#[derive(Debug, Clone, PartialEq)]
pub struct SpsMultilayerExtension {
    /// Equal to `true` indicates that vertical component of motion vectors
    /// used for inter-layer prediction are constrained in the layers for which this SPS RBSP is the active SPS
    /// RBSP. When this value is equal to `true`, the vertical component of the motion
    /// vectors used for inter-layer prediction shall be less than or equal to 56 in units of luma samples.
    ///
    /// When this value is equal to `false`, no constraint on the vertical component of the motion
    /// vectors used for inter-layer prediction is signalled by this flag.
    pub inter_view_mv_vert_constraint_flag: bool,
}

impl SpsMultilayerExtension {
    pub(crate) fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        Ok(Self {
            inter_view_mv_vert_constraint_flag: bit_reader.read_bit()?,
        })
    }
}
