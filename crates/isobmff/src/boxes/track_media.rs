//! Track media structure boxes defined in ISO/IEC 14496-12 - 8.4

use nutype_enum::nutype_enum;
use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, Serialize};

use super::{
    DataInformationBox, HintMediaHeaderBox, SampleTableBox, SoundMediaHeaderBox, SubtitleMediaHeaderBox,
    VideoMediaHeaderBox, VolumetricVisualMediaHeaderBox,
};
use crate::common_types::Utf8String;
use crate::{BoxHeader, FullBoxHeader, IsoBox, IsoSized, Langauge, UnknownBox};

/// Media box
///
/// ISO/IEC 14496-12 - 8.4.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mdia", crate_path = crate)]
pub struct MediaBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub mdhd: MediaHeaderBox,
    #[iso_box(nested_box)]
    pub hdlr: HandlerBox,
    #[iso_box(nested_box)]
    pub minf: MediaInformationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub elng: Option<ExtendedLanguageBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Media header box
///
/// ISO/IEC 14496-12 - 8.4.2
#[derive(Debug, IsoBox)]
#[iso_box(box_type = b"mdhd", skip_impl(deserialize_seed, serialize, sized), crate_path = crate)]
pub struct MediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: Langauge,
    pub pre_defined: u16,
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for MediaHeaderBox {
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
        let timescale = u32::deserialize(&mut reader)?;
        let duration = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        let language = Langauge::deserialize(&mut reader)?;
        let pre_defined = u16::deserialize(&mut reader)?;

        Ok(Self {
            header: seed,
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
        self.header.serialize(&mut writer)?;

        if self.header.version == 1 {
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
        let mut size = self.header.size();
        if self.header.version == 1 {
            size += 8 + 8 + 4 + 8; // creation_time, modification_time, timescale, duration
        } else {
            size += 4 + 4 + 4 + 4; // creation_time, modification_time, timescale, duration
        }
        size += self.language.size(); // language
        size += 2; // pre_defined
        size
    }
}

nutype_enum! {
    pub enum HandlerType([u8; 4]) {
        Null = *b"null",
        Video = *b"vide",
        AuxiliaryVideo = *b"auxv",
        Audio = *b"soun",
        Metadata = *b"meta",
        MetadataMpeg7t = *b"mp7t",
        MetadataMpeg7b = *b"mp7b",
        Hint = *b"hint",
        Text = *b"text",
        Subtitle = *b"subt",
        Font = *b"fdsm",
        VolumetricVisual = *b"volv",
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
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"hdlr", crate_path = crate)]
pub struct HandlerBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub pre_defined: u32,
    #[iso_box(from = "[u8; 4]")]
    pub handler_type: HandlerType,
    pub reserved: [u32; 3],
    pub name: Utf8String,
}

/// Media information box
///
/// ISO/IEC 14496-12 - 8.4.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"minf", crate_path = crate)]
pub struct MediaInformationBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub stbl: SampleTableBox<'a>,
    #[iso_box(nested_box)]
    pub dinf: DataInformationBox<'a>,
    #[iso_box(nested_box(collect))]
    pub vmhd: Option<VideoMediaHeaderBox>,
    #[iso_box(nested_box(collect))]
    pub smhd: Option<SoundMediaHeaderBox>,
    #[iso_box(nested_box(collect))]
    pub hmhd: Option<HintMediaHeaderBox>,
    #[iso_box(nested_box(collect))]
    pub sthd: Option<SubtitleMediaHeaderBox>,
    #[iso_box(nested_box(collect))]
    pub vvhd: Option<VolumetricVisualMediaHeaderBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Null media header box
///
/// ISO/IEC 14496-12 - 8.4.5.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"nmhd", crate_path = crate)]
pub struct NullMediaHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
}

/// Extended language tag
///
/// ISO/IEC 14496-12 - 8.4.6
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"elng", crate_path = crate)]
pub struct ExtendedLanguageBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub extended_language: Utf8String,
}
