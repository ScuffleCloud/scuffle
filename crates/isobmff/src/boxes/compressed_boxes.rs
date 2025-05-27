use scuffle_bytes_util::BytesCow;

use super::{ExtendedTypeBox, MovieBox, MovieFragmentBox, SegmentIndexBox, SubsegmentIndexBox};
use crate::{IsoBox, UnknownBox};

/// Original file-type box
///
/// ISO/IEC 14496-12 - 8.19.5
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"otyp", crate_path = crate)]
pub struct OriginalFileTypeBox<'a> {
    #[iso_box(nested_box(collect))]
    pub etyp: Option<ExtendedTypeBox<'a>>,
    #[iso_box(nested_box(collect_unknown))]
    pub unknown_boxes: Vec<UnknownBox<'a>>,
}

pub trait CompressedBox {
    type UncompressedBox: IsoBox;
}

/// Compressed movie box
///
/// ISO/IEC 14496-12 - 8.19.6
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"!mov", crate_path = crate)]
pub struct CompressedMovieBox<'a> {
    pub data: BytesCow<'a>,
}

impl<'a> CompressedBox for CompressedMovieBox<'a> {
    type UncompressedBox = MovieBox<'a>;
}

/// Compressed movie fragment box
///
/// ISO/IEC 14496-12 - 8.19.7
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"!mof", crate_path = crate)]
pub struct CompressedMovieFragmentBox<'a> {
    pub data: BytesCow<'a>,
}

impl<'a> CompressedBox for CompressedMovieFragmentBox<'a> {
    type UncompressedBox = MovieFragmentBox<'a>;
}

/// Compressed segment index box
///
/// ISO/IEC 14496-12 - 8.19.8
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"!six", crate_path = crate)]
pub struct CompressedSegmentIndexBox<'a> {
    pub data: BytesCow<'a>,
}

impl CompressedBox for CompressedSegmentIndexBox<'_> {
    type UncompressedBox = SegmentIndexBox;
}

/// Compressed subsegment index box
///
/// ISO/IEC 14496-12 - 8.19.9
#[derive(IsoBox, Debug, PartialEq, Eq)]
#[iso_box(box_type = b"!ssx", crate_path = crate)]
pub struct CompressedSubsegmentIndexBox<'a> {
    pub data: BytesCow<'a>,
}

impl CompressedBox for CompressedSubsegmentIndexBox<'_> {
    type UncompressedBox = SubsegmentIndexBox;
}
