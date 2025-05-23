//! Track structure boxes defined in ISO/IEC 14496-12 - 8.3

use fixed::traits::ToFixed;
use fixed::types::extra::{U8, U16};
use fixed::{FixedI16, FixedU32};
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be};

use super::{Brand, EditBox, MediaBox, MetaBox, UserDataBox};
use crate::{BoxHeader, IsoBox, IsoSized, UnknownBox};

/// Track box
///
/// ISO/IEC 14496-12 - 8.3.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trak", crate_path = crate)]
pub struct TrackBox<'a> {
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

impl<'a> TrackBox<'a> {
    pub fn new(tkhd: TrackHeaderBox, edts: Option<EditBox>, mdia: MediaBox<'a>) -> Self {
        Self {
            tkhd,
            tref: None,
            trgr: None,
            edts,
            ttyp: None,
            meta: None,
            mdia,
            unknown_boxes: vec![],
            udta: None,
        }
    }
}

bitflags::bitflags! {
    /// Track header box flags
    #[derive(Debug, Clone, Copy)]
    pub struct TrackHeaderBoxFlags: u32 {
        const TrackEnabled = 0x000001;
        const TrackInMovie = 0x000002;
        const TrackInPreview = 0x000004;
        const TrackSizeIsAspectRatio = 0x000008;
    }
}

impl<'a> Deserialize<'a> for TrackHeaderBoxFlags {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let flags = U24Be::deserialize(reader)?;
        Ok(Self::from_bits_truncate(*flags))
    }
}

impl Serialize for TrackHeaderBoxFlags {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.bits().serialize(writer)
    }
}

impl IsoSized for TrackHeaderBoxFlags {
    fn size(&self) -> usize {
        3
    }
}

/// Track header box
///
/// ISO/IEC 14496-12 - 8.3.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"tkhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackHeaderBox {
    // full header:
    pub version: u8,
    pub flags: TrackHeaderBoxFlags,
    // body:
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub reserved1: u32,
    pub duration: u64,
    pub reserved2: u64,
    pub layer: i16,
    pub alternate_group: i16,
    pub volume: FixedI16<U8>,
    pub reserved3: u16,
    pub matrix: [i32; 9],
    pub width: FixedU32<U16>,
    pub height: FixedU32<U16>,
}

impl TrackHeaderBox {
    pub fn new(
        creation_time: u64,
        modification_time: u64,
        track_id: u32,
        duration: u64,
        dimensions: Option<(u32, u32)>,
    ) -> Self {
        let version = if creation_time > u32::MAX as u64 || modification_time > u32::MAX as u64 || duration > u32::MAX as u64
        {
            1
        } else {
            0
        };

        let (width, height) = dimensions.unwrap_or((0, 0));
        let volume = if dimensions.is_some() { 0 } else { 1 };

        Self {
            version,
            flags: TrackHeaderBoxFlags::TrackEnabled | TrackHeaderBoxFlags::TrackInMovie,
            creation_time,
            modification_time,
            track_id,
            reserved1: 0,
            duration,
            reserved2: 0,
            layer: 0,
            alternate_group: 0,
            volume: volume.to_fixed(),
            reserved3: 0,
            matrix: [0x00010000, 0, 0, 0, 0x00010000, 0, 0, 0, 0x40000000],
            width: width.to_fixed(),
            height: height.to_fixed(),
        }
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for TrackHeaderBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let version = u8::deserialize(&mut reader)?;
        let flags = TrackHeaderBoxFlags::deserialize(&mut reader)?;

        let creation_time = if version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let modification_time = if version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let track_id = u32::deserialize(&mut reader)?;
        let reserved1 = u32::deserialize(&mut reader)?;
        let duration = if version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        let reserved2 = u64::deserialize(&mut reader)?;

        let layer = i16::deserialize(&mut reader)?;
        let alternate_group = i16::deserialize(&mut reader)?;
        let volume = FixedI16::from_bits(i16::deserialize(&mut reader)?);

        let reserved3 = u16::deserialize(&mut reader)?;

        let mut matrix = [0; 9];
        for m in &mut matrix {
            *m = i32::deserialize(&mut reader)?;
        }

        let width = FixedU32::from_bits(u32::deserialize(&mut reader)?);
        let height = FixedU32::from_bits(u32::deserialize(&mut reader)?);

        Ok(Self {
            version,
            flags,
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
        self.serialize_box_header(&mut writer)?;
        self.version.serialize(&mut writer)?;
        self.flags.serialize(&mut writer)?;

        if self.version == 1 {
            self.creation_time.serialize(&mut writer)?;
            self.modification_time.serialize(&mut writer)?;
        } else {
            (self.creation_time as u32).serialize(&mut writer)?;
            (self.modification_time as u32).serialize(&mut writer)?;
        }
        self.track_id.serialize(&mut writer)?;
        self.reserved1.serialize(&mut writer)?;
        if self.version == 1 {
            self.duration.serialize(&mut writer)?;
        } else {
            (self.duration as u32).serialize(&mut writer)?;
        }

        self.reserved2.serialize(&mut writer)?;

        self.layer.serialize(&mut writer)?;
        self.alternate_group.serialize(&mut writer)?;
        self.volume.to_bits().serialize(&mut writer)?;

        self.reserved3.serialize(&mut writer)?;

        self.matrix.serialize(&mut writer)?;

        self.width.to_bits().serialize(&mut writer)?;
        self.height.to_bits().serialize(&mut writer)?;

        Ok(())
    }
}

impl IsoSized for TrackHeaderBox {
    fn size(&self) -> usize {
        let mut size = self.version.size() + self.flags.size();
        if self.version == 1 {
            size += 8 + 8; // creation_time, modification_time
        } else {
            size += 4 + 4; // creation_time, modification_time
        }
        size += 4; // track_id
        size += 4; // reserved1
        if self.version == 1 {
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

        Self::add_header_size(size)
    }
}

/// Track reference box
///
/// ISO/IEC 14496-12 - 8.3.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"tref", crate_path = crate)]
pub struct TrackReferenceBox<'a> {
    #[iso_box(nested_box(collect_unknown))]
    pub track_reference_type: Vec<UnknownBox<'a>>,
}

/// Track group box
///
/// ISO/IEC 14496-12 - 8.3.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trgr", crate_path = crate)]
pub struct TrackGroupBox<'a> {
    #[iso_box(nested_box(collect_unknown))]
    pub track_group_type: Vec<UnknownBox<'a>>,
}

/// Track type box
///
/// ISO/IEC 14496-12 - 8.3.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"ttyp", crate_path = crate)]
pub struct TrackTypeBox {
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    pub minor_version: u32,
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
