use std::io;

use scuffle_bytes_util::zero_copy::Deserialize;
use scuffle_bytes_util::{BitReader, BitWriter, BytesCow};

/// AV1 Video Descriptor
///
/// <https://aomediacodec.github.io/av1-mpeg2-ts/#av1-video-descriptor>
#[derive(Debug, Clone, PartialEq)]
pub struct AV1VideoDescriptor<'a> {
    /// This value shall be set to `0x80`.
    ///
    /// 8 bits
    pub tag: u8,
    /// This value shall be set to 4.
    ///
    /// 8 bits
    pub length: u8,
    /// AV1 Codec Configuration Record
    pub codec_configuration_record: AV1CodecConfigurationRecord<'a>,
}

impl<'a> Deserialize<'a> for AV1VideoDescriptor<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let tag = u8::deserialize(&mut reader)?;
        if tag != 0x80 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid AV1 video descriptor tag"));
        }

        let length = u8::deserialize(&mut reader)?;
        if length != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid AV1 video descriptor length",
            ));
        }

        Ok(AV1VideoDescriptor {
            tag,
            length,
            codec_configuration_record: AV1CodecConfigurationRecord::deserialize(reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// AV1 Codec Configuration Record
///
/// <https://aomediacodec.github.io/av1-isobmff/#av1codecconfigurationbox-syntax>
pub struct AV1CodecConfigurationRecord<'a> {
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf).
    ///
    /// 3 bits
    pub seq_profile: u8,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf).
    ///
    /// 5 bits
    pub seq_level_idx_0: u8,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf), when present.
    /// If they are not present, they will be coded using the value inferred by the semantics.
    ///
    /// 1 bit
    pub seq_tier_0: bool,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf).
    ///
    /// 1 bit
    pub high_bitdepth: bool,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf), when present.
    /// If they are not present, they will be coded using the value inferred by the semantics.
    ///
    /// 1 bit
    pub twelve_bit: bool,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf), when present.
    /// If they are not present, they will be coded using the value inferred by the semantics.
    ///
    /// 1 bit
    pub monochrome: bool,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf), when present.
    /// If they are not present, they will be coded using the value inferred by the semantics.
    ///
    /// 1 bit
    pub chroma_subsampling_x: bool,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf), when present.
    /// If they are not present, they will be coded using the value inferred by the semantics.
    ///
    /// 1 bit
    pub chroma_subsampling_y: bool,
    /// This field shall be coded according to the semantics defined in [AV1](https://aomediacodec.github.io/av1-spec/av1-spec.pdf), when present.
    /// If they are not present, they will be coded using the value inferred by the semantics.
    ///
    /// 2 bits
    pub chroma_sample_position: u8,
    /// The value of this syntax element indicates the presence or absence of high dynamic range (HDR) and/or
    /// wide color gamut (WCG) video components in the associated PID according to the table below.
    ///
    /// | HDR/WCG IDC | Description   |
    /// |-------------|---------------|
    /// | 0           | SDR           |
    /// | 1           | WCG only      |
    /// | 2           | HDR and WCG   |
    /// | 3           | No indication |
    ///
    /// 2 bits
    ///
    /// From a newer spec: <https://aomediacodec.github.io/av1-mpeg2-ts/#av1-video-descriptor>
    pub hdr_wcg_idc: u8,
    /// Ignored for [MPEG-2 TS](https://www.iso.org/standard/83239.html) use,
    /// included only to aid conversion to/from ISOBMFF.
    ///
    /// 4 bits
    pub initial_presentation_delay_minus_one: Option<u8>,
    /// Zero or more OBUs. Refer to the linked specification for details.
    ///
    /// 8 bits
    pub config_obu: BytesCow<'a>,
}

