use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed, ZeroCopyReader};

use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox};

use super::{
    CompositionToDecodeBox, SampleAuxiliaryInformationOffsetsBox, SampleAuxiliaryInformationSizesBox,
    SubSampleInformationBox,
};

/// Movie extends box
///
/// ISO/IEC 14496-12 - 8.8.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mvex", crate_path = crate)]
pub struct MovieExtendsBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box(collect))]
    pub mehd: Option<MovieExtendsHeaderBox>,
    #[iso_box(nested_box(collect))]
    pub trex: Vec<TrackExtendsBox>,
}

/// Movie extends header box
///
/// ISO/IEC 14496-12 - 8.8.2
#[derive(Debug)]
pub struct MovieExtendsHeaderBox {
    pub header: FullBoxHeader,
    pub fragment_duration: u64,
}

impl IsoBox for MovieExtendsHeaderBox {
    const TYPE: [u8; 4] = *b"mehd";
    type Header = FullBoxHeader;
}

impl<'a> Deserialize<'a> for MovieExtendsHeaderBox {
    fn deserialize<R>(mut reader: R) -> ::std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader, header)?;
        Self::deserialize_seed(reader, header)
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for MovieExtendsHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> ::std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let fragment_duration = if seed.version == 1 {
            u64::deserialize(&mut reader)?
        } else {
            u32::deserialize(&mut reader)? as u64
        };

        Ok(Self {
            header: seed,
            fragment_duration,
        })
    }
}

/// Track extends box
///
/// ISO/IEC 14496-12 - 8.8.3
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trex", crate_path = crate)]
pub struct TrackExtendsBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: u32,
}

/// Movie fragment box
///
/// ISO/IEC 14496-12 - 8.8.4
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"moof", crate_path = crate)]
pub struct MovieFragmentBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub mfhd: MovieFragmentHeaderBox,
    #[iso_box(nested_box(collect))]
    pub traf: Vec<TrackFragmentBox<'a>>,
}

/// Movie fragment header box
///
/// ISO/IEC 14496-12 - 8.8.5
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"mfhd", crate_path = crate)]
pub struct MovieFragmentHeaderBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub sequence_number: u32,
}

/// Track fragment box
///
/// ISO/IEC 14496-12 - 8.8.6
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"traf", crate_path = crate)]
pub struct TrackFragmentBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub tfhd: TrackFragmentHeaderBox,
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
}

/// Track fragment header box
///
/// ISO/IEC 14496-12 - 8.8.7
#[derive(Debug)]
pub struct TrackFragmentHeaderBox {
    pub header: FullBoxHeader,
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<u32>,
}

impl IsoBox for TrackFragmentHeaderBox {
    const TYPE: [u8; 4] = *b"tfhd";
    type Header = FullBoxHeader;
}

impl<'a> Deserialize<'a> for TrackFragmentHeaderBox {
    fn deserialize<R>(mut reader: R) -> ::std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let header = BoxHeader::deserialize(&mut reader)?;
        let header = FullBoxHeader::deserialize_seed(&mut reader,header)?;
        Self::deserialize_seed(reader, header)
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TfFlags: u32 {
        const BaseDataOffsetPresent = 0x000001;
        const SampleDescriptionIndexPresent = 0x000002;
        const DefaultSampleDurationPresent = 0x000008;
        const DefaultSampleSizePresent = 0x000010;
        const DefaultSampleFlagsPresent = 0x000020;
        const DurationIsEmpty = 0x010000;
        const DefaultBaseIsMoof = 0x020000;
    }
}

impl<'a> DeserializeSeed<'a, FullBoxHeader> for TrackFragmentHeaderBox {
    fn deserialize_seed<R>(mut reader: R, seed: FullBoxHeader) -> ::std::io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let flags = TfFlags::from_bits_truncate(seed.flags);

        let track_id = u32::deserialize(&mut reader)?;
        let base_data_offset = if flags.contains(TfFlags::BaseDataOffsetPresent) {
            Some(u64::deserialize(&mut reader)?)
        } else {
            None
        };
        let sample_description_index = if flags.contains(TfFlags::SampleDescriptionIndexPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_sample_duration = if flags.contains(TfFlags::DefaultSampleDurationPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_sample_size = if flags.contains(TfFlags::DefaultSampleSizePresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };
        let default_sample_flags = if flags.contains(TfFlags::DefaultSampleFlagsPresent) {
            Some(u32::deserialize(&mut reader)?)
        } else {
            None
        };

        Ok(Self {
            header: seed,
            track_id,
            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        })
    }
}


/// Track fragment run box
///
/// ISO/IEC 14496-12 - 8.8.8

/// Movie fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.9

/// Track fragment random access box
///
/// ISO/IEC 14496-12 - 8.8.10

/// Movie fragment random access offset box
///
/// ISO/IEC 14496-12 - 8.8.11

/// Track fragment decode time box
///
/// ISO/IEC 14496-12 - 8.8.12

/// Level assignment box
///
/// ISO/IEC 14496-12 - 8.8.13

/// Track Extension Properties box
///
/// ISO/IEC 14496-12 - 8.8.15
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"trep", crate_path = crate)]
pub struct TrackExtensionPropertiesBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub track_id: u32,
    #[iso_box(nested_box(collect))]
    pub cslg: Option<CompositionToDecodeBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

/// Alternative startup sequence properties box
///
/// ISO/IEC 14496-12 - 8.8.16
