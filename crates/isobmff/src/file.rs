use std::fmt::Debug;

use scuffle_bytes_util::zero_copy::{Deserialize, DeserializeSeed};

use crate::boxes::{
    ExtendedTypeBox, FileTypeBox, IdentifiedMediaDataBox, MediaDataBox, MetaBox, MovieBox, MovieFragmentBox,
    MovieFragmentRandomAccessBox, OriginalFileTypeBox, ProducerReferenceTimeBox, ProgressiveDownloadInfoBox,
    SegmentIndexBox, SegmentTypeBox, SubsegmentIndexBox,
};
use crate::{BoxHeader, BoxSize, BoxType, IsoBox, UnknownBox};

/// Represents an ISO Base Media File Format (ISOBMFF) file.
///
/// This encapsulates all boxes that may be present in an ISOBMFF file.
/// You can also use the boxes directly for more fine-grained control.
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(skip_impl(iso_box, deserialize), crate_path = crate)]
pub struct IsobmffFile<'a> {
    /// Optional [`FileTypeBox`].
    ///
    /// According to the official specification the [`FileTypeBox`] is mandatory
    /// but in reality some files do not contain it. (e.g. recording of live streams)
    #[iso_box(nested_box(collect))]
    pub ftyp: Option<FileTypeBox>,
    /// A list of [`ExtendedTypeBox`]es.
    #[iso_box(nested_box(collect))]
    pub etyp: Vec<ExtendedTypeBox<'a>>,
    /// A list of [`OriginalFileTypeBox`]es.
    #[iso_box(nested_box(collect))]
    pub otyp: Vec<OriginalFileTypeBox<'a>>,
    /// Optional [`ProgressiveDownloadInfoBox`].
    #[iso_box(nested_box(collect))]
    pub pdin: Option<ProgressiveDownloadInfoBox>,
    /// Optional [`MovieBox`].
    ///
    /// According to the official specification the [`MovieBox`] is mandatory,
    /// but in reality some files (e.g. HEIF) do not contain it.
    /// Apparently it is possible for derived specifications to change the
    /// rules of the base specification.
    ///
    /// See: https://github.com/MPEGGroup/FileFormatConformance/issues/154
    #[iso_box(nested_box(collect))]
    pub moov: Option<MovieBox<'a>>,
    /// A list of [`MovieFragmentBox`]es.
    #[iso_box(nested_box(collect))]
    pub moof: Vec<MovieFragmentBox<'a>>,
    /// A list of [`MediaDataBox`]es.
    #[iso_box(nested_box(collect))]
    pub mdat: Vec<MediaDataBox<'a>>,
    /// A list of [`IdentifiedMediaDataBox`]es.
    #[iso_box(nested_box(collect))]
    pub imda: Vec<IdentifiedMediaDataBox<'a>>,
    #[iso_box(nested_box(collect))]
    /// Optional [`MetaBox`].
    pub meta: Option<MetaBox<'a>>,
    /// A list of [`SegmentTypeBox`]es.
    #[iso_box(nested_box(collect))]
    pub styp: Vec<SegmentTypeBox>,
    /// A list of [`SegmentIndexBox`]es.
    #[iso_box(nested_box(collect))]
    pub sidx: Vec<SegmentIndexBox>,
    /// A list of [`SubsegmentIndexBox`]es.
    #[iso_box(nested_box(collect))]
    pub ssix: Vec<SubsegmentIndexBox>,
    /// A list of [`ProducerReferenceTimeBox`]es.
    #[iso_box(nested_box(collect))]
    pub prft: Vec<ProducerReferenceTimeBox>,
    /// Any unknown boxes that were not recognized during deserialization.
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
    /// Optional [`MovieFragmentRandomAccessBox`].
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
