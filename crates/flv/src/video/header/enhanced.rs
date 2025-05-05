//! Enhanced video header types and functions.

use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use super::VideoFrameType;
use crate::common::AvMultitrackType;
use crate::error::FlvError;
use crate::video::header::VideoCommand;

nutype_enum! {
    /// Different types of video packets.
    ///
    /// Defined by:
    /// - Enhanced RTMP spec, page 27-28, Enhanced Video
    pub enum VideoPacketType(u8) {
        /// Sequence start.
        SequenceStart = 0,
        /// Coded frames.
        CodedFrames = 1,
        /// Sequence end.
        SequenceEnd = 2,
        /// Coded frames without extra data.
        CodedFramesX = 3,
        /// Metadata.
        Metadata = 4,
        /// MPEG-2 TS sequence start.
        Mpeg2TsSequenceStart = 5,
        /// Turns on audio multitrack mode.
        Multitrack = 6,
        /// Modifier extension.
        ModEx = 7,
    }
}

nutype_enum! {
    /// Different types of audio packet modifier extensions.
    pub enum VideoPacketModExType(u8) {
        /// Timestamp offset in nanoseconds.
        TimestampOffsetNano = 0,
    }
}

/// This is a helper enum to represent the different types of video packet modifier extensions.
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketModEx {
    /// Timestamp offset in nanoseconds.
    TimestampOffsetNano {
        /// The timestamp offset in nanoseconds.
        video_timestamp_nano_offset: u32,
    },
    /// Any other modifier extension.
    Other {
        /// The type of the modifier extension.
        video_packet_mod_ex_type: VideoPacketModExType,
        /// The data of the modifier extension.
        mod_ex_data: Bytes,
    },
}

impl VideoPacketModEx {
    /// Demux a [`VideoPacketModEx`] from the given reader.
    ///
    /// Returns the demuxed [`VideoPacketModEx`] and the next [`VideoPacketType`], if successful.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<(Self, VideoPacketType), FlvError> {
        let mut mod_ex_data_size = reader.read_u8()? as usize + 1;
        if mod_ex_data_size == 256 {
            mod_ex_data_size = reader.read_u16::<BigEndian>()? as usize + 1;
        }

        let mod_ex_data = reader.extract_bytes(mod_ex_data_size)?;

        let next_byte = reader.read_u8()?;
        let video_packet_mod_ex_type = VideoPacketModExType::from(next_byte >> 4); // 0b1111_0000
        let video_packet_type = VideoPacketType::from(next_byte & 0b0000_1111);

        if video_packet_mod_ex_type == VideoPacketModExType::TimestampOffsetNano {
            if mod_ex_data_size < 3 {
                // too few data bytes for the timestamp offset
                return Err(FlvError::InvalidModExData { expected_bytes: 3 });
            }

            let mod_ex_data = &mut io::Cursor::new(mod_ex_data);

            Ok((
                VideoPacketModEx::TimestampOffsetNano {
                    video_timestamp_nano_offset: mod_ex_data.read_u24::<BigEndian>()?,
                },
                video_packet_type,
            ))
        } else {
            Ok((
                VideoPacketModEx::Other {
                    video_packet_mod_ex_type,
                    mod_ex_data,
                },
                video_packet_type,
            ))
        }
    }
}

nutype_enum! {
    /// Valid FOURCC values for signaling support of video codecs
    /// in the enhanced FourCC pipeline.
    ///
    /// Defined by:
    /// - Enhanced RTMP spec, page 28, Enhanced Video
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
    /// Video command.
    VideoCommand(VideoCommand),
    /// Not multitrack.
    NoMultiTrack(VideoFourCc),
    /// Multirack with one track.
    OneTrack(VideoFourCc),
    /// Multitrack with many tracks of the same codec.
    ManyTracks(VideoFourCc),
    /// Multitrack with many tracks of different codecs.
    ManyTracksManyCodecs,
    /// Unknown multitrack type.
    Unknown {
        /// The type of the multitrack video.
        video_multitrack_type: AvMultitrackType,
        /// The FOURCC of the video codec.
        video_four_cc: VideoFourCc,
    },
}

