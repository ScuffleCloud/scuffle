//! Track media structure boxes defined in ISO/IEC 14496-12 - 8.4

use nutype_enum::nutype_enum;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize, U24Be};

use super::{
    DataInformationBox, HintMediaHeaderBox, SampleTableBox, SoundMediaHeaderBox, SubtitleMediaHeaderBox,
    VideoMediaHeaderBox, VolumetricVisualMediaHeaderBox,
};
use crate::common_types::Utf8String;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, Langauge, UnknownBox};

/// Media box
///
/// ISO/IEC 14496-12 - 8.4.1
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mdia", crate_path = crate)]
pub struct MediaBox<'a> {
    /// The contained [`MediaHeaderBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub mdhd: MediaHeaderBox,
    /// The contained [`HandlerBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    /// The optional [`ExtendedLanguageBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub elng: Option<ExtendedLanguageBox>,
    /// The contained [`MediaInformationBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub minf: MediaInformationBox<'a>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

impl<'a> MediaBox<'a> {
    /// Creates a new `MediaBox` with the given `mdhd`, `hdlr`, and `minf`.
    pub fn new(mdhd: MediaHeaderBox, hdlr: HandlerBox, minf: MediaInformationBox<'a>) -> Self {
        Self {
            mdhd,
            hdlr,
            elng: None,
            minf,
            unknown_boxes: vec![],
        }
    }
}

/// Media header box
///
/// ISO/IEC 14496-12 - 8.4.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"mdhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct MediaHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that declares the creation time of the media in this track (in seconds since
    /// midnight, Jan. 1, 1904, in UTC time).
    pub creation_time: u64,
    /// An integer that declares the most recent time the media in this track was modified
    /// (in seconds since midnight, Jan. 1, 1904, in UTC time).
    pub modification_time: u64,
    /// An integer that specifies the number of time units that pass in one second for this media.
    /// For example, a time coordinate system that measures time in sixtieths of a second has a time scale
    /// of 60.
    pub timescale: u32,
    /// An integer that declares the duration of this media (in the scale of the timescale) and should
    /// be the largest composition timestamp plus the duration of that sample. If the duration cannot be
    /// determined then duration is set to all 1s.
    pub duration: u64,
    /// Declares the language code for this media, as a packed three-character code defined in
    /// ISO 639-2.
    pub language: Langauge,
    /// Pre-defined 16 bits, must be set to 0.
    pub pre_defined: u16,
}

impl MediaHeaderBox {
    /// Creates a new [`MediaHeaderBox`] with the specified parameters.
    ///
    /// All other fields are set to their default values.
    pub fn new(creation_time: u64, modification_time: u64, timescale: u32, duration: u64) -> Self {
        let version = if creation_time > u32::MAX as u64 || modification_time > u32::MAX as u64 || duration > u32::MAX as u64
        {
            1
        } else {
            0
        };

        Self {
            full_header: FullBoxHeader {
                version,
                flags: U24Be(0),
            },
            creation_time,
            modification_time,
            timescale,
            duration,
            language: Langauge::UNDETERMINED,
            pre_defined: 0,
        }
    }
}

impl<'a> DeserializeSeed<'a, BoxHeader> for MediaHeaderBox {
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

        let language = Langauge::deserialize(&mut reader)?;
        let pre_defined = u16::deserialize(&mut reader)?;

        Ok(Self {
            full_header,
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
            pre_defined,
        })
    }
}

impl Serialize for MediaHeaderBox {
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

        self.language.serialize(&mut writer)?;
        self.pre_defined.serialize(&mut writer)?;

        Ok(())
    }
}

impl IsoSized for MediaHeaderBox {
    fn size(&self) -> usize {
        let mut size = self.full_header.size();
        if self.full_header.version == 1 {
            size += 8 + 8 + 4 + 8; // creation_time, modification_time, timescale, duration
        } else {
            size += 4 + 4 + 4 + 4; // creation_time, modification_time, timescale, duration
        }
        size += self.language.size(); // language
        size += 2; // pre_defined

        Self::add_header_size(size)
    }
}

