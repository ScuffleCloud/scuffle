use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, ZeroCopyReader};

use super::{
    ChunkLargeOffsetBox, ChunkOffsetBox, CompactSampleSizeBox, CompactSampleToGroupBox, CompositionOffsetBox,
    CompositionToDecodeBox, PaddingBitsBox, SampleAuxiliaryInformationOffsetsBox, SampleAuxiliaryInformationSizesBox,
    SampleDependencyTypeBox, SampleGroupDescriptionBox, SampleSizeBox, SampleToChunkBox, SampleToGroupBox,
    ShadowSyncSampleBox, SubSampleInformationBox, SyncSampleBox, TimeToSampleBox,
};
use crate::{FullBoxHeader, IsoBox, IsoSized, UnknownBox};

/// Sample table box
///
/// ISO/IEC 14496-12 - 8.5.1
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stbl", crate_path = crate)]
pub struct SampleTableBox<'a> {
    #[iso_box(nested_box)]
    pub stsd: SampleDescriptionBox<'a>,
    #[iso_box(nested_box)]
    pub stts: TimeToSampleBox,
    #[iso_box(nested_box(collect))]
    pub ctts: Option<CompositionOffsetBox>,
    #[iso_box(nested_box(collect))]
    pub cslg: Option<CompositionToDecodeBox>,
    #[iso_box(nested_box)]
    pub stsc: SampleToChunkBox,
    // one of stsz or stz2 must be present
    #[iso_box(nested_box(collect))]
    pub stsz: Option<SampleSizeBox>,
    #[iso_box(nested_box(collect))]
    pub stz2: Option<CompactSampleSizeBox<'a>>,
    // one of stco or co64 must be present
    #[iso_box(nested_box(collect))]
    pub stco: Option<ChunkOffsetBox>,
    #[iso_box(nested_box(collect))]
    pub co64: Option<ChunkLargeOffsetBox>,
    #[iso_box(nested_box(collect))]
    pub stss: Option<SyncSampleBox>,
    #[iso_box(nested_box(collect))]
    pub stsh: Option<ShadowSyncSampleBox>,
    #[iso_box(nested_box(collect))]
    pub padb: Option<PaddingBitsBox>,
    #[iso_box(nested_box(collect))]
    pub stdp: Option<DegradationPriorityBox>,
    #[iso_box(nested_box(collect))]
    pub sdtp: Option<SampleDependencyTypeBox>,
    #[iso_box(nested_box(collect))]
    pub sbgp: Vec<SampleToGroupBox>,
    #[iso_box(nested_box(collect))]
    pub sgpd: Vec<SampleGroupDescriptionBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
    #[iso_box(nested_box(collect))]
    pub csgp: Vec<CompactSampleToGroupBox>,
}

impl<'a> SampleTableBox<'a> {
    pub fn new(
        stsd: SampleDescriptionBox<'a>,
        stts: TimeToSampleBox,
        stsc: SampleToChunkBox,
        stsz: Option<SampleSizeBox>,
        stco: ChunkOffsetBox,
    ) -> Self {
        Self {
            stsd,
            stts,
            ctts: None,
            cslg: None,
            stsc,
            stsz,
            stz2: None,
            stco: Some(stco),
            co64: None,
            stss: None,
            stsh: None,
            padb: None,
            stdp: None,
            sdtp: None,
            sbgp: vec![],
            sgpd: vec![],
            subs: vec![],
            saiz: vec![],
            saio: vec![],
            csgp: vec![],
        }
    }
}

/// Sample entry
///
/// ISO/IEC 14496-12 - 8.5.2
///
/// Sub boxes:
/// - [`btrt`](BitRateBox)
#[derive(Debug)]
pub struct SampleEntry {
    pub reserved: [u8; 6],
    pub data_reference_index: u16,
}

impl Default for SampleEntry {
    fn default() -> Self {
        Self {
            reserved: [0; 6],
            data_reference_index: 1,
        }
    }
}

impl<'a> Deserialize<'a> for SampleEntry {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let reserved = <[u8; 6]>::deserialize(&mut reader)?;
        let data_reference_index = u16::deserialize(&mut reader)?;

        Ok(Self {
            reserved,
            data_reference_index,
        })
    }
}

impl Serialize for SampleEntry {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        self.reserved.serialize(&mut writer)?;
        self.data_reference_index.serialize(&mut writer)?;
        Ok(())
    }
}

impl IsoSized for SampleEntry {
    fn size(&self) -> usize {
        self.reserved.size() + self.data_reference_index.size()
    }
}

/// BitRateBox
///
/// ISO/IEC 14496-12 - 8.5.2
#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"btrt", crate_path = crate)]
pub struct BitRateBox {
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
    pub full_header: FullBoxHeader,
    pub entry_count: u32,
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

impl<'a> SampleDescriptionBox<'a> {
    pub fn new(boxes: Vec<UnknownBox<'a>>) -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            entry_count: boxes.len() as u32,
            boxes,
        }
    }
}

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"stdp", crate_path = crate)]
pub struct DegradationPriorityBox {
    pub full_header: FullBoxHeader,
    #[iso_box(repeated)]
    pub priority: Vec<u16>,
}
