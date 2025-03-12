use std::io::{self, Seek};

use byteorder::ReadBytesExt;
use bytes::Bytes;
use nutype_enum::nutype_enum;

use crate::error::Error;

pub mod enhanced;
pub mod legacy;

nutype_enum! {
    /// FLV Frame Type
    /// This enum represents the different types of frames in a FLV file.
    /// Defined by:
    /// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Video tags)
    /// - video_file_format_spec_v10_1.pdf (Annex E.4.3.1 - VIDEODATA)
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
    pub enum VideoCommand(u8) {
        StartSeek = 0,
        EndSeek = 1,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoTagHeaderData {
    Legacy(legacy::LegacyVideoTagHeader),
    Enhanced(enhanced::ExVideoTagHeader),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoTagHeader {
    /// The frame type of the video data. (4 bits)
    pub frame_type: VideoFrameType,
    pub data: VideoTagHeaderData,
}

impl VideoTagHeader {
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
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
