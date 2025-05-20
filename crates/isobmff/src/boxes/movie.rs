//! Movie structure boxes defined in ISO/IEC 14496-12 - 8.2

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use super::{MetaBox, MovieExtendsBox, TrackBox, UserDataBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Movie box
///
/// ISO/IEC 14496-12 - 8.2.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"moov", crate_path = crate)]
pub struct MovieBox<'a> {
    #[iso_box(nested_box)]
    pub mvhd: MovieHeaderBox,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub trak: Vec<TrackBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub mvex: Option<MovieExtendsBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Movie header box
///
/// ISO/IEC 14496-12 - 8.2.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mvhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct MovieHeaderBox {
    pub full_header: FullBoxHeader,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: i32,
    pub volume: i16,
    pub reserved1: u16,
    pub reserved2: u64,
    pub matrix: [i32; 9],
    pub pre_defined: [u32; 6],
    pub next_track_id: u32,
}

impl<'a> DeserializeSeed<'a, BoxHeader> for MovieHeaderBox {
    fn deserialize_seed<R>(mut reader: R, _seed: BoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let full_header = FullBoxHeader::deserialize(&mut reader)?;

        let creation_time = if full_header.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let modification_time = if full_header.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };
        let timescale = u32::deserialize(&mut reader)?;
        let duration = if full_header.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        let rate = i32::deserialize(&mut reader)?;
        let volume = i16::deserialize(&mut reader)?;

        let reserved1 = u16::deserialize(&mut reader)?;
        let reserved2 = u64::deserialize(&mut reader)?;

        let mut matrix = [0; 9];
        for m in &mut matrix {
            *m = i32::deserialize(&mut reader)?;
        }

        let mut pre_defined = [0; 6];
        for p in &mut pre_defined {
            *p = u32::deserialize(&mut reader)?;
        }

        let next_track_id = u32::deserialize(&mut reader)?;

        Ok(Self {
            full_header,
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            reserved1,
            reserved2,
            matrix,
            pre_defined,
            next_track_id,
        })
    }
}

impl Serialize for MovieHeaderBox {
    fn serialize<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.serialize_box_header(&mut writer)?;
        self.full_header.serialize(&mut writer)?;

        if self.full_header.version == 1 {
            self.creation_time.serialize(&mut writer)?;
            self.modification_time.serialize(&mut writer)?;
            self.timescale.serialize(&mut writer)?;
            self.duration.serialize(&mut writer)?;
        } else {
            (self.creation_time as u32).serialize(&mut writer)?;
            (self.modification_time as u32).serialize(&mut writer)?;
            self.timescale.serialize(&mut writer)?;
            (self.duration as u32).serialize(&mut writer)?;
        }

        self.rate.serialize(&mut writer)?;
        self.volume.serialize(&mut writer)?;
        self.reserved1.serialize(&mut writer)?;
        self.reserved2.serialize(&mut writer)?;
        self.matrix.serialize(&mut writer)?;
        self.pre_defined.serialize(&mut writer)?;
        self.next_track_id.serialize(writer)?;

        Ok(())
    }
}

impl IsoSized for MovieHeaderBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        if self.full_header.version == 1 {
            size += 8 + 8 + 4 + 8; // creation_time, modification_time, timescale, duration
        } else {
            size += 4 + 4 + 4 + 4; // creation_time, modification_time, timescale, duration
        }
        size += 4 // rate
            + 2 // volume
            + 2 // reserved1
            + 8 // reserved2
            + self.matrix.size()
            + self.pre_defined.size()
            + 4; // next_track_id

        Self::add_header_size(size)
    }
}
