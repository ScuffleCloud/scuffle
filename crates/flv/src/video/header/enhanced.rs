use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use super::VideoFrameType;
use crate::common::AvMultitrackType;
use crate::error::Error;
use crate::video::header::VideoCommand;

nutype_enum! {
    /// The type of video packet in an enhanced FLV file.
    ///
    /// Defined by:
    /// - enhanced_rtmp-v1.pdf (Defining Additional Video Codecs)
    /// - enhanced_rtmp-v2.pdf (Enhanced Video)
    pub enum VideoPacketType(u8) {
        /// Sequence Start
        SequenceStart = 0,
        /// Coded Frames
        CodedFrames = 1,
        /// Sequence End
        SequenceEnd = 2,
        /// Coded Frames X
        CodedFramesX = 3,
        /// Metadata
        Metadata = 4,
        /// MPEG-2 TS Sequence Start
        Mpeg2TsSequenceStart = 5,
        /// Multitrack mode
        Multitrack = 6,
        /// Modifier extensions
        ModEx = 7,
    }
}

nutype_enum! {
    pub enum VideoPacketModExType(u8) {
        TimestampOffsetNano = 0,
    }
}

/// This is a helper enum to represent the different types of video packet modifier extensions.
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketModEx {
    TimestampOffsetNano {
        video_timestamp_nano_offset: u32,
    },
    Other {
        video_packet_mod_ex_type: VideoPacketModExType,
        mod_ex_data: Bytes,
    },
}

nutype_enum! {
    /// FLV Video FourCC
    ///
    /// Denotes the different types of video codecs that can be used in a FLV file.
    ///
    /// Defined by:
    /// - enhanced_rtmp-v1.pdf (Defining Additional Video Codecs)
    /// - enhanced_rtmp-v2.pdf (Enhanced Video)
    pub enum VideoFourCc([u8; 4]) {
        /// VP8
        Vp8 = *b"vp08",
        /// VP9
        Vp9 = *b"vp09",
        /// AV1
        Av1 = *b"av01",
        /// AVC (H.264)
        Avc = *b"avc1",
        /// HEVC (H.265)
        Hevc = *b"hvc1",
    }
}

/// This is a helper enum to represent the different types of enhanced video headers.
#[derive(Debug, Clone, PartialEq)]
pub enum ExVideoTagHeaderContent {
    VideoCommand(VideoCommand),
    NoMultiTrack(VideoFourCc),
    OneTrack(VideoFourCc),
    ManyTracks(VideoFourCc),
    ManyTracksManyCodecs,
    Other {
        video_multitrack_type: AvMultitrackType,
        video_four_cc: VideoFourCc,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExVideoTagHeader {
    pub video_packet_type: VideoPacketType,
    pub video_packet_mod_exs: Vec<VideoPacketModEx>,
    pub content: ExVideoTagHeaderContent,
}

impl ExVideoTagHeader {
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let byte = reader.read_u8()?;
        let video_frame_type = VideoFrameType::from((byte & 0b0_111_0000) >> 4);
        let mut video_packet_type = VideoPacketType::from(byte & 0b0000_1111);

        let mut video_packet_mod_exs = Vec::new();

        // Read all modifier extensions
        while video_packet_type == VideoPacketType::ModEx {
            let mut mod_ex_data_size = reader.read_u8()? as usize + 1;
            if mod_ex_data_size == 256 {
                mod_ex_data_size = reader.read_u16::<BigEndian>()? as usize + 1;
            }

            let mod_ex_data = reader.extract_bytes(mod_ex_data_size)?;

            let next_byte = reader.read_u8()?;
            let video_packet_mod_ex_type = VideoPacketModExType::from(next_byte >> 4); // 0b1111_0000
            video_packet_type = VideoPacketType::from(next_byte & 0b0000_1111);

            if video_packet_mod_ex_type == VideoPacketModExType::TimestampOffsetNano {
                if mod_ex_data_size < 3 {
                    // too few data bytes for the timestamp offset
                    return Err(Error::InvalidModExData { expected_bytes: 3 });
                }

                let mod_ex_data = &mut io::Cursor::new(mod_ex_data);
                video_packet_mod_exs.push(VideoPacketModEx::TimestampOffsetNano {
                    video_timestamp_nano_offset: mod_ex_data.read_u24::<BigEndian>()?,
                });
            } else {
                video_packet_mod_exs.push(VideoPacketModEx::Other {
                    video_packet_mod_ex_type,
                    mod_ex_data,
                });
            }
        }

        let content = if video_packet_type != VideoPacketType::Metadata && video_frame_type == VideoFrameType::Command {
            let video_command = VideoCommand::from(reader.read_u8()?);
            ExVideoTagHeaderContent::VideoCommand(video_command)
        } else if video_packet_type == VideoPacketType::Multitrack {
            let next_byte = reader.read_u8()?;
            let video_multitrack_type = AvMultitrackType::from(next_byte >> 4); // 0b1111_0000
            video_packet_type = VideoPacketType::from(next_byte & 0b0000_1111);

            if video_packet_type == VideoPacketType::Multitrack {
                // nested multitracks are not allowed
                return Err(Error::NestedMultitracks);
            }

            let mut video_four_cc = [0; 4];
            // Only read the four cc if it's not ManyTracksManyCodecs
            if video_multitrack_type != AvMultitrackType::ManyTracksManyCodecs {
                reader.read_exact(&mut video_four_cc)?;
            }

            match video_multitrack_type {
                AvMultitrackType::OneTrack => ExVideoTagHeaderContent::OneTrack(VideoFourCc::from(video_four_cc)),
                AvMultitrackType::ManyTracks => ExVideoTagHeaderContent::ManyTracks(VideoFourCc::from(video_four_cc)),
                AvMultitrackType::ManyTracksManyCodecs => ExVideoTagHeaderContent::ManyTracksManyCodecs,
                _ => ExVideoTagHeaderContent::Other {
                    video_multitrack_type,
                    video_four_cc: VideoFourCc::from(video_four_cc),
                },
            }
        } else {
            let mut video_four_cc = [0; 4];
            reader.read_exact(&mut video_four_cc)?;

            ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::from(video_four_cc))
        };

        Ok(Self {
            video_packet_type,
            video_packet_mod_exs,
            content,
        })
    }
}
