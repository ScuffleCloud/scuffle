//! FLV video tag bodies.

use std::io;

use bytes::Bytes;
use enhanced::ExVideoTagBody;
use legacy::LegacyVideoTagBody;

use super::header::{VideoTagHeader, VideoTagHeaderData};
use crate::error::FlvError;

pub mod enhanced;
pub mod legacy;

/// FLV `VideoTagBody`
///
/// This only describes the video tag body, see [`VideoData`](super::VideoData) for the full video data container.
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.3.1
/// - Enhanced RTMP spec, page 27-31, Enhanced Video
#[derive(Debug, Clone, PartialEq)]
pub enum VideoTagBody<'a> {
    /// Legacy video tag body.
    Legacy(LegacyVideoTagBody),
    /// Enhanced video tag body.
    Enhanced(ExVideoTagBody<'a>),
}

impl VideoTagBody<'_> {
    /// Demux the video tag body from the given reader.
    ///
    /// If you want to demux the full video data tag, use [`VideoData::demux`](super::VideoData::demux) instead.
    /// This function will automatically determine whether the given data represents a legacy or an enhanced video tag body
    /// and demux it accordingly.
    ///
    /// The reader will be entirely consumed.
    pub fn demux(header: &VideoTagHeader, reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
        match &header.data {
            VideoTagHeaderData::Legacy(header) => Ok(Self::Legacy(LegacyVideoTagBody::demux(header, reader)?)),
            VideoTagHeaderData::Enhanced(header) => ExVideoTagBody::demux(header, reader).map(Self::Enhanced),
        }
    }
}
