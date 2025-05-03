use std::io;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use crate::boxes::{
    ExtendedTypeBox, FileTypeBox, IdentifiedMediaDataBox, MediaDataBox, MetaBox, MovieBox, ProgressiveDownloadInfoBox,
};
use crate::{IsoBox, UnknownBox};

#[derive(Debug)]
pub struct EmptyHeader;

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"root", crate_path = "crate", skip_deserialize_impl)] // the box_type is not used
pub struct IsobmffFile<'a> {
    #[iso_box(header)]
    pub header: EmptyHeader,
    pub ftyp: FileTypeBox,
    #[iso_box(nested_box(collect))]
    pub etyp: Vec<ExtendedTypeBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub mdat: Vec<MediaDataBox>,
    #[iso_box(nested_box(collect))]
    pub pdin: Option<ProgressiveDownloadInfoBox>,
    #[iso_box(nested_box(collect))]
    pub imda: Vec<IdentifiedMediaDataBox>,
    #[iso_box(nested_box)]
    pub moov: MovieBox<'a>,
    // pub moof: Vec<MovieFragmentBox>,
    // pub mfra: Vec<MovieFragmentRandomAccessBox>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    // pub styp: Vec<SegmentTypeBox>,
    // pub sidx: Vec<SegmentIndexBox>,
    // pub ssix: Vec<SubsegmentIndexBox>,
    // pub prft: Vec<ProducerReferenceTimeBox>,
    // pub otyp: Vec<OriginalFileTypeBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

impl<'a> Deserialize<'a> for IsobmffFile<'a> {
    fn deserialize<R>(reader: R) -> io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        <Self as DeserializeSeed<EmptyHeader>>::deserialize_seed(reader, EmptyHeader)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::path::PathBuf;

    use scuffle_bytes_util::zero_copy::Deserialize;

    use super::IsobmffFile;

    #[test]
    fn avc_aac_sample() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let data = std::fs::read(dir.join("avc_aac.mp4").to_str().unwrap()).unwrap();
        let mut reader = scuffle_bytes_util::zero_copy::Slice::from(&data[..]);
        let file = IsobmffFile::deserialize(&mut reader).unwrap();
        insta::assert_debug_snapshot!(file);
    }
}
