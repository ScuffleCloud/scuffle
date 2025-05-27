use std::io::{
    Read, Write, {self},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};
use scuffle_bytes_util::{BitReader, BitWriter, BytesCow};

use crate::{ConstantFrameRate, NALUnitType, NumTemporalLayers, ParallelismType, ProfileCompatibilityFlags};

/// HEVC Decoder Configuration Record.
///
/// ISO/IEC 14496-15 - 8.3.2.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HEVCDecoderConfigurationRecord<'a> {
    /// Matches the [`general_profile_space`](crate::Profile::profile_space) field as defined in ISO/IEC 23008-2.
    pub general_profile_space: u8,
    /// Matches the [`general_tier_flag`](crate::Profile::tier_flag) field as defined in ISO/IEC 23008-2.
    pub general_tier_flag: bool,
    /// Matches the [`general_profile_idc`](crate::Profile::profile_idc) field as defined in ISO/IEC 23008-2.
    pub general_profile_idc: u8,
    /// Matches the [`general_profile_compatibility_flag`](crate::Profile::profile_compatibility_flag) field as defined in ISO/IEC 23008-2.
    pub general_profile_compatibility_flags: ProfileCompatibilityFlags,
    /// This is stored as a 48-bit (6 bytes) unsigned integer.
    /// Therefore only the first 48 bits of this value are used.
    pub general_constraint_indicator_flags: u64,
    /// Matches the [`general_level_idc`](crate::Profile::level_idc) field as defined in ISO/IEC 23008-2.
    pub general_level_idc: u8,
    /// Matches the [`min_spatial_segmentation_idc`](crate::BitStreamRestriction::min_spatial_segmentation_idc) field as defined in ISO/IEC 23008-2.
    pub min_spatial_segmentation_idc: u16,
    /// See [`ParallelismType`] for more info.
    pub parallelism_type: ParallelismType,
    /// Matches the [`chroma_format_idc`](crate::SpsRbsp::chroma_format_idc) field as defined in ISO/IEC 23008-2.
    pub chroma_format_idc: u8,
    /// Matches the [`bit_depth_luma_minus8`](crate::SpsRbsp::bit_depth_luma_minus8) field as defined in ISO/IEC 23008-2.
    pub bit_depth_luma_minus8: u8,
    /// Matches the [`bit_depth_chroma_minus8`](crate::SpsRbsp::bit_depth_chroma_minus8) field as defined in ISO/IEC 23008-2.
    pub bit_depth_chroma_minus8: u8,
    /// Gives the average frame rate in units of frames/(256 seconds), for the stream to
    /// which this configuration record applies.
    ///
    /// Value 0 indicates an unspecified average frame rate.
    pub avg_frame_rate: u16,
    /// See [`ConstantFrameRate`] for more info.
    pub constant_frame_rate: ConstantFrameRate,
    /// This is the count of tepmoral layers or sub-layers as defined in ISO/IEC 23008-2.
    pub num_temporal_layers: NumTemporalLayers,
    /// Equal to `true` indicates that all SPSs that are activated when the stream to which
    /// this configuration record applies is decoded have
    /// [`sps_temporal_id_nesting_flag`](crate::SpsRbsp::sps_temporal_id_nesting_flag) as defined in
    /// ISO/IEC 23008-2 equal to `true` and temporal sub-layer up-switching to any higher temporal layer
    /// can be performed at any sample.
    ///
    /// Value `false` indicates that the conditions above are not or may not be met.
    pub temporal_id_nested: bool,
    /// This value plus 1 indicates the length in bytes of the `NALUnitLength` field in an
    /// HEVC video sample in the stream to which this configuration record applies.
    ///
    /// For example, a size of one byte is indicated with a value of 0.
    /// The value of this field is one of 0, 1, or 3
    /// corresponding to a length encoded with 1, 2, or 4 bytes, respectively.
    pub length_size_minus_one: u8,
    /// [`NaluArray`]s in that are part of this configuration record.
    pub arrays: Vec<NaluArray<'a>>,
}

/// Nalu Array Structure
///
/// ISO/IEC 14496-15 - 8.3.2.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaluArray<'a> {
    /// When equal to `true` indicates that all NAL units of the given type are in the
    /// following array and none are in the stream; when equal to `false` indicates that additional NAL units
    /// of the indicated type may be in the stream; the default and permitted values are constrained by
    /// the sample entry name.
    pub array_completeness: bool,
    /// Indicates the type of the NAL units in the following array (which shall be all of
    /// that type); it takes a value as defined in ISO/IEC 23008-2; it is restricted to take one of the
    /// values indicating a VPS, SPS, PPS, prefix SEI, or suffix SEI NAL unit.
    pub nal_unit_type: NALUnitType,
    /// The raw byte stream of NAL units.
    ///
    /// You might want to use [`SpsNALUnit::parse`](crate::SpsNALUnit::parse)
    /// to parse an SPS NAL unit.
    pub nalus: Vec<BytesCow<'a>>,
}

