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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stbl", crate_path = crate)]
pub struct SampleTableBox<'a> {
    /// The contained [`SampleDescriptionBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub stsd: SampleDescriptionBox<'a>,
    /// The contained [`TimeToSampleBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub stts: TimeToSampleBox,
    /// The contained [`CompositionOffsetBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub ctts: Option<CompositionOffsetBox>,
    /// The contained [`CompositionToDecodeBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub cslg: Option<CompositionToDecodeBox>,
    /// The contained [`SampleToChunkBox`]. (mandatory)
    #[iso_box(nested_box)]
    pub stsc: SampleToChunkBox,
    /// The contained [`SampleSizeBox`].
    ///
    /// One of [`stsz`](Self::stsz) or [`stz2`](Self::stz2) must be present.
    #[iso_box(nested_box(collect))]
    pub stsz: Option<SampleSizeBox>,
    /// The contained [`CompactSampleSizeBox`].
    ///
    /// One of [`stsz`](Self::stsz) or [`stz2`](Self::stz2) must be present.
    #[iso_box(nested_box(collect))]
    pub stz2: Option<CompactSampleSizeBox<'a>>,
    /// The contained [`ChunkOffsetBox`].
    ///
    /// One of [`stco`](Self::stco) or [`co64`](Self::co64) must be present
    #[iso_box(nested_box(collect))]
    pub stco: Option<ChunkOffsetBox>,
    /// The contained [`ChunkLargeOffsetBox`].
    ///
    /// One of [`stco`](Self::stco) or [`co64`](Self::co64) must be present
    #[iso_box(nested_box(collect))]
    pub co64: Option<ChunkLargeOffsetBox>,
    /// The contained [`SyncSampleBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub stss: Option<SyncSampleBox>,
    /// The contained [`ShadowSyncSampleBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub stsh: Option<ShadowSyncSampleBox>,
    /// The contained [`PaddingBitsBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub padb: Option<PaddingBitsBox>,
    /// The contained [`DegradationPriorityBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub stdp: Option<DegradationPriorityBox>,
    /// The contained [`SampleDependencyTypeBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub sdtp: Option<SampleDependencyTypeBox>,
    /// The contained [`SampleToGroupBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub sbgp: Vec<SampleToGroupBox>,
    /// The contained [`SampleGroupDescriptionBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub sgpd: Vec<SampleGroupDescriptionBox<'a>>,
    /// The contained [`SubSampleInformationBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub subs: Vec<SubSampleInformationBox>,
    /// The contained [`SampleAuxiliaryInformationSizesBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub saiz: Vec<SampleAuxiliaryInformationSizesBox<'a>>,
    /// The contained [`SampleAuxiliaryInformationOffsetsBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub saio: Vec<SampleAuxiliaryInformationOffsetsBox>,
    /// The contained [`CompactSampleToGroupBox`]. (optional)
    #[iso_box(nested_box(collect))]
    pub csgp: Vec<CompactSampleToGroupBox>,
}

impl<'a> SampleTableBox<'a> {
    /// Creates a new [`SampleTableBox`] with the mandatory boxes and optional boxes set to `None`.
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
#[derive(Debug, PartialEq, Eq)]
pub struct SampleEntry {
    /// Reserved 6 bytes, must be zero.
    pub reserved: [u8; 6],
    /// An integer that contains the index of the `DataEntry` to use to retrieve
    /// data associated with samples that use this sample description. Data entries are stored in
    /// [`DataReferenceBox`](super::DataReferenceBox)es. The index ranges from 1 to the number of data entries.
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
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"btrt", crate_path = crate)]
pub struct BitRateBox {
    /// Gives the size of the decoding buffer for the elementary stream in bytes.
    pub buffer_size_db: u32,
    /// Gives the maximum rate in bits/second over any window of one second; this is a measured
    /// value for stored content, or a value that a stream is configured not to exceed; the stream shall not
    /// exceed this bitrate.
    pub max_bitrate: u32,
    /// Gives the average rate in bits/second of the stream; this is a measured value (cumulative
    /// over the entire presentation) for stored content, or the configured target average bitrate for a
    /// stream.
    pub avg_bitrate: u32,
}

/// Sample description box
///
/// ISO/IEC 14496-12 - 8.5.2
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stsd", crate_path = crate)]
pub struct SampleDescriptionBox<'a> {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer that gives the number of entries in the [`boxes`](Self::boxes) vec.
    pub entry_count: u32,
    /// The contained boxes, which can be any sample entry box.
    #[iso_box(nested_box(collect_unknown))]
    pub boxes: Vec<UnknownBox<'a>>,
}

impl<'a> SampleDescriptionBox<'a> {
    /// Creates a new [`SampleDescriptionBox`] with the given sample entry boxes.
    pub fn new(boxes: Vec<UnknownBox<'a>>) -> Self {
        Self {
            full_header: FullBoxHeader::default(),
            entry_count: boxes.len() as u32,
            boxes,
        }
    }
}

/// Degradation priority box
///
/// ISO/IEC 14496-12 - 8.5.3
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"stdp", crate_path = crate)]
pub struct DegradationPriorityBox {
    /// The full box header.
    pub full_header: FullBoxHeader,
    /// An integer specifying the degradation priority for each sample.
    #[iso_box(repeated)]
    pub priority: Vec<u16>,
}
