use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, ZeroCopyReader};

use super::{
    ChunkLargeOffsetBox, ChunkOffsetBox, CompactSampleSizeBox, CompositionOffsetBox, CompositionToDecodeBox, PaddingBitsBox,
    SampleAuxiliaryInformationOffsetsBox, SampleAuxiliaryInformationSizesBox, SampleDependencyTypeBox, SampleSizeBox,
    SampleToChunkBox, ShadowSyncSampleBox, SubSampleInformationBox, SyncSampleBox, TimeToSampleBox,
};
use crate::{BoxHeader, FullBoxHeader, IsoBox, UnknownBox};

/// Sample table box
///
/// ISO/IEC 14496-12 - 8.5.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stbl", crate_path = crate)]
pub struct SampleTableBox<'a> {
    #[iso_box(header)]
    pub header: BoxHeader,
    #[iso_box(nested_box)]
    pub stsd: SampleDescriptionBox<'a>,
    #[iso_box(nested_box(collect))]
    pub stdp: Option<DegradationPriorityBox>,
    #[iso_box(nested_box)]
    pub stts: TimeToSampleBox,
    #[iso_box(nested_box(collect))]
    pub ctts: Option<CompositionOffsetBox>,
    #[iso_box(nested_box(collect))]
    pub cslg: Option<CompositionToDecodeBox>,
    #[iso_box(nested_box(collect))]
    pub stss: Option<SyncSampleBox>,
    #[iso_box(nested_box(collect))]
    pub stsh: Option<ShadowSyncSampleBox>,
    #[iso_box(nested_box(collect))]
    pub sdtp: Option<SampleDependencyTypeBox>,

    // one of stsz or stz2 must be present
    #[iso_box(nested_box(collect))]
    pub stsz: Option<SampleSizeBox>,
    #[iso_box(nested_box(collect))]
    pub stz2: Option<CompactSampleSizeBox<'a>>,

    #[iso_box(nested_box)]
    pub stsc: SampleToChunkBox,

    // one of stco or co64 must be present
    #[iso_box(nested_box(collect))]
    pub stco: Option<ChunkOffsetBox>,
    #[iso_box(nested_box(collect))]
    pub co64: Option<ChunkLargeOffsetBox>,

    #[iso_box(nested_box(collect))]
    pub padb: Option<PaddingBitsBox>,
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
}

/// Sample entry
///
/// ISO/IEC 14496-12 - 8.5.2
#[derive(Debug)]
pub struct SampleEntry {
    pub data_reference_index: u16,
}

impl<'a> Deserialize<'a> for SampleEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        <[u8; 6]>::deserialize(&mut reader)?; // reserved
        let data_reference_index = u16::deserialize(&mut reader)?;

        Ok(Self { data_reference_index })
    }
}

/// BitRateBox
///
/// ISO/IEC 14496-12 - 8.5.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"btrt", crate_path = crate)]
pub struct BitRateBox {
    #[iso_box(header)]
    pub header: BoxHeader,
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
}

/// Sample description box
///
/// ISO/IEC 14496-12 - 8.5.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stsd", crate_path = crate)]
pub struct SampleDescriptionBox<'a> {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stdp", crate_path = crate)]
pub struct DegradationPriorityBox {
    #[iso_box(header)]
    pub header: FullBoxHeader,
    #[iso_box(repeated)]
    pub priority: Vec<u16>,
}
