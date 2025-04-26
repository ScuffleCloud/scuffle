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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"trak", crate_path = crate)]
pub struct TrackBox<'a> {
    /// The contained [`TrackHeaderBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub tkhd: TrackHeaderBox,
    /// The contained [`TrackReferenceBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub tref: Option<TrackReferenceBox<'a>>,
    /// The contained [`TrackGroupBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub trgr: Option<TrackGroupBox<'a>>,
    /// The contained [`EditBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub edts: Option<EditBox>,
    /// The contained [`TrackTypeBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub ttyp: Option<TrackTypeBox>,
    /// The contained [`MetaBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    /// The contained [`MediaBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub mdia: MediaBox<'a>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    /// The contained [`UserDataBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub udta: Option<UserDataBox<'a>>,
}

impl<'a> TrackBox<'a> {
    /// Creates a new [`TrackBox`] with the given `tkhd`, optional `edts`, and `mdia`.
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
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct TrackHeaderBoxFlags: u32 {
        /// If this flag is set, it indicates that the track is enabled.
        /// A disabled track (when this flag is not set) is treated as if it were not present.
        const TrackEnabled = 0x000001;
        /// If this flag is set, it indicates that the track, or one of its
        /// alternatives (if any) forms a direct part of the presentation.
        /// If this flag is unset, it indicates that the track does not represent a direct part of the presentation.
        const TrackInMovie = 0x000002;
        /// This flag currently has no assigned meaning, and the
        /// value should be ignored by readers. In the absence of further guidance (e.g. from derived
        /// specifications), the same value as for [`Self::TrackInMovie`] should be written.
        const TrackInPreview = 0x000004;
        /// If this flag is set, it indicates that the width and
        /// height fields are not expressed in pixel units. The values have the same units but these units
        /// are not specified. The values are only an indication of the desired aspect ratio. If the aspect
        /// ratios of this track and other related tracks are not identical, then the respective positioning of
        /// the tracks is undefined, possibly defined by external contexts.
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
        U24Be(self.bits()).serialize(writer)
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tkhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct TrackHeaderBox {
    // full header:
    /// An integer that specifies the version of this box.
    ///
    /// Part of full box header.
    pub version: u8,
    /// The flags for this box.
    ///
    /// Part of full box header.
    pub flags: TrackHeaderBoxFlags,
    // body:
    /// An integer that declares the creation time of this track (in seconds since midnight,
    /// Jan. 1, 1904, in UTC time).
    pub creation_time: u64,
    /// An integer that declares the most recent time the track was modified (in seconds
    /// since midnight, Jan. 1, 1904, in UTC time).
    pub modification_time: u64,
    /// An integer that uniquely identifies this track over the entire life-time of this presentation;
    /// `track_ID`s are never re-used and cannot be zero.
    pub track_id: u32,
    /// Reserved 32 bits, must be set to 0.
    pub reserved1: u32,
    /// An integer that indicates the duration of this track (in the timescale indicated in the
    /// [`MovieHeaderBox`](super::MovieHeaderBox)) This duration field may be indefinite (all 1s) when either there is no edit list
    /// and the [`MediaHeaderBox`](super::MediaHeaderBox) duration is indefinite (i.e. all 1s), or when an indefinitely repeated edit
    /// list is desired (see subclause 8.6.6 for repeated edits). If there is no edit list and the duration is
    /// not indefinite, then the duration shall be equal to the media duration given in the [`MediaHeaderBox`](super::MediaHeaderBox),
    /// converted into the timescale in the [`MovieHeaderBox`](super::MovieHeaderBox). Otherwise the value of this field is equal to the
    /// sum of the durations of all of the track’s edits (possibly including repetitions).
    pub duration: u64,
    /// Reserved 64 bits, must be set to 0.
    pub reserved2: u64,
    /// Specifies the front-to-back ordering of video tracks; tracks with lower numbers are closer to the
    /// viewer. 0 is the normal value, and -1 would be in front of track 0, and so on
    pub layer: i16,
    /// An integer that specifies a group or collection of tracks. If this field is 0 there is
    /// no information on possible relations to other tracks. If this field is not 0, it should be the same for
    /// tracks that contain alternate data for one another and different for tracks belonging to different
    /// such groups. Only one track within an alternate group should be played or streamed at any one
    /// time, and shall be distinguishable from other tracks in the group via attributes such as bitrate,
    /// codec, language, packet size etc. A group may have only one member.
    pub alternate_group: i16,
    /// Specifies the track's relative audio volume. Full volume is 1.0
    /// and is the normal value. Its value is irrelevant for a purely visual track. Tracks may be composed
    /// by combining them according to their volume, and then using the overall [`MovieHeaderBox`](super::MovieHeaderBox) volume
    /// setting; or more complex audio composition (e.g. MPEG-4 BIFS) may be used.
    pub volume: FixedI16<U8>,
    /// Reserved 16 bits, must be set to 0.
    pub reserved3: u16,
    /// Provides a transformation matrix for the video; `(u,v,w)` are restricted here to `(0,0,1)`, hex
    /// `(0,0,0x40000000)`.
    pub matrix: [i32; 9],
    /// [`width`](Self::width) and [`height`](Self::height) are track-dependent as follows:
    ///
    /// For text and subtitle tracks, they may, depending on the coding format, describe the suggested size of
    /// the rendering area. For such tracks, the value `0x0` may also be used to indicate that the data may
    /// be rendered at any size, that no preferred size has been indicated and that the actual size may be
    /// determined by the external context or by reusing the width and height of another track. For those
    /// tracks, the flag [`TrackSizeIsAspectRatio`](TrackHeaderBoxFlags::TrackSizeIsAspectRatio) may also be used.
    ///
    /// For non-visual tracks (e.g. audio), they should be set to zero.
    ///
    /// For all other tracks, they specify the track's visual presentation size. These need not be the same as the
    /// pixel dimensions of the images, which is documented in the sample description(s); all images in the
    /// sequence are scaled to this size, before any overall transformation of the track represented by the
    /// matrix. The pixel dimensions of the images are the default values.
    pub width: FixedU32<U16>,
    /// See [`width`](Self::width).
    pub height: FixedU32<U16>,
}

impl TrackHeaderBox {
    /// Creates a new [`TrackHeaderBox`] with the specified parameters.
    ///
    /// All other fields are initialized to their default values.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"tref", crate_path = crate)]
pub struct TrackReferenceBox<'a> {
    /// Any unknown `TrackReferenceTypeBox`es that are contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub track_reference_type: Vec<UnknownBox<'a>>,
}

/// Track group box
///
/// ISO/IEC 14496-12 - 8.3.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"trgr", crate_path = crate)]
pub struct TrackGroupBox<'a> {
    /// Any unknown `TrackGroupTypeBox`es that are contained in this box.
    #[iso_box(nested_box(collect_unknown))]
    pub track_group_type: Vec<UnknownBox<'a>>,
}

/// Track type box
///
/// ISO/IEC 14496-12 - 8.3.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"ttyp", crate_path = crate)]
pub struct TrackTypeBox {
    /// The "best use" brand of the file which will provide the greatest compatibility.
    #[iso_box(from = "[u8; 4]")]
    pub major_brand: Brand,
    /// Minor version of the major brand.
    pub minor_version: u32,
    /// A list of compatible brands.
    #[iso_box(repeated, from = "[u8; 4]")]
    pub compatible_brands: Vec<Brand>,
}
