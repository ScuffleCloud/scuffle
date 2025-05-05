//! FLV file processing

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};

use super::header::FlvHeader;
use super::tag::FlvTag;
use crate::error::FlvError;

/// An FLV file is a combination of a [`FlvHeader`] followed by the
/// FLV File Body (which is a series of [`FlvTag`]s)
///
/// The FLV File Body is defined by:
/// - Legacy FLV spec, Annex E.3
#[derive(Debug, Clone, PartialEq)]
pub struct FlvFile<'a> {
    /// The header of the FLV file.
    pub header: FlvHeader,
    /// The tags in the FLV file.
    pub tags: Vec<FlvTag<'a>>,
}

impl FlvFile<'_> {
    /// Demux an FLV file from a reader.
    ///
    /// The reader needs to be a [`std::io::Cursor`] with a [`Bytes`] buffer because we
    /// take advantage of zero-copy reading.
    pub fn demux(reader: &mut std::io::Cursor<Bytes>) -> Result<Self, FlvError> {
        let header = FlvHeader::demux(reader)?;

        let mut tags = Vec::new();
        while reader.has_remaining() {
            // We don't care about the previous tag size, its only really used for seeking
            // backwards.
            reader.read_u32::<BigEndian>()?;

            // If there is no more data, we can stop reading.
            if !reader.has_remaining() {
                break;
            }

            // Demux the tag from the reader.
            let tag = FlvTag::demux(reader)?;
            tags.push(tag);
        }

        Ok(FlvFile { header, tags })
    }
}