impl<'a> Deserialize<'a> for HEVCDecoderConfigurationRecord<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let mut bit_reader = BitReader::new(reader.as_std());

        // This demuxer only supports version 1
        let configuration_version = bit_reader.read_u8()?;
        if configuration_version != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid configuration version"));
        }

        let general_profile_space = bit_reader.read_bits(2)? as u8;
        let general_tier_flag = bit_reader.read_bit()?;
        let general_profile_idc = bit_reader.read_bits(5)? as u8;
        let general_profile_compatibility_flags =
            ProfileCompatibilityFlags::from_bits_retain(bit_reader.read_u32::<BigEndian>()?);
        let general_constraint_indicator_flags = bit_reader.read_u48::<BigEndian>()?;
        let general_level_idc = bit_reader.read_u8()?;

        bit_reader.read_bits(4)?; // reserved_4bits
        let min_spatial_segmentation_idc = bit_reader.read_bits(12)? as u16;

        bit_reader.read_bits(6)?; // reserved_6bits
        let parallelism_type = bit_reader.read_bits(2)? as u8;

        bit_reader.read_bits(6)?; // reserved_6bits
        let chroma_format_idc = bit_reader.read_bits(2)? as u8;

        bit_reader.read_bits(5)?; // reserved_5bits
        let bit_depth_luma_minus8 = bit_reader.read_bits(3)? as u8;

        bit_reader.read_bits(5)?; // reserved_5bits
        let bit_depth_chroma_minus8 = bit_reader.read_bits(3)? as u8;

        let avg_frame_rate = bit_reader.read_u16::<BigEndian>()?;
        let constant_frame_rate = bit_reader.read_bits(2)? as u8;
        let num_temporal_layers = bit_reader.read_bits(3)? as u8;
        let temporal_id_nested = bit_reader.read_bit()?;
        let length_size_minus_one = bit_reader.read_bits(2)? as u8;

        if length_size_minus_one == 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "length_size_minus_one must be 0, 1, or 3",
            ));
        }

        let num_of_arrays = bit_reader.read_u8()?;

        let mut arrays = Vec::with_capacity(num_of_arrays as usize);

        for _ in 0..num_of_arrays {
            let array_completeness = bit_reader.read_bit()?;
            bit_reader.read_bits(1)?; // reserved

            let nal_unit_type = bit_reader.read_bits(6)? as u8;
            let nal_unit_type = NALUnitType::from(nal_unit_type);
            if nal_unit_type != NALUnitType::VpsNut
                && nal_unit_type != NALUnitType::SpsNut
                && nal_unit_type != NALUnitType::PpsNut
                && nal_unit_type != NALUnitType::PrefixSeiNut
                && nal_unit_type != NALUnitType::SuffixSeiNut
            {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid nal_unit_type"));
            }

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
                nal_unit_type,
                nalus,
            });
        }

        Ok(HEVCDecoderConfigurationRecord {
            general_profile_space,
            general_tier_flag,
            general_profile_idc,
            general_profile_compatibility_flags,
            general_constraint_indicator_flags,
            general_level_idc,
            min_spatial_segmentation_idc,
            parallelism_type: ParallelismType(parallelism_type),
            chroma_format_idc,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            avg_frame_rate,
            constant_frame_rate: ConstantFrameRate(constant_frame_rate),
            num_temporal_layers: NumTemporalLayers(num_temporal_layers),
            temporal_id_nested,
            length_size_minus_one,
            arrays,
        })
    }
}

impl Serialize for HEVCDecoderConfigurationRecord<'_> {
    fn serialize<W>(&self, writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        // This muxer only supports version 1
        bit_writer.write_u8(1)?; // configuration_version
        bit_writer.write_bits(self.general_profile_space as u64, 2)?;
        bit_writer.write_bit(self.general_tier_flag)?;
        bit_writer.write_bits(self.general_profile_idc as u64, 5)?;
        bit_writer.write_u32::<BigEndian>(self.general_profile_compatibility_flags.bits())?;
        bit_writer.write_u48::<BigEndian>(self.general_constraint_indicator_flags)?;
        bit_writer.write_u8(self.general_level_idc)?;

        bit_writer.write_bits(0b1111, 4)?; // reserved_4bits
        bit_writer.write_bits(self.min_spatial_segmentation_idc as u64, 12)?;

        bit_writer.write_bits(0b111111, 6)?; // reserved_6bits
        bit_writer.write_bits(self.parallelism_type.0 as u64, 2)?;

        bit_writer.write_bits(0b111111, 6)?; // reserved_6bits
        bit_writer.write_bits(self.chroma_format_idc as u64, 2)?;

        bit_writer.write_bits(0b11111, 5)?; // reserved_5bits
        bit_writer.write_bits(self.bit_depth_luma_minus8 as u64, 3)?;

        bit_writer.write_bits(0b11111, 5)?; // reserved_5bits
        bit_writer.write_bits(self.bit_depth_chroma_minus8 as u64, 3)?;

        bit_writer.write_u16::<BigEndian>(self.avg_frame_rate)?;
        bit_writer.write_bits(self.constant_frame_rate.0 as u64, 2)?;

        bit_writer.write_bits(self.num_temporal_layers.0 as u64, 3)?;
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
                bit_writer.write_all(nalu.as_bytes())?;
            }
        }

        bit_writer.finish()?;

        Ok(())
    }
}