impl<'a> Deserialize<'a> for AV1CodecConfigurationRecord<'a> {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let mut bit_reader = BitReader::new(reader.as_std());

        let marker = bit_reader.read_bit()?;
        if !marker {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "marker is not set"));
        }

        let version = bit_reader.read_bits(7)? as u8;
        if version != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "version is not 1"));
        }

        let seq_profile = bit_reader.read_bits(3)? as u8;
        let seq_level_idx_0 = bit_reader.read_bits(5)? as u8;

        let seq_tier_0 = bit_reader.read_bit()?;
        let high_bitdepth = bit_reader.read_bit()?;
        let twelve_bit = bit_reader.read_bit()?;
        let monochrome = bit_reader.read_bit()?;
        let chroma_subsampling_x = bit_reader.read_bit()?;
        let chroma_subsampling_y = bit_reader.read_bit()?;
        let chroma_sample_position = bit_reader.read_bits(2)? as u8;

        // This is from the https://aomediacodec.github.io/av1-mpeg2-ts/#av1-video-descriptor spec
        // The spec from https://aomediacodec.github.io/av1-isobmff/#av1codecconfigurationbox-section is old and contains 3 bits reserved
        // The newer spec takes 2 of those reserved bits to represent the HDR WCG IDC
        // Leaving 1 bit for future use
        let hdr_wcg_idc = bit_reader.read_bits(2)? as u8;

        bit_reader.read_bits(1)?; // reserved 1 bits

        let initial_presentation_delay_minus_one = if bit_reader.read_bit()? {
            Some(bit_reader.read_bits(4)? as u8)
        } else {
            bit_reader.read_bits(4)?; // reserved 4 bits
            None
        };

        {
            let bit_reader = bit_reader; // move
            if !bit_reader.is_aligned() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Bit reader is not aligned"));
            }
        }

        Ok(AV1CodecConfigurationRecord {
            seq_profile,
            seq_level_idx_0,
            seq_tier_0,
            high_bitdepth,
            twelve_bit,
            monochrome,
            chroma_subsampling_x,
            chroma_subsampling_y,
            chroma_sample_position,
            hdr_wcg_idc,
            initial_presentation_delay_minus_one,
            config_obu: reader.try_read_to_end()?,
        })
    }
}

