use std::io;

use bytes::Bytes;
use enhanced::ExVideoTagBody;
use legacy::LegacyVideoTagBody;

use super::header::{VideoTagHeader, VideoTagHeaderData};
use crate::error::Error;

pub mod enhanced;
pub mod legacy;

#[derive(Debug, Clone, PartialEq)]
pub enum VideoTagBody {
    Legacy(LegacyVideoTagBody),
    Enhanced(ExVideoTagBody),
}

impl VideoTagBody {
    /// Demux the video tag body from the given reader.
    ///
    /// The reader will be entirely consumed.
    pub fn demux(header: &VideoTagHeader, reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        match &header.data {
            VideoTagHeaderData::Legacy(header) => Ok(Self::Legacy(LegacyVideoTagBody::demux(header, reader)?)),
            VideoTagHeaderData::Enhanced(header) => ExVideoTagBody::demux(header, reader).map(Self::Enhanced),
        }
    }
}