impl isobmff::IsoSized for HEVCDecoderConfigurationRecord<'_> {
    fn size(&self) -> usize {
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
                + nalu.len() // nal_unit
            }).sum::<usize>()
        }).sum::<usize>()
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use isobmff::IsoSized;
    use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, Slice};

    use crate::{
        ConstantFrameRate, HEVCDecoderConfigurationRecord, NALUnitType, NumTemporalLayers, ParallelismType,
        ProfileCompatibilityFlags, SpsNALUnit,
    };

    #[test]
    fn test_config_demux() {
        // h265 config
        let data = b"\x01\x01@\0\0\0\x90\0\0\0\0\0\x99\xf0\0\xfc\xfd\xf8\xf8\0\0\x0f\x03 \0\x01\0\x18@\x01\x0c\x01\xff\xff\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\x95@\x90!\0\x01\0=B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\x80\x80\x82\0\0\x03\0\x02\0\0\x03\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08\"\0\x01\0\x07D\x01\xc0\x93|\x0c\xc9";

        let config = HEVCDecoderConfigurationRecord::deserialize(Slice::from(&data[..])).unwrap();

        assert_eq!(config.general_profile_space, 0);
        assert!(!config.general_tier_flag);
        assert_eq!(config.general_profile_idc, 1);
        assert_eq!(
            config.general_profile_compatibility_flags,
            ProfileCompatibilityFlags::MainProfile
        );
        assert_eq!(config.general_constraint_indicator_flags, (1 << 47) | (1 << 44)); // 1. bit and 4. bit
        assert_eq!(config.general_level_idc, 153);
        assert_eq!(config.min_spatial_segmentation_idc, 0);
        assert_eq!(config.parallelism_type, ParallelismType::MixedOrUnknown);
        assert_eq!(config.chroma_format_idc, 1);
        assert_eq!(config.bit_depth_luma_minus8, 0);
        assert_eq!(config.bit_depth_chroma_minus8, 0);
        assert_eq!(config.avg_frame_rate, 0);
        assert_eq!(config.constant_frame_rate, ConstantFrameRate::Unknown);
        assert_eq!(config.num_temporal_layers, NumTemporalLayers::NotScalable);
        assert!(config.temporal_id_nested);
        assert_eq!(config.length_size_minus_one, 3);
        assert_eq!(config.arrays.len(), 3);

        let vps = &config.arrays[0];
        assert!(!vps.array_completeness);
        assert_eq!(vps.nal_unit_type, NALUnitType::VpsNut);
        assert_eq!(vps.nalus.len(), 1);

        let sps = &config.arrays[1];
        assert!(!sps.array_completeness);
        assert_eq!(sps.nal_unit_type, NALUnitType::SpsNut);
        assert_eq!(sps.nalus.len(), 1);
        let sps = SpsNALUnit::parse(io::Cursor::new(sps.nalus[0].clone())).unwrap();
        insta::assert_debug_snapshot!(sps);

        let pps = &config.arrays[2];
        assert!(!pps.array_completeness);
        assert_eq!(pps.nal_unit_type, NALUnitType::PpsNut);
        assert_eq!(pps.nalus.len(), 1);
    }

    #[test]
    fn test_config_mux() {
        let data = b"\x01\x01@\0\0\0\x90\0\0\0\0\0\x99\xf0\0\xfc\xfd\xf8\xf8\0\0\x0f\x03 \0\x01\0\x18@\x01\x0c\x01\xff\xff\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\x95@\x90!\0\x01\0=B\x01\x01\x01@\0\0\x03\0\x90\0\0\x03\0\0\x03\0\x99\xa0\x01@ \x05\xa1e\x95R\x90\x84d_\xf8\xc0Z\x80\x80\x80\x82\0\0\x03\0\x02\0\0\x03\x01 \xc0\x0b\xbc\xa2\0\x02bX\0\x011-\x08\"\0\x01\0\x07D\x01\xc0\x93|\x0c\xc9";

        let config = HEVCDecoderConfigurationRecord::deserialize(Slice::from(&data[..])).unwrap();

        assert_eq!(config.size(), data.len());

        let mut buf = Vec::new();
        config.serialize(&mut buf).unwrap();

        assert_eq!(buf, data.to_vec());
    }
}
