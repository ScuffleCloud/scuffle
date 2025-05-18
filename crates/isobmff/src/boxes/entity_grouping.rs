use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Groups list box
///
/// ISO/IEC 14496-12 - 8.18.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"grpl", crate_path = crate)]
pub struct GroupsListBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub altr: Vec<AltrEntityToGroupBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Entity to group box
///
/// ISO/IEC 14496-12 - 8.18.3
#[derive(Debug)]
pub struct EntityToGroupBox {
    pub group_id: u32,
    pub num_entities_in_group: u32,
    pub entity_id: Vec<u32>,
}

impl<'a> Deserialize<'a> for EntityToGroupBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let group_id = u32::deserialize(&mut reader)?;
        let num_entities_in_group = u32::deserialize(&mut reader)?;

        let mut entity_id = Vec::with_capacity(num_entities_in_group as usize);
        for _ in 0..num_entities_in_group {
            entity_id.push(u32::deserialize(&mut reader)?);
        }

        Ok(EntityToGroupBox {
            group_id,
            num_entities_in_group,
            entity_id,
        })
    }
}

impl Serialize for EntityToGroupBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.group_id.serialize(&mut writer)?;
        self.num_entities_in_group.serialize(&mut writer)?;

        for id in &self.entity_id {
            id.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl IsoSized for EntityToGroupBox {
    fn size(&self) -> usize {
        self.group_id.size() + self.num_entities_in_group.size() + self.entity_id.size()
    }
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"altr", crate_path = crate)]
pub struct AltrEntityToGroupBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entity_to_group: EntityToGroupBox,
}