impl AV1CodecConfigurationRecord<'_> {
    /// Returns the size of the AV1 Codec Configuration Record.
    pub fn size(&self) -> u64 {
        1 // marker, version
        + 1 // seq_profile, seq_level_idx_0
        + 1 // seq_tier_0, high_bitdepth, twelve_bit, monochrome, chroma_subsampling_x, chroma_subsampling_y, chroma_sample_position
        + 1 // reserved, initial_presentation_delay_present, initial_presentation_delay_minus_one/reserved
        + self.config_obu.len() as u64
    }

    /// Muxes the AV1 Codec Configuration Record to the given writer.
    pub fn mux<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_bit(true)?; // marker
        bit_writer.write_bits(1, 7)?; // version

        bit_writer.write_bits(self.seq_profile as u64, 3)?;
        bit_writer.write_bits(self.seq_level_idx_0 as u64, 5)?;

        bit_writer.write_bit(self.seq_tier_0)?;
        bit_writer.write_bit(self.high_bitdepth)?;
        bit_writer.write_bit(self.twelve_bit)?;
        bit_writer.write_bit(self.monochrome)?;
        bit_writer.write_bit(self.chroma_subsampling_x)?;
        bit_writer.write_bit(self.chroma_subsampling_y)?;
        bit_writer.write_bits(self.chroma_sample_position as u64, 2)?;

        bit_writer.write_bits(0, 3)?; // reserved 3 bits

        if let Some(initial_presentation_delay_minus_one) = self.initial_presentation_delay_minus_one {
            bit_writer.write_bit(true)?;
            bit_writer.write_bits(initial_presentation_delay_minus_one as u64, 4)?;
        } else {
            bit_writer.write_bit(false)?;
            bit_writer.write_bits(0, 4)?; // reserved 4 bits
        }

        bit_writer.finish()?.write_all(self.config_obu.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;
    use scuffle_bytes_util::zero_copy::Slice;

    use super::*;

    #[test]
    fn test_config_demux() {
        let data = b"\x81\r\x0c\0\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@";

        let config = AV1CodecConfigurationRecord::deserialize(Slice::from(&data[..])).unwrap();

        insta::assert_debug_snapshot!(config, @r#"
        AV1CodecConfigurationRecord {
            seq_profile: 0,
            seq_level_idx_0: 13,
            seq_tier_0: false,
            high_bitdepth: false,
            twelve_bit: false,
            monochrome: false,
            chroma_subsampling_x: true,
            chroma_subsampling_y: true,
            chroma_sample_position: 0,
            hdr_wcg_idc: 0,
            initial_presentation_delay_minus_one: None,
            config_obu: b"\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@",
        }
        "#);
    }

    #[test]
    fn test_marker_is_not_set() {
        let data = vec![0b00000000];

        let err = AV1CodecConfigurationRecord::deserialize(Slice::from(&data[..])).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "marker is not set");
    }

    #[test]
    fn test_version_is_not_1() {
        let data = vec![0b10000000];

        let err = AV1CodecConfigurationRecord::deserialize(Slice::from(&data[..])).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "version is not 1");
    }

    #[test]
    fn test_config_demux_with_initial_presentation_delay() {
        let data = b"\x81\r\x0c\x3f\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@".to_vec();

        let config = AV1CodecConfigurationRecord::deserialize(Slice::from(&data[..])).unwrap();

        insta::assert_debug_snapshot!(config, @r#"
        AV1CodecConfigurationRecord {
            seq_profile: 0,
            seq_level_idx_0: 13,
            seq_tier_0: false,
            high_bitdepth: false,
            twelve_bit: false,
            monochrome: false,
            chroma_subsampling_x: true,
            chroma_subsampling_y: true,
            chroma_sample_position: 0,
            hdr_wcg_idc: 0,
            initial_presentation_delay_minus_one: Some(
                15,
            ),
            config_obu: b"\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@",
        }
        "#);
    }

    #[test]
    fn test_config_mux() {
        let config = AV1CodecConfigurationRecord {
            seq_profile: 0,
            seq_level_idx_0: 0,
            seq_tier_0: false,
            high_bitdepth: false,
            twelve_bit: false,
            monochrome: false,
            chroma_subsampling_x: false,
            chroma_subsampling_y: false,
            chroma_sample_position: 0,
            hdr_wcg_idc: 0,
            initial_presentation_delay_minus_one: None,
            config_obu: BytesCow::from_static(b"HELLO FROM THE OBU"),
        };

        let mut buf = Vec::new();
        config.mux(&mut buf).unwrap();

        insta::assert_snapshot!(format!("{:?}", Bytes::from(buf)), @r#"b"\x81\0\0\0HELLO FROM THE OBU""#);
    }

    #[test]
    fn test_config_mux_with_delay() {
        let config = AV1CodecConfigurationRecord {
            seq_profile: 0,
            seq_level_idx_0: 0,
            seq_tier_0: false,
            high_bitdepth: false,
            twelve_bit: false,
            monochrome: false,
            chroma_subsampling_x: false,
            chroma_subsampling_y: false,
            chroma_sample_position: 0,
            hdr_wcg_idc: 0,
            initial_presentation_delay_minus_one: Some(0),
            config_obu: BytesCow::from_static(b"HELLO FROM THE OBU"),
        };

        let mut buf = Vec::new();
        config.mux(&mut buf).unwrap();

        insta::assert_snapshot!(format!("{:?}", Bytes::from(buf)), @r#"b"\x81\0\0\x10HELLO FROM THE OBU""#);
    }

    #[test]
    fn test_video_descriptor_demux() {
        let data = b"\x80\x04\x81\r\x0c\x3f\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@".to_vec();

        let config = AV1VideoDescriptor::deserialize(Slice::from(&data[..])).unwrap();

        insta::assert_debug_snapshot!(config, @r#"
        AV1VideoDescriptor {
            tag: 128,
            length: 4,
            codec_configuration_record: AV1CodecConfigurationRecord {
                seq_profile: 0,
                seq_level_idx_0: 13,
                seq_tier_0: false,
                high_bitdepth: false,
                twelve_bit: false,
                monochrome: false,
                chroma_subsampling_x: true,
                chroma_subsampling_y: true,
                chroma_sample_position: 0,
                hdr_wcg_idc: 0,
                initial_presentation_delay_minus_one: Some(
                    15,
                ),
                config_obu: b"\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@",
            },
        }
        "#);
    }

    #[test]
    fn test_video_descriptor_demux_invalid_tag() {
        let data = b"\x81".to_vec();

        let err = AV1VideoDescriptor::deserialize(Slice::from(&data[..])).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Invalid AV1 video descriptor tag");
    }

    #[test]
    fn test_video_descriptor_demux_invalid_length() {
        let data = b"\x80\x05ju".to_vec();

        let err = AV1VideoDescriptor::deserialize(Slice::from(&data[..])).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Invalid AV1 video descriptor length");
    }
}
