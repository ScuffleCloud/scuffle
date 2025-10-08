use isobmff::IsoSized;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be};
use scuffle_bytes_util::{BitWriter, IoResultExt};

use super::profile_level_indication_index::ProfileLevelIndicationIndexDescriptor;
use super::{BaseDescriptor, DescriptorTag, UnknownDescriptor};

nutype_enum! {
    /// ObjectTypeIndication
    ///
    /// ISO/IEC 14496-1 - 7.2.6.6.2
    pub enum ObjectTypeIndication(u8) {
        /// Forbidden
        Forbidden = 0x00,
        /// Systems ISO/IEC 14496-1 a
        Systems14496_1_a = 0x01,
        /// Systems ISO/IEC 14496-1 b
        Systems14496_1_b = 0x02,
        /// Interaction Stream
        InteractionStream = 0x03,
        /// Systems ISO/IEC 14496-1 Extended BIFS Configuration
        Systems14496_1_ExtendedBIFSConfiguration = 0x04,
        /// Systems ISO/IEC 14496-1 AFX
        Systems14496_1_AFX = 0x05,
        /// Font Data Stream
        FontDataStream = 0x06,
        /// Synthesized Texture Stream
        SynthesizedTextureStream = 0x07,
        /// Streaming Text Stream
        StreamingTextStream = 0x08,
        /// Visual ISO/IEC 14496-2
        Visual14496_2 = 0x20,
        /// Visual ITU-T Recommendation H.264 | ISO/IEC 14496-10
        Visual14496_10 = 0x21,
        /// Parameter Sets for ITU-T Recommendation H.264 | ISO/IEC 14496-10
        ParameterSets_14496_10 = 0x22,
        /// Audio ISO/IEC 14496-3
        Audio14496_3 = 0x40,
        /// Visual ISO/IEC 13818-2 Simple Profile
        Visual13818_2_SimpleProfile = 0x60,
        /// Visual ISO/IEC 13818-2 Main Profile
        Visual13818_2_MainProfile = 0x61,
        /// Visual ISO/IEC 13818-2 SNR Profile
        Visual13818_2_SNRProfile = 0x62,
        /// Visual ISO/IEC 13818-2 Spatial Profile
        Visual13818_2_SpatialProfile = 0x63,
        /// Visual ISO/IEC 13818-2 High Profile
        Visual13818_2_HighProfile = 0x64,
        /// Visual ISO/IEC 13818-2 422 Profile
        Visual13818_2_422Profile = 0x65,
        /// Audio ISO/IEC 13818-7 Main Profile
        Audio13818_7_MainProfile = 0x66,
        /// Audio ISO/IEC 13818-7 LowComplexity Profile
        Audio13818_7_LowComplexityProfile = 0x67,
        /// Audio ISO/IEC 13818-7 Scaleable Sampling Rate Profile
        Audio13818_7_ScaleableSamplingRateProfile = 0x68,
        /// Audio ISO/IEC 13818-3
        Audio13818_3 = 0x69,
        /// Visual ISO/IEC 11172-2
        Visual11172_2 = 0x6A,
        /// Audio ISO/IEC 11172-3
        Audio11172_3 = 0x6B,
        /// Visual ISO/IEC 10918-1
        Visual10918_1 = 0x6C,
        /// Visual ISO/IEC 15444-1
        Visual15444_1 = 0x6E,
        /// No object type specified
        Unspecified = 0xFF,
    }
}

impl<'a> Deserialize<'a> for ObjectTypeIndication {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        u8::deserialize(reader).map(Into::into)
    }
}

impl Serialize for ObjectTypeIndication {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.0.serialize(writer)
    }
}

impl IsoSized for ObjectTypeIndication {
    fn size(&self) -> usize {
        1
    }
}

nutype_enum! {
    /// StreamType
    ///
    /// ISO/IEC 14496-1 - 7.2.6.6.2
    pub enum StreamType(u8) {
        /// Forbidden
        Forbidden = 0x00,
        /// ObjectDescriptorStream
        ///
        /// See ISO/IEC 14496-1 - 7.2.5
        ObjectDescriptorStream = 0x01,
        /// ClockReferenceStream
        ///
        /// See ISO/IEC 14496-1 - 7.3.2.5
        ClockReferenceStream = 0x02,
        /// SceneDescriptionStream
        ///
        /// See ISO/IEC 14496-11
        SceneDescriptionStream = 0x03,
        /// VisualStream
        VisualStream = 0x04,
        /// AudioStream
        AudioStream = 0x05,
        /// MPEG7Stream
        MPEG7Stream = 0x06,
        /// IPMPStream
        ///
        /// See ISO/IEC 14496-1 - 7.2.3.2
        IPMPStream = 0x07,
        /// ObjectContentInfoStream
        ///
        ///  See ISO/IEC 14496-1 - 7.2.4.2
        ObjectContentInfoStream = 0x08,
        /// MPEGJStream
        MPEGJStream = 0x09,
        /// Interaction Stream
        InteractionStream = 0x0A,
        /// IPMPToolStream
        ///
        /// See ISO/IEC 14496-13
        IPMPToolStream = 0x0B,
    }
}

