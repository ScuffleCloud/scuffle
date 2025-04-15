use std::io;
use std::num::NonZero;

use scuffle_bytes_util::BitReader;

use crate::NALUnitType;
use crate::range_check::range_check;

#[derive(Debug, Clone, PartialEq)]
pub struct NALUnitHeader {
    pub nal_unit_type: NALUnitType,

    /// Specifies the identifier of the layer to which a VCL NAL unit belongs or the identifier of a
    /// layer to which a non-VCL NAL unit applies.
    ///
    /// This value ranges from \[0, 63\], with 63 being reserved for future use.
    pub nuh_layer_id: u8,

    /// The `nuh_temporal_id_plus1` is 3 bits, where the value minus 1 is the temporal id for the NAL unit.
    ///
    /// This value cannot be 0.
    pub nuh_temporal_id_plus1: NonZero<u8>,
}

impl NALUnitHeader {
    pub fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        let forbidden_zero_bit = bit_reader.read_bit()?;
        if forbidden_zero_bit {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "forbidden_zero_bit is not zero"));
        }

        let nal_unit_type = NALUnitType::from(bit_reader.read_bits(6)? as u8);
        let nuh_layer_id = bit_reader.read_bits(6)? as u8;
        range_check!(nuh_layer_id, 0, 63)?;

        if nal_unit_type == NALUnitType::EobNut && nuh_layer_id != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "nuh_layer_id must be 0 when nal_unit_type is EOB_NUT",
            ));
        }

        let nuh_temporal_id_plus1 = bit_reader.read_bits(3)? as u8;
        let nuh_temporal_id_plus1 = NonZero::new(nuh_temporal_id_plus1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "nuh_temporal_id_plus1 cannot be 0"))?;

        if ((NALUnitType::BlaWLp..=NALUnitType::RsvIrapVcl23).contains(&nal_unit_type)
            || nal_unit_type == NALUnitType::VpsNut
            || nal_unit_type == NALUnitType::SpsNut
            || nal_unit_type == NALUnitType::EosNut
            || nal_unit_type == NALUnitType::EobNut)
            && nuh_temporal_id_plus1.get() != 1
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TemporalId must be 0 for nal_unit_type {:?}", nal_unit_type),
            ));
        }

        if (nal_unit_type == NALUnitType::TsaR || nal_unit_type == NALUnitType::TsaN) && nuh_temporal_id_plus1.get() == 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TemporalId must not be 0 for nal_unit_type {:?}", nal_unit_type),
            ));
        }

        if nuh_layer_id == 0
            && (nal_unit_type == NALUnitType::StsaR || nal_unit_type == NALUnitType::StsaN)
            && nuh_temporal_id_plus1.get() == 1
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "TemporalId must not be 0 for nuh_layer_id 0 and nal_unit_type {:?}",
                    nal_unit_type
                ),
            ));
        }

        Ok(Self {
            nal_unit_type,
            nuh_layer_id,
            nuh_temporal_id_plus1,
        })
    }

    /// Returns the temporal id of the NAL unit.
    ///
    /// Defined as `TemporalId` (7-1) in the spec.
    pub fn temporal_id(&self) -> u8 {
        self.nuh_temporal_id_plus1.get() - 1
    }
}