nutype_enum! {
    /// Handler type as defined in ISO/IEC 14496-12 - 12.
    pub enum HandlerType([u8; 4]) {
        /// `null`
        Null = *b"null",
        /// `vide`
        Video = *b"vide",
        /// `auxv`
        AuxiliaryVideo = *b"auxv",
        /// `soun`
        Audio = *b"soun",
        /// `meta`
        Metadata = *b"meta",
        /// `mp7t`
        MetadataMpeg7t = *b"mp7t",
        /// `mp7b`
        MetadataMpeg7b = *b"mp7b",
        /// `hint`
        Hint = *b"hint",
        /// `text`
        Text = *b"text",
        /// `subt`
        Subtitle = *b"subt",
        /// `fdsm`
        Font = *b"fdsm",
        /// `volv`
        VolumetricVisual = *b"volv",
        /// `hapt`
        Haptic = *b"hapt",
    }
}

impl IsoSized for HandlerType {
    fn size(&self) -> usize {
        4
    }
}

/// Handler reference box
///
/// ISO/IEC 14496-12 - 8.4.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"hdlr", crate_path = crate)]
pub struct HandlerBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Pre-defined 32 bits, must be set to 0.
    pub pre_defined: u32,
    /// - When present in a [`MediaBox`], contains a value as defined in Clause 12, or a value from a derived
    ///   specification, or registration.
    /// - When present in a [`MetaBox`](super::MetaBox), contains an appropriate value to indicate the format of the
    ///   [`MetaBox`](super::MetaBox) contents. The value 'null' can be used in the primary [`MetaBox`](super::MetaBox)
    ///   to indicate that it is merely being used to hold resources.
    #[iso_box(from = "[u8; 4]")]
    pub handler_type: HandlerType,
    /// Reserved 64 bits, must be set to 0.
    pub reserved: [u32; 3],
    /// Gives a human-readable name for the track type (for debugging and inspection purposes).
    pub name: Utf8String,
}

impl HandlerBox {
    /// Creates a new [`HandlerBox`] with the specified `handler_type` and `name`.
    pub fn new(handler_type: HandlerType, name: Utf8String) -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            pre_defined: 0,
            handler_type,
            reserved: [0; 3],
            name,
        }
    }
}

/// Media information box
///
/// ISO/IEC 14496-12 - 8.4.4
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"minf", crate_path = crate)]
pub struct MediaInformationBox<'a> {
    /// The optional [`VideoMediaHeaderBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub vmhd: Option<VideoMediaHeaderBox>,
    /// The optional [`SoundMediaHeaderBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub smhd: Option<SoundMediaHeaderBox>,
    /// The optional [`HintMediaHeaderBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub hmhd: Option<HintMediaHeaderBox>,
    /// The optional [`SubtitleMediaHeaderBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub sthd: Option<SubtitleMediaHeaderBox>,
    /// The optional [`VolumetricVisualMediaHeaderBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub vvhd: Option<VolumetricVisualMediaHeaderBox>,
    /// A list of unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    /// The contained [`DataInformationBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub dinf: DataInformationBox<'a>,
    /// The contained [`SampleTableBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub stbl: SampleTableBox<'a>,
}

impl<'a> MediaInformationBox<'a> {
    /// Creates a new [`MediaInformationBox`] with the given `stbl`, `vmhd`, and `smhd`.
    pub fn new(stbl: SampleTableBox<'a>, vmhd: Option<VideoMediaHeaderBox>, smhd: Option<SoundMediaHeaderBox>) -> Self {
        Self {
            vmhd,
            smhd,
            hmhd: None,
            sthd: None,
            vvhd: None,
            unknown_boxes: Vec::new(),
            dinf: DataInformationBox::default(),
            stbl,
        }
    }
}

/// Null media header box
///
/// ISO/IEC 14496-12 - 8.4.5.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"nmhd", crate_path = crate)]
pub struct NullMediaHeaderBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
}

/// Extended language tag
///
/// ISO/IEC 14496-12 - 8.4.6
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"elng", crate_path = crate)]
pub struct ExtendedLanguageBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// Contains an IETF BCP 47 compliant language tag string, such as "en-US", "fr-FR", or
    /// "zh-CN".
    pub extended_language: Utf8String,
}