/// `ExVideoTagHeader`
///
/// Defined by:
/// - Enhanced RTMP spec, page 27-28, Enhanced Video
#[derive(Debug, Clone, PartialEq)]
pub struct ExVideoTagHeader {
    /// The modifier extensions of the video packet.
    ///
    /// This can be empty if there are no modifier extensions.
    pub video_packet_mod_exs: Vec<VideoPacketModEx>,
    /// The type of the video packet.
    pub video_packet_type: VideoPacketType,
    /// The content of the video packet which contains more information about the multitrack configuration.
    pub content: ExVideoTagHeaderContent,
}

impl ExVideoTagHeader {
    /// Demux an [`ExVideoTagHeader`] from the given reader.
    ///
    /// This is implemented as per Enhanced RTMP spec, page 27-28, ExVideoTagHeader.
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
        let byte = reader.read_u8()?;
        let video_frame_type = VideoFrameType::from((byte & 0b0_111_0000) >> 4);
        let mut video_packet_type = VideoPacketType::from(byte & 0b0000_1111);

        let mut video_packet_mod_exs = Vec::new();

        // Read all modifier extensions
        while video_packet_type == VideoPacketType::ModEx {
            let (mod_ex, next_video_packet_type) = VideoPacketModEx::demux(reader)?;
            video_packet_mod_exs.push(mod_ex);
            video_packet_type = next_video_packet_type;
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
                return Err(FlvError::NestedMultitracks);
            }

            let mut video_four_cc = [0; 4];
            // Only read the FOURCC if it's not ManyTracksManyCodecs
            if video_multitrack_type != AvMultitrackType::ManyTracksManyCodecs {
                reader.read_exact(&mut video_four_cc)?;
            }

            match video_multitrack_type {
                AvMultitrackType::OneTrack => ExVideoTagHeaderContent::OneTrack(VideoFourCc::from(video_four_cc)),
                AvMultitrackType::ManyTracks => ExVideoTagHeaderContent::ManyTracks(VideoFourCc::from(video_four_cc)),
                AvMultitrackType::ManyTracksManyCodecs => ExVideoTagHeaderContent::ManyTracksManyCodecs,
                _ => ExVideoTagHeaderContent::Unknown {
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

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;

    use crate::common::AvMultitrackType;
    use crate::error::FlvError;
    use crate::video::header::VideoCommand;
    use crate::video::header::enhanced::{
        ExVideoTagHeader, ExVideoTagHeaderContent, VideoFourCc, VideoPacketModEx, VideoPacketModExType, VideoPacketType,
    };

    #[test]
    fn small_mod_ex_demux() {
        let data = &[
            1,  // size 2
            42, // data
            42,
            0b0001_0001, // type 1, next packet 1
        ];

        let (mod_ex, next_packet) = VideoPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            mod_ex,
            VideoPacketModEx::Other {
                video_packet_mod_ex_type: VideoPacketModExType(1),
                mod_ex_data: Bytes::from_static(&[42, 42])
            }
        );
        assert_eq!(next_packet, VideoPacketType::CodedFrames);
    }

    #[test]
    fn timestamp_offset_mod_ex_demux() {
        let data = &[
            2, // size 3
            0, // data
            0,
            1,
            0b0000_0000, // type 0, next packet 0
        ];

        let (mod_ex, next_packet) = VideoPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            mod_ex,
            VideoPacketModEx::TimestampOffsetNano {
                video_timestamp_nano_offset: 1
            },
        );
        assert_eq!(next_packet, VideoPacketType::SequenceStart);
    }

