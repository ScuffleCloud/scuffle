use std::io;

use scuffle_bytes_util::BitReader;

#[derive(Debug, Clone, PartialEq)]
pub struct SpsMultilayerExtension {
    pub inter_view_mv_vert_constraint_flag: bool,
}

impl SpsMultilayerExtension {
    pub fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        Ok(Self {
            inter_view_mv_vert_constraint_flag: bit_reader.read_bit()?,
        })
    }
}
