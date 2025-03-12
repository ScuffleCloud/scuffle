use std::io;

use body::VideoTagBody;
use bytes::Bytes;
use header::VideoTagHeader;

use crate::error::Error;

pub mod body;
pub mod header;

/// FLV `VIDEODATA` tag
///
/// This is a container for video data.
/// This enum contains the data for the different types of video tags.
/// Defined by:
/// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Video tags)
/// - video_file_format_spec_v10_1.pdf (Annex E.4.3.1 - VIDEODATA)
#[derive(Debug, Clone, PartialEq)]
pub struct VideoData {
    /// The header of the video data.
    pub header: VideoTagHeader,
    /// The body of the video data.
    pub body: VideoTagBody,
}

impl VideoData {
    /// Demux a video data from the given reader
    #[allow(clippy::unusual_byte_groupings)]
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let header = VideoTagHeader::demux(reader)?;
        let body = VideoTagBody::demux(&header, reader)?;

        Ok(VideoData { header, body })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_amf0::Amf0Marker;
    use scuffle_av1::AV1CodecConfigurationRecord;

    use super::header::enhanced::{VideoFourCc, VideoPacketType};
    use super::header::legacy::VideoCodecId;
    use super::header::{VideoCommand, VideoFrameType};
    use super::*;
    use crate::video::body::enhanced::metadata::VideoPacketMetadataEntry;
    use crate::video::body::enhanced::{ExVideoTagBody, VideoPacket, VideoPacketCodedFrames, VideoPacketSequenceStart};
    use crate::video::body::legacy::LegacyVideoTagBody;
    use crate::video::header::VideoTagHeaderData;
    use crate::video::header::enhanced::{ExVideoTagHeader, ExVideoTagHeaderContent};
    use crate::video::header::legacy::{AvcPacketType, LegacyVideoTagHeader, LegacyVideoTagHeaderAvcPacket};

    #[test]
    fn test_video_fourcc() {
        let cases = [
            (VideoFourCc::Av1, *b"av01", "VideoFourCc::Av1"),
            (VideoFourCc::Vp9, *b"vp09", "VideoFourCc::Vp9"),
            (VideoFourCc::Hevc, *b"hvc1", "VideoFourCc::Hevc"),
            (VideoFourCc(*b"av02"), *b"av02", "VideoFourCc([97, 118, 48, 50])"),
        ];

        for (expected, bytes, name) in cases {
            assert_eq!(VideoFourCc::from(bytes), expected);
            assert_eq!(format!("{:?}", VideoFourCc::from(bytes)), name);
        }
    }

    #[test]
    fn test_enhanced_packet_type() {
        let cases = [
            (VideoPacketType::SequenceStart, 0, "VideoPacketType::SequenceStart"),
            (VideoPacketType::CodedFrames, 1, "VideoPacketType::CodedFrames"),
            (VideoPacketType::SequenceEnd, 2, "VideoPacketType::SequenceEnd"),
            (VideoPacketType::CodedFramesX, 3, "VideoPacketType::CodedFramesX"),
            (VideoPacketType::Metadata, 4, "VideoPacketType::Metadata"),
            (
                VideoPacketType::Mpeg2TsSequenceStart,
                5,
                "VideoPacketType::Mpeg2TsSequenceStart",
            ),
            (VideoPacketType(6), 6, "VideoPacketType(6)"),
            (VideoPacketType(7), 7, "VideoPacketType(7)"),
        ];

        for (expected, value, name) in cases {
            assert_eq!(VideoPacketType::from(value), expected);
            assert_eq!(format!("{:?}", VideoPacketType::from(value)), name);
        }
    }

    #[test]
    fn test_frame_type() {
        let cases = [
            (VideoFrameType::KeyFrame, 1, "FrameType::KeyFrame"),
            (VideoFrameType::InterFrame, 2, "VideoFrameType::InterFrame"),
            (
                VideoFrameType::DisposableInterFrame,
                3,
                "VideoFrameType::DisposableInterFrame",
            ),
            (VideoFrameType::GeneratedKeyFrame, 4, "VideoFrameType::GeneratedKeyFrame"),
            (VideoFrameType::Command, 5, "VideoFrameType::Command"),
            (VideoFrameType(6), 6, "VideoFrameType(6)"),
            (VideoFrameType(7), 7, "VideoFrameType(7)"),
        ];

        for (expected, value, name) in cases {
            assert_eq!(VideoFrameType::from(value), expected);
            assert_eq!(format!("{:?}", VideoFrameType::from(value)), name);
        }
    }

