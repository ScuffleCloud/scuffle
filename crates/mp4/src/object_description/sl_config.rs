use isobmff::IsoSized;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};
use scuffle_bytes_util::{BitReader, BitWriter};

use super::{BaseDescriptor, DescriptorTag};

nutype_enum! {
    /// [`SLConfigDescriptor::predefined`]
    pub enum SLConfigDescriptorPredefined(u8) {
        /// Custom.
        Custom = 0,
        /// Null SL packet header.
        NullSLPacketHeader = 1,
    }
}

impl<'a> Deserialize<'a> for SLConfigDescriptorPredefined {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        u8::deserialize(reader).map(Into::into)
    }
}

impl Serialize for SLConfigDescriptorPredefined {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.0.serialize(writer)
    }
}

impl IsoSized for SLConfigDescriptorPredefined {
    fn size(&self) -> usize {
        1
    }
}

/// SL Packet Header Configuration
///
/// ISO/IEC 14496-1 - 7.3.2.3
#[derive(Debug, PartialEq, Eq)]
pub struct SLConfigDescriptor {
    /// Allows to default the values from a set of predefined parameter sets as detailed below.
    pub predefined: SLConfigDescriptorPredefined,
    /// Set if `predefined` is [`SLConfigDescriptorPredefined::Custom`].
    pub custom: Option<SLConfigDescriptorCustom>,
}

impl SLConfigDescriptor {
    /// Returns the base descriptor of this `SLConfigDescriptor`.
    pub fn base_descriptor(&self) -> BaseDescriptor {
        BaseDescriptor {
            tag: DescriptorTag::SLConfigDescrTag,
            size_of_instance: self.payload_size() as u32,
        }
    }

    fn payload_size(&self) -> usize {
        self.predefined.size() + self.custom.size()
    }
}

impl<'a> Deserialize<'a> for SLConfigDescriptor {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let base_descriptor = BaseDescriptor::deserialize(&mut reader)?;
        let mut reader = reader.take(base_descriptor.size_of_instance as usize);

        let predefined = SLConfigDescriptorPredefined::deserialize(&mut reader)?;

        let predefined_0 = if predefined == SLConfigDescriptorPredefined::Custom {
            Some(SLConfigDescriptorCustom::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            predefined,
            custom: predefined_0,
        })
    }
}

impl Serialize for SLConfigDescriptor {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.base_descriptor().serialize(&mut writer)?;
        self.predefined.serialize(&mut writer)?;

        if self.predefined == SLConfigDescriptorPredefined::Custom {
            self.custom
                .as_ref()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "predefined_0 is required when predefined is 0",
                ))?
                .serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for SLConfigDescriptor {
    fn size(&self) -> usize {
        self.base_descriptor().size() + self.payload_size()
    }
}

/// Present in [`SLConfigDescriptor`] when [`predefined`](SLConfigDescriptor::predefined) is [`SLConfigDescriptorPredefined::Custom`].
#[derive(Debug, PartialEq, Eq)]
pub struct SLConfigDescriptorCustom {
    /// Indicates that the `accessUnitStartFlag` is present in each SL packet
    /// header of this elementary stream.
    pub use_access_unit_start_flag: bool,
    /// Indicates that the `accessUnitEndFlag` is present in each SL packet header of
    /// this elementary stream.
    pub use_access_unit_end_flag: bool,
    /// Indicates that the `RandomAccessPointFlag` is present in each SL
    /// packet header of this elementary stream.
    pub use_random_access_point_flag: bool,
    /// Indicates that each SL packet corresponds to a random access point.
    /// In that case the `randomAccessPointFlag` need not be used.
    pub has_random_access_units_only_flag: bool,
    /// Indicates that the paddingFlag is present in each SL packet header of this
    /// elementary stream.
    pub use_padding_flag: bool,
    /// Indicates that time stamps are used for synchronisation of this elementary stream.
    /// They are conveyed in the SL packet headers. Otherwise, the parameters `accessUnitDuration`,
    /// `compositionUnitDuration`, `startDecodingTimeStamp` and `startCompositionTimeStamp`
    /// conveyed in this SL packet header configuration shall be used for synchronisation.
    pub use_time_stamps_flag: bool,
    /// Indicates that `idleFlag` is used in this elementary stream.
    pub use_idle_flag: bool,
    /// Indicates that the constant duration of access units and composition units for this
    /// elementary stream is subsequently signaled.
    pub duration_flag: bool,
    /// Is the resolution of the time stamps in clock ticks per second.
    pub time_stamp_resolution: u32,
    /// Is the resolution of the object time base in cycles per second.
    pub ocr_resolution: u32,
    /// Is the length of the time stamp fields in SL packet headers. Shall
    /// take values between zero and 64 bit.
    pub time_stamp_length: u8,
    /// Is the length of the `objectClockReference` field in SL packet headers. A length of zero
    /// indicates that no `objectClockReferences` are present in this elementary stream. If `OCRstreamFlag` is
    /// set, `OCRLength` shall be zero. Else `OCRlength` shall take values between zero and 64 bit.
    pub ocr_length: u8,
    /// Is the length of the accessUnitLength fields in SL packet headers for this elementary stream.
    /// Shall take values between zero and 32 bit.
    pub au_length: u8,
    /// Is the length of the `instantBitrate` field in SL packet headers for this
    /// elementary stream.
    pub instant_bitrate_length: u8,
    /// Is the length of the `degradationPriority` field in SL packet headers
    /// for this elementary stream.
    pub degradation_priority_length: u8,
    /// Is the length of the `AU_sequenceNumber` field in SL packet headers for this
    /// elementary stream.
    pub au_seq_num_length: u8,
    /// Is the length of the `packetSequenceNumber` field in SL packet headers for this
    /// elementary stream.
    pub packet_seq_num_length: u8,
    /// Reserved 2 bits.
    pub reserved: u8,
    /// Present if [`duration_flag`](SLConfigDescriptorCustom::duration_flag) is set.
    pub duration: Option<SLConfigDescriptorDuration>,
    /// Present if [`use_time_stamps_flag`](SLConfigDescriptorCustom::use_time_stamps_flag) is set.
    pub time_stamps: Option<SLConfigDescriptorTimeStamps>,
}

