use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use crate::{FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Groups list box
///
/// ISO/IEC 14496-12 - 8.18.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"grpl", crate_path = crate)]
pub struct GroupsListBox<'a> {
    /// The contained [`AltrEntityToGroupBox`]es. (one or more)
    #[iso_box(nested_box(collect))]
    pub altr: Vec<AltrEntityToGroupBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// General entity to group box
///
/// ISO/IEC 14496-12 - 8.18.3
#[derive(Debug, PartialEq, Eq)]
pub struct EntityToGroupBox {
    /// A non-negative integer assigned to the particular grouping that shall not be equal to any
    /// `group_id` value of any other [`EntityToGroupBox`], any `item_ID` value of the hierarchy level (file, movie
    /// or track) that contains the [`GroupsListBox`], or any `track_ID` value (when the [`GroupsListBox`] is
    /// contained in the file level).
    pub group_id: u32,
    /// Specifies the number of `entity_id` values mapped to this entity group.
    pub num_entities_in_group: u32,
    /// Resolved to an item, when an item with `item_ID` equal to `entity_id` is present in the
    /// hierarchy level (file, movie or track) that contains the [`GroupsListBox`], or to a track, when a track
    /// with `track_ID` equal to `entity_id` is present and the [`GroupsListBox`] is contained in the file level.
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

/// 'altr' entity to group box
///
/// ISO/IEC 14496-12 - 8.18.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"altr", crate_path = crate)]
pub struct AltrEntityToGroupBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// The contained [`EntityToGroupBox`].
    pub entity_to_group: EntityToGroupBox,
}