    #[test]
    fn test_video_codec_id() {
        let cases = [
            (VideoCodecId::SorensonH263, 2, "VideoCodecId::SorensonH263"),
            (VideoCodecId::ScreenVideo, 3, "VideoCodecId::ScreenVideo"),
            (VideoCodecId::On2VP6, 4, "VideoCodecId::On2VP6"),
            (
                VideoCodecId::On2VP6WithAlphaChannel,
                5,
                "VideoCodecId::On2VP6WithAlphaChannel",
            ),
            (VideoCodecId::ScreenVideoVersion2, 6, "VideoCodecId::ScreenVideoVersion2"),
            (VideoCodecId::Avc, 7, "VideoCodecId::Avc"),
            (VideoCodecId(10), 10, "VideoCodecId(10)"),
            (VideoCodecId(11), 11, "VideoCodecId(11)"),
            (VideoCodecId(15), 15, "VideoCodecId(15)"),
        ];

        for (expected, value, name) in cases {
            assert_eq!(VideoCodecId::from(value), expected);
            assert_eq!(format!("{:?}", VideoCodecId::from(value)), name);
        }
    }

    #[test]
    fn test_command_packet() {
        let cases = [
            (VideoCommand::StartSeek, 1, "VideoCommand::StartSeek"),
            (VideoCommand::EndSeek, 2, "VideoCommand::EndSeek"),
            (VideoCommand(3), 3, "VideoCommand(3)"),
            (VideoCommand(4), 4, "VideoCommand(4)"),
        ];

        for (expected, value, name) in cases {
            assert_eq!(VideoCommand::from(value), expected);
            assert_eq!(format!("{:?}", VideoCommand::from(value)), name);
        }
    }