impl<'a> Deserialize<'a> for SLConfigDescriptorCustom {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let byte = u8::deserialize(&mut reader)?;
        let use_access_unit_start_flag = byte & 0b1000_0000 != 0;
        let use_access_unit_end_flag = byte & 0b0100_0000 != 0;
        let use_random_access_point_flag = byte & 0b0010_0000 != 0;
        let has_random_access_units_only_flag = byte & 0b0001_0000 != 0;
        let use_padding_flag = byte & 0b0000_1000 != 0;
        let use_time_stamps_flag = byte & 0b0000_0100 != 0;
        let use_idle_flag = byte & 0b0000_0010 != 0;
        let duration_flag = byte & 0b0000_0001 != 0;

        let time_stamp_resolution = u32::deserialize(&mut reader)?;
        let ocr_resolution = u32::deserialize(&mut reader)?;
        let time_stamp_length = u8::deserialize(&mut reader)?;
        if time_stamp_length > 64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "timeStampLength must be <= 64",
            ));
        }

        let ocr_length = u8::deserialize(&mut reader)?;
        if ocr_length > 64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "OCRLength must be <= 64",
            ));
        }

        let au_length = u8::deserialize(&mut reader)?;
        if au_length > 32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "AU_Length must be <= 64",
            ));
        }

        let instant_bitrate_length = u8::deserialize(&mut reader)?;

        let bytes = u16::deserialize(&mut reader)?;
        let degradation_priority_length = ((bytes & 0b1111_0000_0000_0000) >> 12) as u8;
        let au_seq_num_length = ((bytes & 0b0000_1111_1000_0000) >> 7) as u8;
        if au_seq_num_length > 16 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "AU_SeqNumLength must be <= 16",
            ));
        }
        let packet_seq_num_length = (byte & 0b0000_0000_0111_1100) >> 2;
        if packet_seq_num_length > 16 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "packetSeqNumLength must be <= 16",
            ));
        }
        let reserved = byte & 0b0000_0000_0000_0011;

        let duration = if duration_flag {
            Some(SLConfigDescriptorDuration::deserialize(&mut reader)?)
        } else {
            None
        };

        let time_stamps = if use_time_stamps_flag {
            Some(SLConfigDescriptorTimeStamps::deserialize_seed(
                &mut reader,
                time_stamp_length,
            )?)
        } else {
            None
        };

        Ok(Self {
            use_access_unit_start_flag,
            use_access_unit_end_flag,
            use_random_access_point_flag,
            has_random_access_units_only_flag,
            use_padding_flag,
            use_time_stamps_flag,
            use_idle_flag,
            duration_flag,
            time_stamp_resolution,
            ocr_resolution,
            time_stamp_length,
            ocr_length,
            au_length,
            instant_bitrate_length,
            degradation_priority_length,
            au_seq_num_length,
            packet_seq_num_length,
            reserved,
            duration,
            time_stamps,
        })
    }
}

impl Serialize for SLConfigDescriptorCustom {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        bit_writer.write_bit(self.use_access_unit_start_flag)?;
        bit_writer.write_bit(self.use_access_unit_end_flag)?;
        bit_writer.write_bit(self.use_random_access_point_flag)?;
        bit_writer.write_bit(self.has_random_access_units_only_flag)?;
        bit_writer.write_bit(self.use_padding_flag)?;
        bit_writer.write_bit(self.use_time_stamps_flag)?;
        bit_writer.write_bit(self.use_idle_flag)?;
        bit_writer.write_bit(self.duration_flag)?;

