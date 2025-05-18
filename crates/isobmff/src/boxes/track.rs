//! Track structure boxes defined in ISO/IEC 14496-12 - 8.3

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use super::{Brand, EditBox, MediaBox, MetaBox, UserDataBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Track box
///
/// ISO/IEC 14496-12 - 8.3.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trak", crate_path = crate)]
pub struct TrackBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub tkhd: TrackHeaderBox,
    #[iso_box(nested_box(collect))]
    pub tref: Option<TrackReferenceBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub trgr: Option<TrackGroupBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub edts: Option<EditBox>,
    #[iso_box(nested_box(collect))]
    pub ttyp: Option<TrackTypeBox>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    #[iso_box(nested_box)]
    pub mdia: MediaBox<'a>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Track header box
///
/// ISO/IEC 14496-12 - 8.3.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"tkhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub reserved1: u32,
    pub duration: u64,
    pub reserved2: u64,
    pub layer: i16,
    pub alternate_group: i16,
    pub volume: i16,
    pub reserved3: u16,
    pub matrix: [i32; 9],
    pub width: u32,
    pub height: u32,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let creation_time = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let modification_time = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let track_id = u32::deserialize(&mut reader)?;
        let reserved1 = u32::deserialize(&mut reader)?;
        let duration = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        let reserved2 = u64::deserialize(&mut reader)?;

        let layer = i16::deserialize(&mut reader)?;
        let alternate_group = i16::deserialize(&mut reader)?;
        let volume = i16::deserialize(&mut reader)?;

        let reserved3 = u16::deserialize(&mut reader)?;

        let mut matrix = [0; 9];
        for m in &mut matrix {
            *m = i32::deserialize(&mut reader)?;
        }

        let width = u32::deserialize(&mut reader)?;
        let height = u32::deserialize(&mut reader)?;

        Ok(Self {
            header: seed,
            creation_time,
            modification_time,
            track_id,
            reserved1,
            duration,
            reserved2,
            layer,
            alternate_group,
            volume,
            reserved3,
            matrix,
            width,
            height,
        })
    }
}

impl Serialize for TrackHeaderBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.header.serialize(&mut writer)?;

        if self.header.version == 1 {
            self.creation_time.serialize(&mut writer)?;
            self.modification_time.serialize(&mut writer)?;
        } else {
            (self.creation_time as u32).serialize(&mut writer)?;
            (self.modification_time as u32).serialize(&mut writer)?;
        }
        self.track_id.serialize(&mut writer)?;
        self.reserved1.serialize(&mut writer)?;
        if self.header.version == 1 {
            self.duration.serialize(&mut writer)?;
        } else {
            (self.duration as u32).serialize(&mut writer)?;
        }

        self.reserved2.serialize(&mut writer)?;

        self.layer.serialize(&mut writer)?;
        self.alternate_group.serialize(&mut writer)?;
        self.volume.serialize(&mut writer)?;

        self.reserved3.serialize(&mut writer)?;

        self.matrix.serialize(&mut writer)?;

        self.width.serialize(&mut writer)?;
        self.height.serialize(&mut writer)?;

        Ok(())
    }
}

impl IsoSized for TrackHeaderBox {
    fn size(&self) -> usize {
        let mut size = self.header.size();
        if self.header.version == 1 {
            size += 8 + 8; // creation_time, modification_time
        } else {
            size += 4 + 4; // creation_time, modification_time
        }
        size += 4; // track_id
        size += 4; // reserved1
        if self.header.version == 1 {
            size += 8; // duration
        } else {
            size += 4; // duration
        }
        size += 8; // reserved2
        size += 2; // layer
        size += 2; // alternate_group
        size += 2; // volume
        size += 2; // reserved3
        size += self.matrix.size(); // matrix
        size += 4; // width
        size += 4; // height

        size
    }
}

/// Track reference box
///
/// ISO/IEC 14496-12 - 8.3.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"tref", crate_path = crate)]
pub struct TrackReferenceBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect_unknown))]
    pub track_reference_type: Vec<UnknownBox<'a>>,
}

/// Track group box
///
/// ISO/IEC 14496-12 - 8.3.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trgr", crate_path = crate)]
pub struct TrackGroupBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect_unknown))]
    pub track_group_type: Vec<UnknownBox<'a>>,
}

/// Track type box
///
/// ISO/IEC 14496-12 - 8.3.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ttyp", crate_path = crate)]
pub struct TrackTypeBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