    #[test]
    fn test_video_data_body_metadata() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b1001_0100, // enhanced + keyframe + metadata
            1,
            2,
            3,
            4,
            Amf0Marker::String as u8,
            0,
            Amf0Marker::Object as u8,
            Amf0Marker::ObjectEnd as u8,
        ]));
        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Enhanced(ExVideoTagHeader {
                    video_packet_type: VideoPacketType::Metadata,
                    video_packet_mod_exs: vec![],
                    content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([1, 2, 3, 4]))
                })
            }
        );

        let VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
            video_four_cc: VideoFourCc([1, 2, 3, 4]),
            packet: VideoPacket::Metadata(metadata),
        }) = video.body
        else {
            panic!("unexpected body: {:?}", video.body);
        };

        assert_eq!(metadata.len(), 1);
        assert_eq!(
            metadata[0],
            VideoPacketMetadataEntry::Other {
                key: "".to_string(),
                object: vec![], // empty object
            }
        );
    }

    #[test]
    fn test_video_data_body_avc() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b0001_0111, // legacy + keyframe + avc packet
            0x01,        // nalu
            0x02,        // composition time
            0x03,
            0x04,
            0x05, // data
            0x06,
            0x07,
            0x08,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Legacy(LegacyVideoTagHeader::AvcPacket(LegacyVideoTagHeaderAvcPacket::Nalu {
                    composition_time: 0x020304
                }))
            }
        );

        assert_eq!(
            video.body,
            VideoTagBody::Legacy(LegacyVideoTagBody::Other {
                data: Bytes::from_static(&[0x05, 0x06, 0x07, 0x08])
            })
        );

        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b0001_0111, // legacy + keyframe + avc packet
            0x05,
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Legacy(LegacyVideoTagHeader::AvcPacket(LegacyVideoTagHeaderAvcPacket::Unknown {
                    avc_packet_type: AvcPacketType(0x05),
                    composition_time: 0x020304
                })),
            }
        );

        assert_eq!(
            video.body,
            VideoTagBody::Legacy(LegacyVideoTagBody::Other {
                data: Bytes::from_static(&[0x05, 0x06, 0x07, 0x08])
            })
        );
    }

    #[test]
    fn test_video_data_body_hevc() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b1001_0011, // enhanced + keyframe + coded frames x
            b'h',        // video codec
            b'v',
            b'c',
            b'1',
            0x01, // data
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Enhanced(ExVideoTagHeader {
                    video_packet_type: VideoPacketType::CodedFramesX,
                    video_packet_mod_exs: vec![],
                    content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([b'h', b'v', b'c', b'1'])),
                })
            }
        );

        assert_eq!(
            video.body,
            VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                video_four_cc: VideoFourCc([b'h', b'v', b'c', b'1']),
                packet: VideoPacket::CodedFramesX(Bytes::from_static(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])),
            })
        );

        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b1001_0011, // enhanced + keyframe + coded frames x
            b'h',        // video codec
            b'v',
            b'c',
            b'1',
            0x01, // data
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Enhanced(ExVideoTagHeader {
                    video_packet_type: VideoPacketType::CodedFramesX,
                    video_packet_mod_exs: vec![],
                    content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Hevc),
                })
            }
        );

        assert_eq!(
            video.body,
            VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                video_four_cc: VideoFourCc::Hevc,
                packet: VideoPacket::CodedFramesX(Bytes::from_static(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])),
            })
        );
    }

    #[test]
    fn test_video_data_body_av1() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b1001_0001, // enhanced + keyframe + coded frames
            b'a',        // video codec
            b'v',
            b'0',
            b'1',
            0x01, // data
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Enhanced(ExVideoTagHeader {
                    video_packet_type: VideoPacketType::Metadata,
                    video_packet_mod_exs: vec![],
                    content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Av1)
                })
            }
        );
        assert_eq!(
            video.body,
            VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                video_four_cc: VideoFourCc::Av1,
                packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
                ])))
            })
        );
    }

    #[test]
    fn test_video_data_command_packet() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b0101_0000, // legacy + command
            0x01,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::Command,
                data: VideoTagHeaderData::Legacy(LegacyVideoTagHeader::VideoCommand(VideoCommand::StartSeek))
            }
        );
        assert_eq!(video.body, VideoTagBody::Legacy(LegacyVideoTagBody::Command));
    }

    #[test]
    fn test_video_data_demux_enhanced() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b1001_0010, // enhanced + keyframe + SequenceEnd
            b'a',
            b'v',
            b'0',
            b'1', // video codec
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Enhanced(ExVideoTagHeader {
                    video_packet_type: VideoPacketType::SequenceEnd,
                    video_packet_mod_exs: vec![],
                    content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Av1)
                })
            }
        );

        assert_eq!(
            video.body,
            VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                video_four_cc: VideoFourCc::Av1,
                packet: VideoPacket::SequenceEnd
            })
        );
    }

    #[test]
    fn test_video_data_demux_h263() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b0001_0010, // legacy + keyframe + SorensonH263
            0,           // data
            1,
            2,
            3,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Legacy(LegacyVideoTagHeader::Other {
                    video_codec_id: VideoCodecId::SorensonH263
                })
            }
        );
        assert_eq!(
            video.body,
            VideoTagBody::Legacy(LegacyVideoTagBody::Other {
                data: Bytes::from_static(&[0, 1, 2, 3])
            })
        );
    }

    #[test]
    fn test_av1_mpeg2_sequence_start() {
        let mut reader = io::Cursor::new(Bytes::from_static(&[
            0b1001_0101, // enhanced + keyframe + MPEG2TSSequenceStart
            b'a',
            b'v',
            b'0',
            b'1', // video codec
            0x80,
            0x4,
            129,
            13,
            12,
            0,
            10,
            15,
            0,
            0,
            0,
            106,
            239,
            191,
            225,
            188,
            2,
            25,
            144,
            16,
            16,
            16,
            64,
        ]));

        let video = VideoData::demux(&mut reader).unwrap();

        assert_eq!(
            video.header,
            VideoTagHeader {
                frame_type: VideoFrameType::KeyFrame,
                data: VideoTagHeaderData::Enhanced(ExVideoTagHeader {
                    video_packet_type: VideoPacketType::SequenceStart,
                    video_packet_mod_exs: vec![],
                    content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Av1),
                })
            }
        );

        assert_eq!(
            video.body,
            VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                video_four_cc: VideoFourCc::Av1,
                packet: VideoPacket::SequenceStart(VideoPacketSequenceStart::Av1(AV1CodecConfigurationRecord {
                    seq_profile: 0,
                    seq_level_idx_0: 13,
                    seq_tier_0: false,
                    high_bitdepth: false,
                    twelve_bit: false,
                    monochrome: false,
                    chroma_subsampling_x: true,
                    chroma_subsampling_y: true,
                    chroma_sample_position: 0,
                    hdr_wcg_idc: 0,
                    initial_presentation_delay_minus_one: None,
                    config_obu: Bytes::from_static(b"\n\x0f\0\0\0j\xef\xbf\xe1\xbc\x02\x19\x90\x10\x10\x10@"),
                }))
            }),
        );
    }
}