/// Deocder Config Descriptor
///
/// ISO/IEC 14496-1 - 7.2.6.6
#[derive(Debug, PartialEq, Eq)]
pub struct DecoderConfigDescriptor<'a> {
    /// An indication of the object or scene description type that needs to be supported
    /// by the decoder for this elementary stream.
    pub object_type_indication: ObjectTypeIndication,
    /// Conveys the type of this elementary stream.
    pub stream_type: StreamType,
    /// Indicates that this stream is used for upstream information.
    pub up_stream: bool,
    /// Reserved bit.
    pub reserved: bool,
    /// Is the size of the decoding buffer for this elementary stream in bytes.
    pub buffer_size_db: U24Be,
    /// Is the maximum bitrate in bits per second of this elementary stream in any time window of
    /// one second duration.
    pub max_bitrate: u32,
    /// Is the average bitrate in bits per second of this elementary stream. For streams with variable
    /// bitrate this value shall be set to zero.
    pub avg_bitrate: u32,
    /// Decoder specific information.
    pub dec_specific_info: Option<UnknownDescriptor<'a>>,
    /// A list of [`ProfileLevelIndicationIndexDescriptor`]s.
    pub profile_level_indication_index_descr: Vec<ProfileLevelIndicationIndexDescriptor>,
    /// Any other unknown descriptors that are contained in this descriptor but not deserialized.
    pub unknown_descriptors: Vec<UnknownDescriptor<'a>>,
}

impl DecoderConfigDescriptor<'_> {
    fn payload_size(&self) -> usize {
        let mut size = 0;
        size += self.object_type_indication.size();
        size += 1; // stream_type + up_stream + reserved
        size += self.buffer_size_db.size();
        size += self.max_bitrate.size();
        size += self.avg_bitrate.size();

        size += self.dec_specific_info.size();
        size += self.profile_level_indication_index_descr.size();
        size += self.unknown_descriptors.size();

        size
    }

    /// Returns the base descriptor.
    pub fn base_descriptor(&self) -> BaseDescriptor {
        BaseDescriptor {
            tag: DescriptorTag::DecoderConfigDescrTag,
            size_of_instance: self.payload_size() as u32,
        }
    }
}

impl<'a> Deserialize<'a> for DecoderConfigDescriptor<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let base_descriptor = BaseDescriptor::deserialize(&mut reader)?;
        let mut reader = reader.take(base_descriptor.size_of_instance as usize);

        let object_type_indication = ObjectTypeIndication::deserialize(&mut reader)?;

        let byte = u8::deserialize(&mut reader)?;
        let stream_type = StreamType::from((byte & 0b111_1100) >> 2);
        let up_stream = (byte & 0b0000_0010) != 0;
        let reserved = (byte & 0b0000_0001) != 0;
        let buffer_size_db = U24Be::deserialize(&mut reader)?;
        let max_bitrate = u32::deserialize(&mut reader)?;
        let avg_bitrate = u32::deserialize(&mut reader)?;

        let mut dec_specific_info = None;
        let mut profile_level_indication_index_descr = Vec::new();
        let mut unknown_descriptors = Vec::new();

        loop {
            let Some(base_descriptor) = BaseDescriptor::deserialize(&mut reader).eof_to_none()? else {
                break;
            };

            match base_descriptor.tag {
                DescriptorTag::DecSpecificInfoTag => {
                    let Some(descr) = UnknownDescriptor::deserialize_seed(&mut reader, base_descriptor).eof_to_none()?
                    else {
                        break;
                    };
                    dec_specific_info = Some(descr);
                }
                DescriptorTag::profileLevelIndicationIndexDescrTag => {
                    let Some(descr) = ProfileLevelIndicationIndexDescriptor::deserialize_seed(&mut reader, base_descriptor)
                        .eof_to_none()?
                    else {
                        break;
                    };
                    profile_level_indication_index_descr.push(descr);
                }
                _ => {
                    let Some(descr) = UnknownDescriptor::deserialize_seed(&mut reader, base_descriptor).eof_to_none()?
                    else {
                        break;
                    };
                    unknown_descriptors.push(descr);
                }
            }
        }

        Ok(Self {
            object_type_indication,
            stream_type,
            up_stream,
            reserved,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            dec_specific_info,
            profile_level_indication_index_descr,
            unknown_descriptors,
        })
    }
}

impl Serialize for DecoderConfigDescriptor<'_> {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        self.base_descriptor().serialize(&mut bit_writer)?;
        self.object_type_indication.serialize(&mut bit_writer)?;
        bit_writer.write_bits(self.stream_type.0 as u64, 6)?;
        bit_writer.write_bit(self.up_stream)?;
        bit_writer.write_bit(self.reserved)?;
        self.buffer_size_db.serialize(&mut bit_writer)?;
        self.max_bitrate.serialize(&mut bit_writer)?;
        self.avg_bitrate.serialize(&mut bit_writer)?;

        if let Some(dec_specific_info) = &self.dec_specific_info {
            dec_specific_info.serialize(&mut bit_writer)?;
        }

        for profile_level_indication_index_descr in &self.profile_level_indication_index_descr {
            profile_level_indication_index_descr.serialize(&mut bit_writer)?;
        }

        for unknown_descriptor in &self.unknown_descriptors {
            unknown_descriptor.serialize(&mut bit_writer)?;
        }

        Ok(())
    }
}

impl IsoSized for DecoderConfigDescriptor<'_> {
    fn size(&self) -> usize {
        self.base_descriptor().size() + self.payload_size()
    }
}
