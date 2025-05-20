use std::fmt::Debug;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use crate::boxes::{
    ExtendedTypeBox, FileTypeBox, IdentifiedMediaDataBox, MediaDataBox, MetaBox, MovieBox, MovieFragmentBox,
    MovieFragmentRandomAccessBox, OriginalFileTypeBox, ProducerReferenceTimeBox, ProgressiveDownloadInfoBox,
    SegmentIndexBox, SegmentTypeBox, SubsegmentIndexBox,
};
use crate::{BoxHeader, BoxSize, BoxType, IsoBox, UnknownBox};

#[derive(IsoBox, Debug)]
#[iso_box(skip_impl(iso_box, deserialize), crate_path = crate)]
pub struct IsobmffFile<'a> {
    #[iso_box(nested_box)]
    pub ftyp: FileTypeBox,
    #[iso_box(nested_box(collect))]
    pub etyp: Vec<ExtendedTypeBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub otyp: Vec<OriginalFileTypeBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub pdin: Option<ProgressiveDownloadInfoBox>,
    #[iso_box(nested_box)]
    pub moov: MovieBox<'a>,
    #[iso_box(nested_box(collect))]
    pub moof: Vec<MovieFragmentBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub mdat: Vec<MediaDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub imda: Vec<IdentifiedMediaDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub meta: Option<MetaBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub styp: Vec<SegmentTypeBox>,
    #[iso_box(nested_box(collect))]
    pub sidx: Vec<SegmentIndexBox>,
    #[iso_box(nested_box(collect))]
    pub ssix: Vec<SubsegmentIndexBox>,
    #[iso_box(nested_box(collect))]
    pub prft: Vec<ProducerReferenceTimeBox>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    #[iso_box(nested_box(collect))]
    pub mfra: Option<MovieFragmentRandomAccessBox>,
}

impl<'a> Deserialize<'a> for IsobmffFile<'a> {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        Self::deserialize_seed(
            reader,
            BoxHeader {
                size: BoxSize::ToEnd,
                box_type: BoxType::FourCc(*b"root"),
            },
        )
    }
}

// This trait is usually not implemented manually.
// Since the file does not have a header, we need to implement it manually here.
impl IsoBox for IsobmffFile<'_> {
    const TYPE: BoxType = BoxType::Uuid(uuid::Uuid::nil());

    fn add_header_size(payload_size: usize) -> usize {
        // Return the payload size, because the file does not have a header
        payload_size
    }

    fn serialize_box_header<W>(&self, _writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        // noop, because the file does not have a header
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

    use super::IsobmffFile;
    use crate::IsoSized;

    fn transmux_sample(sample_name: &str, skip_insta: bool) -> io::Result<()> {
        let test_name = sample_name.split('.').next().unwrap();

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let data = std::fs::read(dir.join(sample_name))?;
        let mut reader = scuffle_bytes_util::zero_copy::Slice::from(&data[..]);
        let og_file = IsobmffFile::deserialize(&mut reader)?;
        if !skip_insta {
            insta::assert_debug_snapshot!(test_name, og_file);
        }
        assert_eq!(og_file.size(), data.len());

        let mut out_data = Vec::new();
        og_file.serialize(&mut out_data)?;
        assert_eq!(out_data.len(), data.len());

        let mut reader = scuffle_bytes_util::zero_copy::Slice::from(&out_data[..]);
        let file = IsobmffFile::deserialize(&mut reader)?;
        if !skip_insta {
            insta::assert_debug_snapshot!(test_name, file);
        }

        Ok(())
    }

    #[test]
    fn avc_aac_sample() {
        transmux_sample("avc_aac.mp4", false).unwrap();
    }

    #[test]
    fn avc_aac_large_sample() {
        transmux_sample("avc_aac_large.mp4", false).unwrap();
    }

    #[test]
    fn avc_aac_fragmented_sample() {
        transmux_sample("avc_aac_fragmented.mp4", false).unwrap();
    }

    #[test]
    fn avc_aac_keyframes_sample() {
        transmux_sample("avc_aac_keyframes.mp4", false).unwrap();
    }

    #[test]
    fn hevc_aac_fragmented_sample() {
        // Skip the insta snapshot because it would be too big
        transmux_sample("hevc_aac_fragmented.mp4", true).unwrap();
    }

    #[test]
    fn av1_aac_fragmented_sample() {
        // Skip the insta snapshot because it would be too big
        transmux_sample("av1_aac_fragmented.mp4", true).unwrap();
    }
}
