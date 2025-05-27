use isobmff::IsoSized;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use super::{BaseDescriptor, DescriptorTag};

/// Profile Level Indication Index Descriptor
///
/// ISO/IEC 14496-1 - 7.2.6.20
#[derive(Debug, PartialEq, Eq)]
pub struct ProfileLevelIndicationIndexDescriptor {
    /// A unique identifier for the set of profile and level indications described
    /// in this descriptor within the name scope defined by the IOD.
    pub profile_level_indication_index: u8,
}

impl ProfileLevelIndicationIndexDescriptor {
    /// Returns the base descriptor of this `ProfileLevelIndicationIndexDescriptor`.
    pub fn base_descriptor(&self) -> BaseDescriptor {
        BaseDescriptor {
            tag: DescriptorTag::profileLevelIndicationIndexDescrTag,
            size_of_instance: self.payload_size() as u32,
        }
    }

    fn payload_size(&self) -> usize {
        self.profile_level_indication_index.size()
    }
}

impl<'a> Deserialize<'a> for ProfileLevelIndicationIndexDescriptor {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let base_descriptor = BaseDescriptor::deserialize(&mut reader)?;
        Self::deserialize_seed(reader, base_descriptor)
    }
}

impl<'a> DeserializeSeed<'a, BaseDescriptor> for ProfileLevelIndicationIndexDescriptor {
    fn deserialize_seed<R>(reader: R, seed: BaseDescriptor) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let reader = reader.take(seed.size_of_instance as usize);

        Ok(Self {
            profile_level_indication_index: u8::deserialize(reader)?,
        })
    }
}

impl Serialize for ProfileLevelIndicationIndexDescriptor {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.base_descriptor().serialize(&mut writer)?;
        self.profile_level_indication_index.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for ProfileLevelIndicationIndexDescriptor {
    fn size(&self) -> usize {
        self.base_descriptor().size() + self.payload_size()
    }
}