        self.time_stamp_resolution.serialize(&mut bit_writer)?;
        self.ocr_resolution.serialize(&mut bit_writer)?;
        self.time_stamp_length.serialize(&mut bit_writer)?;
        self.ocr_length.serialize(&mut bit_writer)?;
        self.au_length.serialize(&mut bit_writer)?;
        self.instant_bitrate_length.serialize(&mut bit_writer)?;
        self.degradation_priority_length.serialize(&mut bit_writer)?;
        self.au_seq_num_length.serialize(&mut bit_writer)?;
        self.packet_seq_num_length.serialize(&mut bit_writer)?;
        self.reserved.serialize(&mut bit_writer)?;

        if self.duration_flag {
            self.duration
                .as_ref()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "duration is required when duration_flag is set",
                ))?
                .serialize(&mut bit_writer)?;
        }

        if self.use_time_stamps_flag {
            self.time_stamps
                .as_ref()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "time_stamps is required when use_time_stamps_flag is set",
                ))?
                .serialize(&mut bit_writer, self.time_stamp_length)?;
        }

        Ok(())
    }
}

impl IsoSized for SLConfigDescriptorCustom {
    fn size(&self) -> usize {
        let mut size = 1; // flags
        size += self.time_stamp_resolution.size();
        size += self.ocr_resolution.size();
        size += self.time_stamp_length.size();
        size += self.ocr_length.size();
        size += self.au_length.size();
        size += self.instant_bitrate_length.size();
        size += self.degradation_priority_length.size();
        size += self.au_seq_num_length.size();
        size += self.packet_seq_num_length.size();
        size += self.reserved.size();
        size += self.duration.size();

        if let Some(time_stamps) = &self.time_stamps {
            size += time_stamps.size(self.time_stamp_length);
        }

        size
    }
}

/// Present in [`SLConfigDescriptorCustom`] when [`duration_flag`](SLConfigDescriptorCustom::duration_flag) is set.
#[derive(Debug, PartialEq, Eq)]
pub struct SLConfigDescriptorDuration {
    /// Used to express the duration of access units and composition units. One second is evenly
    /// divided in timeScale parts.
    pub time_scale: u32,
    /// The duration of an access unit is `accessUnitDuration * 1/timeScale` seconds.
    pub access_unit_duration: u16,
    /// The duration of a composition unit is `compositionUnitDuration * 1/timeScale` seconds.
    pub composition_unit_duration: u16,
}

impl<'a> Deserialize<'a> for SLConfigDescriptorDuration {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Ok(Self {
            time_scale: u32::deserialize(&mut reader)?,
            access_unit_duration: u16::deserialize(&mut reader)?,
            composition_unit_duration: u16::deserialize(&mut reader)?,
        })
    }
}

impl Serialize for SLConfigDescriptorDuration {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.time_scale.serialize(&mut writer)?;
        self.access_unit_duration.serialize(&mut writer)?;
        self.composition_unit_duration.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for SLConfigDescriptorDuration {
    fn size(&self) -> usize {
        self.time_scale.size() + self.access_unit_duration.size() + self.composition_unit_duration.size()
    }
}

/// Present in [`SLConfigDescriptorCustom`] when [`use_time_stamps_flag`](SLConfigDescriptorCustom::use_time_stamps_flag) is set.
#[derive(Debug, PartialEq, Eq)]
pub struct SLConfigDescriptorTimeStamps {
    /// Conveys the time at which the first access unit of this elementary stream shall
    /// be decoded. It is conveyed in the resolution specified by `timeStampResolution`.
    pub start_decoding_time_stamp: u64,
    /// Conveys the time at which the composition unit corresponding to the first
    /// access unit of this elementary stream shall be decoded. It is conveyed in the resolution specified by
    /// `timeStampResolution`.
    pub start_composition_time_stamp: u64,
}

impl<'a> DeserializeSeed<'a, u8> for SLConfigDescriptorTimeStamps {
    fn deserialize_seed<R>(mut reader: R, seed: u8) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let mut bit_reader = BitReader::new(reader.as_std());

        Ok(Self {
            start_decoding_time_stamp: bit_reader.read_bits(seed)?,
            start_composition_time_stamp: bit_reader.read_bits(seed)?,
        })
    }
}

impl SLConfigDescriptorTimeStamps {
    /// Serialize the timestamps using the given timestamp length
    pub fn serialize<W>(&self, writer: W, time_stamp_length: u8) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);
        bit_writer.write_bits(self.start_decoding_time_stamp, time_stamp_length)?;
        bit_writer.write_bits(self.start_composition_time_stamp, time_stamp_length)?;
        Ok(())
    }

    /// Calculate the size of the serialized timestamps based on the given timestamp length.
    pub fn size(&self, time_stamp_length: u8) -> usize {
        time_stamp_length as usize * 2
    }
}
