use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use crate::boxes::{
    ExtendedTypeBox, FileTypeBox, IdentifiedMediaDataBox, MediaDataBox, MetaBox, MovieBox, MovieFragmentBox, ProgressiveDownloadInfoBox
};
use crate::{IsoBox, UnknownBox};

#[derive(IsoBox, Debug)]
#[iso_box(box_type = b"root", skip_deserialize_impl, crate_path = crate)] // The box type does not matter here
pub struct IsobmffFile<'a> {
    #[iso_box(header)]
    _empty_header: (),
    pub ftyp: FileTypeBox,
    #[iso_box(nested_box(collect))]
    pub etyp: Vec<ExtendedTypeBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub mdat: Vec<MediaDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub pdin: Option<ProgressiveDownloadInfoBox>,
    #[iso_box(nested_box(collect))]
    pub imda: Vec<IdentifiedMediaDataBox<'a>>,
    #[iso_box(nested_box)]
    pub moov: MovieBox<'a>,
    #[iso_box(nested_box(collect))]
    pub moof: Vec<MovieFragmentBox<'a>>,
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
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        <Self as DeserializeSeed<()>>::deserialize_seed(reader, ())
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
