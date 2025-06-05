//! Movie structure boxes defined in ISO/IEC 14496-12 - 8.2

use fixed::traits::ToFixed;
use fixed::types::extra::{U8, U16};
use fixed::{FixedI16, FixedI32};
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use super::{MetaBox, MovieExtendsBox, TrackBox, UserDataBox};
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Movie box
///
/// ISO/IEC 14496-12 - 8.2.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"moov", crate_path = crate)]
pub struct MovieBox<'a> {
    /// The contained [`MovieHeaderBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub mvhd: MovieHeaderBox,
    /// The contained [`MetaBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    /// The contained [`TrackBox`]es. (mandatory, at least one)
    #[iso_box(nested_box(collect))]
    pub trak: Vec<TrackBox<'a>>,
    /// The contained [`MovieExtendsBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub mvex: Option<MovieExtendsBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    /// The contained [`UserDataBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

/// Movie header box
///
/// ISO/IEC 14496-12 - 8.2.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mvhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct MovieHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that declares the creation time of the presentation (in seconds since
    /// midnight, Jan. 1, 1904, in UTC time).
    pub creation_time: u64,
    /// An integer that declares the most recent time the presentation was modified (in
    /// seconds since midnight, Jan. 1, 1904, in UTC time).
    pub modification_time: u64,
    /// An integer that specifies the time-scale for the entire presentation; this is the number of
    /// time units that pass in one second. For example, a time coordinate system that measures time in
    /// sixtieths of a second has a time scale of 60.
    pub timescale: u32,
    /// An integer that declares length of the presentation (in the indicated timescale). This property
    /// is derived from the presentation’s tracks: the value of this field corresponds to the duration of the
    /// longest track in the presentation. If the duration cannot be determined then duration is set to all 1s.
    pub duration: u64,
    /// Indicates the preferred rate to play the presentation; 1.0 is normal forward playback.
    pub rate: FixedI32<U16>,
    /// Indicates the preferred playback volume. 1.0 is full volume.
    pub volume: FixedI16<U8>,
    /// Reserved 16 bits, must be set to 0.
    pub reserved1: u16,
    /// Reserved 64 bits, must be set to 0.
    pub reserved2: u64,
    /// Provides a transformation matrix for the video; `(u,v,w)` are restricted here to `(0,0,1)`, hex values
    /// `(0,0,0x40000000)`.
    pub matrix: [i32; 9],
    /// Reserved 6 * 32 bits, must be set to 0.
    pub pre_defined: [u32; 6],
    /// Non-zero integer that indicates a value to use for the `track_ID` of the next track to
    /// be added to this presentation. Zero is not a valid `track_ID` value. The value of `next_track_ID` shall
    /// be larger than the largest `track_ID` in use. If this value is equal to all 1s ([`u32::MAX`]), and a new
    /// media track is to be added, then a search must be made in the file for an unused value of `track_ID`.
    pub next_track_id: u32,
}

impl MovieHeaderBox {
    /// Creates a new [`MovieHeaderBox`] with the specified parameters.
    ///
    /// All other fields are initialized to their default values.
    pub fn new(creation_time: u64, modification_time: u64, timescale: u32, duration: u64, next_track_id: u32) -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            creation_time,
            modification_time,
            timescale,
            duration,
            rate: 1.to_fixed(),
            volume: 1.to_fixed(),
            reserved1: 0,
            reserved2: 0,
            matrix: [0x00010000, 0, 0, 0, 0x00010000, 0, 0, 0, 0x40000000],
            pre_defined: [0; 6],
            next_track_id,
        }
    }
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

        let rate = FixedI32::from_bits(i32::deserialize(&mut reader)?);
        let volume = FixedI16::from_bits(i16::deserialize(&mut reader)?);

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

        self.rate.to_bits().serialize(&mut writer)?;
        self.volume.to_bits().serialize(&mut writer)?;
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
