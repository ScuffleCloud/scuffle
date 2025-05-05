//! Movie structure boxes defined in ISO/IEC 14496-12 - 8.2

use byteorder::ReadBytesExt;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use super::{MovieExtendsBox, TrackBox, UserDataBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox};

/// Movie box
///
/// ISO/IEC 14496-12 - 8.2.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"moov", crate_path = crate)]
pub struct MovieBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub mvhd: MovieHeaderBox,
    #[iso_box(nested_box(collect))]
    pub trak: Vec<TrackBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub mvex: Option<MovieExtendsBox>,
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Movie header box
///
/// ISO/IEC 14496-12 - 8.2.2
#[derive(Debug)]
pub struct MovieHeaderBox {
    pub header: FullBoxHeader,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: i32,
    pub volume: i16,
    pub matrix: [i32; 9],
    pub pre_defined: [u32; 6],
    pub next_track_id: u32,
}

// Manual implementation because conditional fields are not supported in the macro

impl IsoBox for MovieHeaderBox {
    type Header = FullBoxHeader;

    const TYPE: [u8; 4] = *b"mvhd";
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for MovieHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let creation_time = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };
        let modification_time = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };
        let timescale = reader.as_std().read_u32::<byteorder::BigEndian>()?;
        let duration = if seed.version == 1 {
            reader.as_std().read_u64::<byteorder::BigEndian>()?
        } else {
            reader.as_std().read_u32::<byteorder::BigEndian>()? as u64
        };

        let rate = reader.as_std().read_i32::<byteorder::BigEndian>()?;
        let volume = reader.as_std().read_i16::<byteorder::BigEndian>()?;

        reader.as_std().read_u16::<byteorder::BigEndian>()?; // reserved
        reader.as_std().read_u64::<byteorder::BigEndian>()?; // reserved

        let mut matrix = [0; 9];
        for m in &mut matrix {
            *m = reader.as_std().read_i32::<byteorder::BigEndian>()?;
        }

        let mut pre_defined = [0; 6];
        for p in &mut pre_defined {
            *p = reader.as_std().read_u32::<byteorder::BigEndian>()?;
        }

        let next_track_id = reader.as_std().read_u32::<byteorder::BigEndian>()?;

        Ok(Self {
            header: seed,
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            matrix,
            pre_defined,
            next_track_id,
        })
    }
}

impl<'a> Deserialize<'a> for MovieHeaderBox {
    fn deserialize<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}
