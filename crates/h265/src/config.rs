use std::io::{
    Read, Write, {self},
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use scuffle_bytes_util::{BitReader, BitWriter};

#[derive(Debug, Clone, PartialEq)]
/// HEVC Decoder Configuration Record
/// ISO/IEC 14496-15:2022(E) - 8.3.2.1
pub struct HEVCDecoderConfigurationRecord {
    /// The `configuration_version` as a u8. Matches the field as defined in ISO/IEC 23008-2.
    pub configuration_version: u8,

    /// The `general_profile_space` as a u8. Matches the field as defined in ISO/IEC 23008-2.
    pub general_profile_space: u8,

    /// The `general_tier_flag` as a bool. Matches the field as defined in ISO/IEC 23008-2.
    pub general_tier_flag: bool,

    /// The `general_profile_idc` as a u8. Matches the field as defined in ISO/IEC 23008-2.
    pub general_profile_idc: u8,

    /// The `general_profile_compatibility_flags` as a u32. Matches the field as defined in ISO/IEC 23008-2.
    pub general_profile_compatibility_flags: u32,

    /// The `general_constraint_indicator_flags` as a u64. Matches the field as defined in ISO/IEC 23008-2.
    pub general_constraint_indicator_flags: u64,

    /// The `general_level_idc` as a u32. Matches the field as defined in ISO/IEC 23008-2.
    pub general_level_idc: u8,

    /// The `min_spatial_segmentation_idc` as a u16. Matches the field as defined in ISO/IEC 23008-2.
    pub min_spatial_segmentation_idc: u16,

    /// The `chroma_format_idc` as a u8. Matches the field as defined in ISO/IEC 23008-2.
    pub chroma_format_idc: u8,

    /// The `bit_depth_luma_minus8` as a u8. Matches the field as defined in ISO/IEC 23008-2.
    pub bit_depth_luma_minus8: u8,

    /// The `bit_depth_chroma_minus8` as a u8. Matches the field as defined in ISO/IEC 23008-2.
    pub bit_depth_chroma_minus8: u8,

    // TODO: nutype enum
    /// The `parallelism_type` as a u8.
    ///
    /// 0 means the stream supports mixed types of parallel decoding or otherwise.
    ///
    /// 1 means the stream supports slice based parallel decoding.
    ///
    /// 2 means the stream supports tile based parallel decoding.
    ///
    /// 3 means the stream supports entropy coding sync based parallel decoding.
    pub parallelism_type: u8,

    // definitely shouldn't be a u16. prolly f64
    /// The `avg_frame_rate` as a u16.
    pub avg_frame_rate: u16,

    /// The `constant_frame_rate` as a u8.
    ///
    /// 0 means the stream might have a constant frame rate.
    ///
    /// 1 means the stream has a constant framerate.
    ///
    /// 2 means the representation of each temporal layer in the stream has a constant framerate.
    pub constant_frame_rate: u8,

    // make this a nutype enum
    /// The `num_temporal_layers` as a u8. This is the count of tepmoral layers or `sub-layer`s as defined in ISO/IEC 23008-2.
    ///
    /// 0 means the stream might be temporally scalable.
    ///
    /// 1 means the stream is NOT temporally scalable.
    ///
    /// 2 or more means the stream is temporally scalable, and the count of temporal layers is equal to this value.
    pub num_temporal_layers: u8,

    /// The `temporal_id_nested` as a bool.
    ///
    /// 0 means means the opposite might not be true (refer to what 1 means for this flag).
    ///
    /// 1 means all the activated SPS have `sps_temporal_id_nesting_flag` (as defined in ISC/IEC 23008-2) set to 1 and that temporal sub-layer up-switching to a higehr temporal layer can be done at any sample.
    pub temporal_id_nested: bool,

    /// The `length_size_minus_one` is the u8 length of the NALUnitLength minus one.
    pub length_size_minus_one: u8,

    /// The `arrays` is a vec of NaluArray.
    /// Refer to the NaluArray struct in the NaluArray docs for more info.
    pub arrays: Vec<NaluArray>,
}

// turn into nutype enum
#[derive(Debug, Clone, PartialEq)]
/// Nalu Array Structure
/// ISO/IEC 14496-15:2022(E) - 8.3.2.1
pub struct NaluArray {
    /// The `array_completeness` is a flag set to 1 when all NAL units are in the array and none are in the stream. It is set to 0 if otherwise.
    pub array_completeness: bool,
    /// The `nal_unit_type` is the type of the NAL units in the `nalus` vec, as defined in ISO/IEC 23008-2.
    /// Refer to the `NaluType` enum for more info.
    pub nal_unit_type: NaluType,
    /// `nalus` is a vec of NAL units. Each of these will contain either a VPS, PPS, SPS, or an unknown u8 as specified in ISO/IEC 23008-2.
    /// Refer to the `NaluType` enum for more info.
    pub nalus: Vec<Bytes>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
/// The Nalu Type.
/// ISO/IEC 23008-2:2020(E) - 7.4.2.2 (Table 7-1)
pub enum NaluType {
    /// The Video Parameter Set.
    Vps,
    /// The Picture Parameter Set.
    Pps,
    /// The Sequence Parameter Set.
    Sps,
    /// An unknown u8. This is the default value if the NaluType is set to something other than VPS, PPS, or SPS.
    Unknown(u8),
}

impl From<u8> for NaluType {
    fn from(value: u8) -> Self {
        match value {
            32 => NaluType::Vps,
            33 => NaluType::Sps,
            34 => NaluType::Pps,
            _ => NaluType::Unknown(value),
        }
    }
}

impl From<NaluType> for u8 {
    fn from(value: NaluType) -> Self {
        match value {
            NaluType::Vps => 32,
            NaluType::Sps => 33,
            NaluType::Pps => 34,
            NaluType::Unknown(value) => value,
        }
    }
}

impl HEVCDecoderConfigurationRecord {
    /// Demuxes an HEVCDecoderConfigurationRecord from a byte stream.
    /// Returns a demuxed HEVCDecoderConfigurationRecord.
    pub fn demux(data: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let mut bit_reader = BitReader::new(data);

        let configuration_version = bit_reader.read_u8()?;
        let general_profile_space = bit_reader.read_bits(2)? as u8;
        let general_tier_flag = bit_reader.read_bit()?;
        let general_profile_idc = bit_reader.read_bits(5)? as u8;
        let general_profile_compatibility_flags = bit_reader.read_u32::<LittleEndian>()?;
        let general_constraint_indicator_flags = bit_reader.read_u48::<LittleEndian>()?;
        let general_level_idc = bit_reader.read_u8()?;

        bit_reader.seek_bits(4)?; // reserved_4bits
        let min_spatial_segmentation_idc = bit_reader.read_bits(12)? as u16;

        bit_reader.seek_bits(6)?; // reserved_6bits
        let parallelism_type = bit_reader.read_bits(2)? as u8;

        bit_reader.seek_bits(6)?; // reserved_6bits
        let chroma_format_idc = bit_reader.read_bits(2)? as u8;

        bit_reader.seek_bits(5)?; // reserved_5bits
        let bit_depth_luma_minus8 = bit_reader.read_bits(3)? as u8;

        bit_reader.seek_bits(5)?; // reserved_5bits
        let bit_depth_chroma_minus8 = bit_reader.read_bits(3)? as u8;

        let avg_frame_rate = bit_reader.read_u16::<BigEndian>()?;
        let constant_frame_rate = bit_reader.read_bits(2)? as u8;
        let num_temporal_layers = bit_reader.read_bits(3)? as u8;
        let temporal_id_nested = bit_reader.read_bit()?;
        let length_size_minus_one = bit_reader.read_bits(2)? as u8;

        let num_of_arrays = bit_reader.read_u8()?;

        let mut arrays = Vec::with_capacity(num_of_arrays as usize);

        for _ in 0..num_of_arrays {
            let array_completeness = bit_reader.read_bit()?;
            bit_reader.seek_bits(1)?; // reserved

            let nal_unit_type = bit_reader.read_bits(6)? as u8;

            let num_nalus = bit_reader.read_u16::<BigEndian>()?;

            let mut nalus = Vec::with_capacity(num_nalus as usize);

            for _ in 0..num_nalus {
                let nal_unit_length = bit_reader.read_u16::<BigEndian>()?;
                let mut data = vec![0; nal_unit_length as usize];
                bit_reader.read_exact(&mut data)?;
                nalus.push(data.into());
            }

            arrays.push(NaluArray {
                array_completeness,
                nal_unit_type: nal_unit_type.into(),
                nalus,
            });
        }

        Ok(HEVCDecoderConfigurationRecord {
            configuration_version,
            general_profile_space,
            general_tier_flag,
            general_profile_idc,
            general_profile_compatibility_flags,
            general_constraint_indicator_flags,
            general_level_idc,
            min_spatial_segmentation_idc,
            parallelism_type,
            chroma_format_idc,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            avg_frame_rate,
            constant_frame_rate,
            num_temporal_layers,
            temporal_id_nested,
            length_size_minus_one,
            arrays,
        })
    }

    /// Returns the total byte size of the HEVCDecoderConfigurationRecord.
    pub fn size(&self) -> u64 {
        1 // configuration_version
        + 1 // general_profile_space, general_tier_flag, general_profile_idc
        + 4 // general_profile_compatibility_flags
        + 6 // general_constraint_indicator_flags
        + 1 // general_level_idc
        + 2 // reserved_4bits, min_spatial_segmentation_idc
        + 1 // reserved_6bits, parallelism_type
        + 1 // reserved_6bits, chroma_format_idc
        + 1 // reserved_5bits, bit_depth_luma_minus8
        + 1 // reserved_5bits, bit_depth_chroma_minus8
        + 2 // avg_frame_rate
        + 1 // constant_frame_rate, num_temporal_layers, temporal_id_nested, length_size_minus_one
        + 1 // num_of_arrays
        + self.arrays.iter().map(|array| {
            1 // array_completeness, reserved, nal_unit_type
            + 2 // num_nalus
            + array.nalus.iter().map(|nalu| {
                2 // nal_unit_length
                + nalu.len() as u64 // nal_unit
            }).sum::<u64>()
        }).sum::<u64>()
    }

    /// Muxes the HEVCDecoderConfigurationRecord into a byte stream.
    /// Returns a muxed byte stream.
    pub fn mux<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_u8(self.configuration_version)?;
        bit_writer.write_bits(self.general_profile_space as u64, 2)?;
        bit_writer.write_bit(self.general_tier_flag)?;
        bit_writer.write_bits(self.general_profile_idc as u64, 5)?;
        bit_writer.write_u32::<LittleEndian>(self.general_profile_compatibility_flags)?;
        bit_writer.write_u48::<LittleEndian>(self.general_constraint_indicator_flags)?;
        bit_writer.write_u8(self.general_level_idc)?;

        bit_writer.write_bits(0b1111, 4)?; // reserved_4bits
        bit_writer.write_bits(self.min_spatial_segmentation_idc as u64, 12)?;

        bit_writer.write_bits(0b111111, 6)?; // reserved_6bits
        bit_writer.write_bits(self.parallelism_type as u64, 2)?;

        bit_writer.write_bits(0b111111, 6)?; // reserved_6bits
        bit_writer.write_bits(self.chroma_format_idc as u64, 2)?;

        bit_writer.write_bits(0b11111, 5)?; // reserved_5bits
        bit_writer.write_bits(self.bit_depth_luma_minus8 as u64, 3)?;

        bit_writer.write_bits(0b11111, 5)?; // reserved_5bits
        bit_writer.write_bits(self.bit_depth_chroma_minus8 as u64, 3)?;

        bit_writer.write_u16::<BigEndian>(self.avg_frame_rate)?;
        bit_writer.write_bits(self.constant_frame_rate as u64, 2)?;

        bit_writer.write_bits(self.num_temporal_layers as u64, 3)?;
        bit_writer.write_bit(self.temporal_id_nested)?;
        bit_writer.write_bits(self.length_size_minus_one as u64, 2)?;

        bit_writer.write_u8(self.arrays.len() as u8)?;
        for array in &self.arrays {
            bit_writer.write_bit(array.array_completeness)?;
            bit_writer.write_bits(0b0, 1)?; // reserved
            bit_writer.write_bits(u8::from(array.nal_unit_type) as u64, 6)?;

            bit_writer.write_u16::<BigEndian>(array.nalus.len() as u16)?;

            for nalu in &array.nalus {
                bit_writer.write_u16::<BigEndian>(nalu.len() as u16)?;
                bit_writer.write_all(nalu)?;
            }
        }

        bit_writer.finish()?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use bytes::Bytes;

    use crate::{ColorConfig, HEVCDecoderConfigurationRecord, NaluType, Sps};

    #[test]
    fn test_config_demux() {
        // h265 config
        let data = Bytes::from(b"\x01\x01@\0\0\0\x90\0\0\0\0\0\x99\xf0\0\xfc\xfd\xf8\xf8\0\0\x0f\x03 \0\x01\0\x18@\x01\x0c\x01\xff\xff\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\x95@\x90!\0\x01\0=B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\x80\x80\x82\0\0\x03\0\x02\0\0\x03\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08\"\0\x01\0\x07D\x01\xc0\x93|\x0c\xc9".to_vec());

        let config = HEVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data)).unwrap();

        assert_eq!(config.configuration_version, 1);
        assert_eq!(config.general_profile_space, 0);
        assert!(!config.general_tier_flag);
        assert_eq!(config.general_profile_idc, 1);
        assert_eq!(config.general_profile_compatibility_flags, 64);
        assert_eq!(config.general_constraint_indicator_flags, 144);
        assert_eq!(config.general_level_idc, 153);
        assert_eq!(config.min_spatial_segmentation_idc, 0);
        assert_eq!(config.parallelism_type, 0);
        assert_eq!(config.chroma_format_idc, 1);
        assert_eq!(config.bit_depth_luma_minus8, 0);
        assert_eq!(config.bit_depth_chroma_minus8, 0);
        assert_eq!(config.avg_frame_rate, 0);
        assert_eq!(config.constant_frame_rate, 0);
        assert_eq!(config.num_temporal_layers, 1);
        assert!(config.temporal_id_nested);
        assert_eq!(config.length_size_minus_one, 3);
        assert_eq!(config.arrays.len(), 3);

        let vps = &config.arrays[0];
        assert!(!vps.array_completeness);
        assert_eq!(vps.nal_unit_type, NaluType::Vps);
        assert_eq!(vps.nalus.len(), 1);

        let sps = &config.arrays[1];
        assert!(!sps.array_completeness);
        assert_eq!(sps.nal_unit_type, NaluType::Sps);
        assert_eq!(sps.nalus.len(), 1);
        let sps = Sps::parse(sps.nalus[0].clone()).unwrap();
        assert_eq!(
            sps,
            Sps {
                color_config: Some(ColorConfig {
                    full_range: false,
                    color_primaries: 1,
                    matrix_coefficients: 1,
                    transfer_characteristics: 1,
                }),
                frame_rate: 144.0,
                width: 2560,
                height: 1440,
            }
        );

        let pps = &config.arrays[2];
        assert!(!pps.array_completeness);
        assert_eq!(pps.nal_unit_type, NaluType::Pps);
        assert_eq!(pps.nalus.len(), 1);
    }

    #[test]
    fn test_config_mux() {
        let data = Bytes::from(b"\x01\x01@\0\0\0\x90\0\0\0\0\0\x99\xf0\0\xfc\xfd\xf8\xf8\0\0\x0f\x03 \0\x01\0\x18@\x01\x0c\x01\xff\xff\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\x95@\x90!\0\x01\0=B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\x80\x80\x82\0\0\x03\0\x02\0\0\x03\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08\"\0\x01\0\x07D\x01\xc0\x93|\x0c\xc9".to_vec());

        let config = HEVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data.clone())).unwrap();

        assert_eq!(config.size(), data.len() as u64);

        let mut buf = Vec::new();
        config.mux(&mut buf).unwrap();

        assert_eq!(buf, data.to_vec());
    }
}
