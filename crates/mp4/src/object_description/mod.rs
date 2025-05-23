//! Object Description Framework
//!
//! ISO/IEC 14496-1 - 7.2

use isobmff::IsoSized;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCow;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

mod decoder_config;
mod es;
mod profile_level_indication_index;
mod sl_config;

pub use decoder_config::*;
pub use es::*;
pub use profile_level_indication_index::*;
pub use sl_config::*;

nutype_enum! {
    /// Class Tags for Descriptors
    ///
    /// ISO/IEC 14496-1 - 7.2.2.1
    pub enum DescriptorTag(u8) {
        /// Forbidden
        Forbidden = 0x00,
        /// ObjectDescrTag
        ObjectDescrTag = 0x01,
        /// InitialObjectDescrTag
        InitialObjectDescrTag = 0x02,
        /// ES_DescrTag
        ES_DescrTag = 0x03,
        /// DecoderConfigDescrTag
        DecoderConfigDescrTag = 0x04,
        /// DecSpecificInfoTag
        DecSpecificInfoTag = 0x05,
        /// SLConfigDescrTag
        SLConfigDescrTag = 0x06,
        /// ContentIdentDescrTag
        ContentIdentDescrTag = 0x07,
        /// SupplContentIdentDescrTag
        SupplContentIdentDescrTag = 0x08,
        /// IPI_DescrPointerTag
        IPI_DescrPointerTag = 0x09,
        /// IPMP_DescrPointerTag
        IPMP_DescrPointerTag = 0x0A,
        /// IPMP_DescrTag
        IPMP_DescrTag = 0x0B,
        /// QoS_DescrTag
        QoS_DescrTag = 0x0C,
        /// RegistrationDescrTag
        RegistrationDescrTag = 0x0D,
        /// ES_ID_IncTag
        ES_ID_IncTag = 0x0E,
        /// ES_ID_RefTag
        ES_ID_RefTag = 0x0F,
        /// MP4_IOD_Tag
        MP4_IOD_Tag = 0x10,
        /// MP4_OD_Tag
        MP4_OD_Tag = 0x11,
        /// IPL_DescrPointerRefTag
        IPL_DescrPointerRefTag = 0x12,
        /// ExtensionProfileLevelDescrTag
        ExtensionProfileLevelDescrTag = 0x13,
        /// profileLevelIndicationIndexDescrTag
        profileLevelIndicationIndexDescrTag = 0x14,
        /// ContentClassificationDescrTag
        ContentClassificationDescrTag = 0x40,
        /// KeyWordDescrTag
        KeyWordDescrTag = 0x41,
        /// RatingDescrTag
        RatingDescrTag = 0x42,
        /// LanguageDescrTag
        LanguageDescrTag = 0x43,
        /// ShortTextualDescrTag
        ShortTextualDescrTag = 0x44,
        /// ExpandedTextualDescrTag
        ExpandedTextualDescrTag = 0x45,
        /// ContentCreatorNameDescrTag
        ContentCreatorNameDescrTag = 0x46,
        /// ContentCreationDateDescrTag
        ContentCreationDateDescrTag = 0x47,
        /// OCICreatorNameDescrTag
        OCICreatorNameDescrTag = 0x48,
        /// OCICreationDateDescrTag
        OCICreationDateDescrTag = 0x49,
        /// SmpteCameraPositionDescrTag
        SmpteCameraPositionDescrTag = 0x4A,
        /// SegmentDescrTag
        SegmentDescrTag = 0x4B,
        /// MediaTimeDescrTag
        MediaTimeDescrTag = 0x4C,
        /// IPMP_ToolsListDescrTag
        IPMP_ToolsListDescrTag = 0x60,
        /// IPMP_ToolTag
        IPMP_ToolTag = 0x61,
        /// M4MuxTimingDescrTag
        M4MuxTimingDescrTag = 0x62,
        /// M4MuxCodeTableDescrTag
        M4MuxCodeTableDescrTag = 0x63,
        /// ExtSLConfigDescrTag
        ExtSLConfigDescrTag = 0x64,
        /// M4MuxBufferSizeDescrTag
        M4MuxBufferSizeDescrTag = 0x65,
        /// M4MuxIdentDescrTag
        M4MuxIdentDescrTag = 0x66,
        /// DependencyPointerTag
        DependencyPointerTag = 0x67,
        /// DependencyMarkerTag
        DependencyMarkerTag = 0x68,
        /// M4MuxChannelDescrTag
        M4MuxChannelDescrTag = 0x69,
        /// Forbidden2
        Forbidden2 = 0xFF,
    }
}

