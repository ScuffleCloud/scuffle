use isobmff::IsoSized;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, ZeroCopyReader};
use scuffle_bytes_util::{BitWriter, IoResultExt, StringCow};

use super::decoder_config::DecoderConfigDescriptor;
use super::sl_config::SLConfigDescriptor;
use super::{BaseDescriptor, DescriptorTag, UnknownDescriptor};

/// ES Descriptor
///
/// ISO/IEC 14496-1 - 7.2.6.5
#[derive(Debug)]
pub struct ESDescriptor<'a> {
    /// Provides a unique label for each elementary stream within its name scope. The
    /// values 0 and 0xFFFF are reserved.
    pub es_id: u16,
    /// Indicates a relative measure for the priority of this elementary stream. An elementary stream with a
    /// higher `streamPriority` is more important than one with a lower `streamPriority`. The absolute values of
    /// `streamPriority` are not normatively defined.
    pub stream_priority: u8,
    /// Is the `ES_ID` of another elementary stream on which this elementary stream depends.
    /// The stream with `dependsOn_ES_ID` shall also be associated to the same object descriptor as the current
    /// ES_Descriptor.
    pub depends_on_es_id: Option<u16>,
    /// Points to the location of an SL-packetized stream by name.
    /// The parameters of the SL-packetized stream that is retrieved from the URL are
    /// fully specified in this ES_Descriptor. See also 7.2.7.3.3. Permissible URLs may be constrained by profile
    /// and levels as well as by specific delivery layers.
    pub url_string: Option<StringCow<'a>>,
    /// Indicates the `ES_ID` of the elementary stream within the name scope (see 7.2.7.2.4) from
    /// which the time base for this elementary stream is derived. Circular references between elementary streams
    /// are not permitted.
    pub ocr_es_id: Option<u16>,
    /// A [`DecoderConfigDescriptor`].
    pub dec_config_descr: DecoderConfigDescriptor<'a>,
    /// An [`SLConfigDescriptor`].
    pub sl_config_descr: Option<SLConfigDescriptor>, /* (IMPORTANT) TODO: The spec says that this is required and NOT optional. */
    /// A list of descriptors that are contained in this descriptor but not deserialized.
    ///
    /// Could be any of:
    /// - `IPI_DescrPointer`
    /// - `IP_IdentificationDataSet`
    /// - `IPMP_DescriptorPointer`
    /// - `LanguageDescriptor`
    /// - `QoS_Descriptor`
    /// - `RegistrationDescriptor`
    /// - `ExtensionDescriptor`
    pub unknown_descriptors: Vec<UnknownDescriptor<'a>>,
}

impl ESDescriptor<'_> {
    /// Returns the base descriptor of this `ESDescriptor`.
    pub fn base_descriptor(&self) -> BaseDescriptor {
        BaseDescriptor {
            tag: DescriptorTag::ES_DescrTag,
            size_of_instance: self.payload_size() as u32,
        }
    }

    fn payload_size(&self) -> usize {
        let mut size = 0;
        size += self.es_id.size(); // es_id
        size += 1; // flags + stream_priority

        size += self.depends_on_es_id.size();

        if let Some(url_string) = self.url_string.as_ref() {
            size += 1; // url_length
            size += url_string.size();
        }

        size += self.ocr_es_id.size();
        size += self.dec_config_descr.size();
        size += self.sl_config_descr.size();
        size += self.unknown_descriptors.size();

        size
    }
}

impl<'a> Deserialize<'a> for ESDescriptor<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let base_descriptor = BaseDescriptor::deserialize(&mut reader)?;
        let mut reader = reader.take(base_descriptor.size_of_instance as usize);

        let es_id = u16::deserialize(&mut reader)?;

        let byte = u8::deserialize(&mut reader)?;
        let stream_dependence_flag = (byte & 0b1000_0000) != 0;
        let url_flag = (byte & 0b0100_0000) != 0;
        let ocr_stream_flag = (byte & 0b0010_0000) != 0;
        let stream_priority = byte & 0b0001_1111;

        let depends_on_es_id = if stream_dependence_flag {
            Some(u16::deserialize(&mut reader)?)
        } else {
            None
        };

        let url_string = if url_flag {
            let url_length = u8::deserialize(&mut reader)?;
            let url_string = reader.try_read(url_length as usize)?;
            Some(url_string.try_into().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("URLString must be valid UTF-8: {e}"))
            })?)
        } else {
            None
        };

        let ocr_es_id = if ocr_stream_flag {
            Some(u16::deserialize(&mut reader)?)
        } else {
            None
        };

        let dec_config_descr = DecoderConfigDescriptor::deserialize(&mut reader)?;
        let sl_config_descr = SLConfigDescriptor::deserialize(&mut reader).eof_to_none()?;

        let mut unknown_descriptors = Vec::new();

        loop {
            let Some(base_descriptor) = BaseDescriptor::deserialize(&mut reader).eof_to_none()? else {
                break;
            };

            let Some(descr) = UnknownDescriptor::deserialize_seed(&mut reader, base_descriptor).eof_to_none()? else {
                break;
            };
            unknown_descriptors.push(descr);
        }

        Ok(Self {
            es_id,
            stream_priority,
            depends_on_es_id,
            url_string,
            ocr_es_id,
            dec_config_descr,
            sl_config_descr,
            unknown_descriptors,
        })
    }
}

impl Serialize for ESDescriptor<'_> {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);

        self.base_descriptor().serialize(&mut bit_writer)?;
        self.es_id.serialize(&mut bit_writer)?;
        bit_writer.write_bit(self.depends_on_es_id.is_some())?;
        bit_writer.write_bit(self.url_string.is_some())?;
        bit_writer.write_bit(self.ocr_es_id.is_some())?;
        bit_writer.write_bits(self.stream_priority as u64, 5)?;

        if let Some(depends_on_es_id) = self.depends_on_es_id {
            depends_on_es_id.serialize(&mut bit_writer)?;
        }

        if let Some(url_string) = &self.url_string {
            let url_length = url_string.len() as u8;
            url_length.serialize(&mut bit_writer)?;
            url_string.serialize(&mut bit_writer)?;
        }

        if let Some(ocr_es_id) = self.ocr_es_id {
            ocr_es_id.serialize(&mut bit_writer)?;
        }

        self.dec_config_descr.serialize(&mut bit_writer)?;

        if let Some(sl_config_descr) = &self.sl_config_descr {
            sl_config_descr.serialize(&mut bit_writer)?;
        }

        for unknown_descriptor in &self.unknown_descriptors {
            unknown_descriptor.serialize(&mut bit_writer)?;
        }

        Ok(())
    }
}

impl IsoSized for ESDescriptor<'_> {
    fn size(&self) -> usize {
        self.base_descriptor().size() + self.payload_size()
    }
}