    #[test]
    fn big_mod_ex_demux() {
        let data = &[
            255, // size 2
            0,
            1,
            42, // data
            42,
            0b0001_0001, // type 1, next packet 1
        ];

        let (mod_ex, next_packet) = VideoPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            mod_ex,
            VideoPacketModEx::Other {
                video_packet_mod_ex_type: VideoPacketModExType(1),
                mod_ex_data: Bytes::from_static(&[42, 42])
            }
        );
        assert_eq!(next_packet, VideoPacketType::CodedFrames);
    }

    #[test]
    fn mod_ex_demux_error() {
        let data = &[
            0, // size 1
            42,
            0b0000_0010, // type 0, next packet 2
        ];

        let err = VideoPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap_err();

        assert!(matches!(err, FlvError::InvalidModExData { expected_bytes: 3 },));
    }

    #[test]
    fn minimal_header() {
        let data = &[
            0b0000_0000, // type 0
            b'a',        // four cc
            b'v',
            b'c',
            b'1',
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 0);
        assert_eq!(header.video_packet_type, VideoPacketType::SequenceStart);
        assert_eq!(header.content, ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Avc));
    }

    #[test]
    fn header_small_mod_ex() {
        let data = &[
            0b0000_0111, // type 7
            1,           // modex size 2
            42,          // modex data
            42,
            0b0001_0001, // type 1, next packet 1
            b'a',        // four cc
            b'v',
            b'c',
            b'1',
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 1);
        assert_eq!(
            header.video_packet_mod_exs[0],
            VideoPacketModEx::Other {
                video_packet_mod_ex_type: VideoPacketModExType(1),
                mod_ex_data: Bytes::from_static(&[42, 42])
            }
        );
        assert_eq!(header.video_packet_type, VideoPacketType::CodedFrames);
        assert_eq!(header.content, ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Avc));
    }

    #[test]
    fn header_multitrack_one_track() {
        let data = &[
            0b0000_0110, // type 6
            0b0000_0000, // one track, type 0
            b'a',        // four cc
            b'v',
            b'c',
            b'1',
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 0);
        assert_eq!(header.video_packet_type, VideoPacketType::SequenceStart);
        assert_eq!(header.content, ExVideoTagHeaderContent::OneTrack(VideoFourCc::Avc));
    }

    #[test]
    fn header_multitrack_many_tracks() {
        let data = &[
            0b0000_0110, // type 6
            0b0001_0000, // many tracks, type 0
            b'a',        // four cc
            b'v',
            b'c',
            b'1',
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 0);
        assert_eq!(header.video_packet_type, VideoPacketType::SequenceStart);
        assert_eq!(header.content, ExVideoTagHeaderContent::ManyTracks(VideoFourCc::Avc));
    }

    #[test]
    fn header_multitrack_many_tracks_many_codecs() {
        let data = &[
            0b0000_0110, // type 6
            0b0010_0000, // many tracks many codecs, type 0
            b'a',        // four cc, should be ignored
            b'v',
            b'c',
            b'1',
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 0);
        assert_eq!(header.video_packet_type, VideoPacketType::SequenceStart);
        assert_eq!(header.content, ExVideoTagHeaderContent::ManyTracksManyCodecs);
    }

    #[test]
    fn header_multitrack_unknown() {
        let data = &[
            0b0000_0110, // type 6
            0b0011_0000, // unknown, type 0
            b'a',        // four cc
            b'v',
            b'c',
            b'1',
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 0);
        assert_eq!(header.video_packet_type, VideoPacketType::SequenceStart);
        assert_eq!(
            header.content,
            ExVideoTagHeaderContent::Unknown {
                video_multitrack_type: AvMultitrackType(3),
                video_four_cc: VideoFourCc::Avc,
            }
        );
    }

    #[test]
    fn nested_multitrack_error() {
        let data = &[
            0b0000_0110, // type 6
            0b0000_0110, // one track, type 5
        ];

        let err = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap_err();
        assert!(matches!(err, FlvError::NestedMultitracks));
    }

    #[test]
    fn video_command() {
        let data = &[
            0b0101_0000, // frame type 5, type 0
            0,           // video command 0
            42,          // should be ignored
        ];

        let header = ExVideoTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.video_packet_mod_exs.len(), 0);
        assert_eq!(header.video_packet_type, VideoPacketType::SequenceStart);
        assert_eq!(header.content, ExVideoTagHeaderContent::VideoCommand(VideoCommand::StartSeek));
    }
}
