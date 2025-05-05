//! FLV video tag headers.

use std::io::{self, Seek};

use byteorder::ReadBytesExt;
use bytes::Bytes;
use nutype_enum::nutype_enum;

use crate::error::FlvError;

pub mod enhanced;
pub mod legacy;

nutype_enum! {
    /// FLV Frame Type
    ///
    /// This enum represents the different types of frames in a FLV file.
    ///
    /// Defined by:
    /// - Legacy FLV spec, Annex E.4.3.1
    pub enum VideoFrameType(u8) {
        /// A keyframe is a frame that is a complete representation of the video content.
        KeyFrame = 1,
        /// An interframe is a frame that is a partial representation of the video content.
        InterFrame = 2,
        /// A disposable interframe is a frame that is a partial representation of the video content, but is not required to be displayed. (h263 only)
        DisposableInterFrame = 3,
        /// A generated keyframe is a frame that is a complete representation of the video content, but is not a keyframe. (reserved for server use only)
        GeneratedKeyFrame = 4,
        /// A video info or command frame is a frame that contains video information or commands.
        /// If the frame is this type, the body will be a CommandPacket
        Command = 5,
    }
}

nutype_enum! {
    /// FLV Video Command
    ///
    /// Defined by:
    /// - Legacy FLV spec, Annex E.4.3.1, VideoTagBody
    /// - Enhanced RTMP spec, page 26, Enhanced Video
    pub enum VideoCommand(u8) {
        /// Start of client-side seeking video frame sequence
        StartSeek = 0,
        /// End of client-side seeking video frame sequence
        EndSeek = 1,
    }
}

/// A wrapper for the different types of video tag header data.
#[derive(Debug, Clone, PartialEq)]
pub enum VideoTagHeaderData {
    /// Legacy video tag header.
    Legacy(legacy::LegacyVideoTagHeader),
    /// Enhanced video tag header.
    Enhanced(enhanced::ExVideoTagHeader),
}

/// FLV `VideoTagHeader`
///
/// This only describes the video tag header, see [`VideoData`](super::VideoData) for the full video data container.
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.3.1
/// - Enhanced RTMP spec, page 26-28, Enhanced Video
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTagHeader {
    /// The frame type of the video data.
    pub frame_type: VideoFrameType,
    /// The data of the video tag header.
    pub data: VideoTagHeaderData,
}

impl VideoTagHeader {
    /// Demux the video tag header from the given reader.
    ///
    /// If you want to demux the full video data tag, use [`VideoData::demux`](super::VideoData::demux) instead.
    /// This function will automatically determine whether the given data represents a legacy or an enhanced video tag header
    /// and demux it accordingly.
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
        let byte = reader.read_u8()?;
        // seek back one byte so that the codec id can be read again
        reader.seek_relative(-1)?;

        let is_ex_video_header = (byte & 0b1_000_0000) != 0;

        let data = if !is_ex_video_header {
            VideoTagHeaderData::Legacy(legacy::LegacyVideoTagHeader::demux(reader)?)
        } else {
            VideoTagHeaderData::Enhanced(enhanced::ExVideoTagHeader::demux(reader)?)
        };

        Ok(VideoTagHeader {
            frame_type: VideoFrameType::from((byte & 0b0_111_0000) >> 4),
            data,
        })
    }
}