impl<'a> Deserialize<'a> for DescriptorTag {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        u8::deserialize(reader).map(Into::into)
    }
}

impl Serialize for DescriptorTag {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.0.serialize(writer)
    }
}

impl IsoSized for DescriptorTag {
    fn size(&self) -> usize {
        1
    }
}

/// Base Descriptor
///
/// ISO/IEC 14496-1 - 7.2.2.2
#[derive(Debug)]
pub struct BaseDescriptor {
    /// Descriptor tag
    pub tag: DescriptorTag,
    /// Number of bytes used to encode the descriptor, excluding the `tag` and
    /// `size_of_instance` fields (defined here).
    ///
    /// Defined for expandable classes.
    /// Every class inhereting from (i.e. containing) `BaseDescriptor` is an expandable class.
    ///
    /// See ISO/IEC 14496-1 - 8.3.3
    pub size_of_instance: u32,
}

impl<'a> Deserialize<'a> for BaseDescriptor {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let tag = DescriptorTag::deserialize(&mut reader)?;

        let mut size_of_instance = 0;
        loop {
            let byte = u8::deserialize(&mut reader)?;
            size_of_instance = (size_of_instance << 7) | (byte & 0b0111_1111) as u32;

            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }

        Ok(Self { tag, size_of_instance })
    }
}

impl Serialize for BaseDescriptor {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.tag.serialize(&mut writer)?;

        let bytes = (32 - self.size_of_instance.leading_zeros()).div_ceil(7);
        for i in (0..bytes).rev() {
            let mut byte = ((self.size_of_instance >> (i * 7)) & 0b0111_1111) as u8;
            if i != 0 {
                byte |= 0b1000_0000;
            }
            byte.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for BaseDescriptor {
    fn size(&self) -> usize {
        let mut size = 1; // tag
        size += (32 - self.size_of_instance.leading_zeros() as usize).div_ceil(7);
        size
    }
}

/// Any descriptor that inherits [`BaseDescriptor`].
#[derive(Debug)]
pub struct UnknownDescriptor<'a> {
    /// The base descriptor.
    pub base_descriptor: BaseDescriptor,
    /// The data of the descriptor.
    ///
    /// Length is `base_descriptor.size_of_instance`.
    pub data: BytesCow<'a>,
}

impl<'a> UnknownDescriptor<'a> {
    /// Create a new [`UnknownDescriptor`] from a given [`DescriptorTag`] and data.
    pub fn new(tag: DescriptorTag, data: BytesCow<'a>) -> Self {
        Self {
            base_descriptor: BaseDescriptor {
                tag,
                size_of_instance: data.len() as u32,
            },
            data,
        }
    }
}

impl<'a> Deserialize<'a> for UnknownDescriptor<'a> {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let base_descriptor = BaseDescriptor::deserialize(&mut reader)?;
        Self::deserialize_seed(reader, base_descriptor)
    }
}

impl<'a> DeserializeSeed<'a, BaseDescriptor> for UnknownDescriptor<'a> {
    fn deserialize_seed<R>(mut reader: R, seed: BaseDescriptor) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let data = reader.try_read(seed.size_of_instance as usize)?;
        Ok(Self {
            base_descriptor: seed,
            data,
        })
    }
}

impl Serialize for UnknownDescriptor<'_> {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.base_descriptor.serialize(&mut writer)?;
        self.data.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for UnknownDescriptor<'_> {
    fn size(&self) -> usize {
        self.base_descriptor.size() + self.data.size()
    }
}
