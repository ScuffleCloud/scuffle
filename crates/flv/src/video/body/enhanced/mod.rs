//! Enhanced video tag body
//!
//! Types and functions defined by the enhanced RTMP spec, page 29-31, ExVideoTagBody.

use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use metadata::VideoPacketMetadataEntry;
use scuffle_amf0::decoder::Amf0Decoder;
use scuffle_av1::{AV1CodecConfigurationRecord, AV1VideoDescriptor};
use scuffle_bytes_util::BytesCursorExt;
use scuffle_h264::AVCDecoderConfigurationRecord;
use scuffle_h265::HEVCDecoderConfigurationRecord;

use crate::error::FlvError;
use crate::video::header::enhanced::{ExVideoTagHeader, ExVideoTagHeaderContent, VideoFourCc, VideoPacketType};

pub mod metadata;

/// Sequence start video packet
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketSequenceStart {
    /// Av1 codec configuration record
    Av1(AV1CodecConfigurationRecord),
    /// H.264/AVC codec configuration record
    Avc(AVCDecoderConfigurationRecord),
    /// H.265/HEVC codec configuration record
    Hevc(HEVCDecoderConfigurationRecord),
    /// Other codecs like VP8 and VP9
    Other(Bytes),
}

/// MPEG2-TS sequence start video packet
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketMpeg2TsSequenceStart {
    /// Av1 video descriptor
    Av1(AV1VideoDescriptor),
    /// Any other codecs
    Other(Bytes),
}

/// Coded frames video packet
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketCodedFrames {
    /// H.264/AVC coded frames
    Avc {
        /// Composition time offset
        composition_time_offset: i32,
        /// Data
        data: Bytes,
    },
    /// H.265/HEVC coded frames
    Hevc {
        /// Composition time offset
        composition_time_offset: i32,
        /// Data
        data: Bytes,
    },
    /// Coded frames of any other codec
    Other(Bytes),
}

/// Video packet
///
/// Appears as part of the [`ExVideoTagBody`].
///
/// Defined by:
/// - Enhanced RTMP spec, page 29-31, ExVideoTagBody
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacket<'a> {
    /// Metadata
    Metadata(Vec<VideoPacketMetadataEntry<'a>>),
    /// Indicates the end of a sequence of video packets.
    SequenceEnd,
    /// Indicates the start of a sequence of video packets.
    SequenceStart(VideoPacketSequenceStart),
    /// Indicates the start of a sequence of video packets in MPEG2-TS format.
    Mpeg2TsSequenceStart(VideoPacketMpeg2TsSequenceStart),
    /// Coded video frames.
    CodedFrames(VideoPacketCodedFrames),
    /// Coded video frames without extra data.
    CodedFramesX {
        /// The video data.
        data: Bytes,
    },
    /// An unknown [`VideoPacketType`].
    Unknown {
        /// The unknown packet type.
        video_packet_type: VideoPacketType,
        /// The data.
        data: Bytes,
    },
}

impl VideoPacket<'_> {
    /// Demux a [`VideoPacket`] from the given reader.
    ///
    /// This is implemented as per spec, Enhanced RTMP page 29-31, ExVideoTagBody.
    pub fn demux(
        header: &ExVideoTagHeader,
        video_four_cc: VideoFourCc,
        reader: &mut io::Cursor<Bytes>,
    ) -> Result<Self, FlvError> {
        let size_of_video_track = if !matches!(
            header.content,
            ExVideoTagHeaderContent::NoMultiTrack(_) | ExVideoTagHeaderContent::OneTrack(_)
        ) {
            Some(reader.read_u24::<BigEndian>()? as usize)
        } else {
            None
        };

        match header.video_packet_type {
            VideoPacketType::Metadata => {
                let data = reader.extract_bytes(size_of_video_track.unwrap_or(reader.remaining()))?;
                let mut decoder = Amf0Decoder::from_buf(data);

                let metadata = decoder
                    .deserialize_stream::<metadata::VideoPacketMetadataEntry>()
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Self::Metadata(metadata))
            }
            VideoPacketType::SequenceEnd => Ok(Self::SequenceEnd),
            VideoPacketType::SequenceStart => {
                let data = reader.extract_bytes(size_of_video_track.unwrap_or(reader.remaining()))?;

                let seq_start = match video_four_cc {
                    VideoFourCc::Av1 => {
                        let record = AV1CodecConfigurationRecord::demux(&mut io::Cursor::new(data))?;
                        VideoPacketSequenceStart::Av1(record)
                    }
                    VideoFourCc::Avc => {
                        let record = AVCDecoderConfigurationRecord::parse(&mut io::Cursor::new(data))?;
                        VideoPacketSequenceStart::Avc(record)
                    }
                    VideoFourCc::Hevc => {
                        let record = HEVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data))?;
                        VideoPacketSequenceStart::Hevc(record)
                    }
                    _ => VideoPacketSequenceStart::Other(data),
                };

                Ok(Self::SequenceStart(seq_start))
            }
            VideoPacketType::Mpeg2TsSequenceStart => {
                let data = reader.extract_bytes(size_of_video_track.unwrap_or(reader.remaining()))?;

                let seq_start = match video_four_cc {
                    VideoFourCc::Av1 => {
                        let descriptor = AV1VideoDescriptor::demux(&mut io::Cursor::new(data))?;
                        VideoPacketMpeg2TsSequenceStart::Av1(descriptor)
                    }
                    _ => VideoPacketMpeg2TsSequenceStart::Other(data),
                };

                Ok(Self::Mpeg2TsSequenceStart(seq_start))
            }
            VideoPacketType::CodedFrames => {
                let coded_frames = match video_four_cc {
                    VideoFourCc::Avc => {
                        let composition_time_offset = reader.read_i24::<BigEndian>()?;
                        let data = reader
                            .extract_bytes(size_of_video_track.map(|s| s.saturating_sub(3)).unwrap_or(reader.remaining()))?;

                        VideoPacketCodedFrames::Avc {
                            composition_time_offset,
                            data,
                        }
                    }
                    VideoFourCc::Hevc => {
                        let composition_time_offset = reader.read_i24::<BigEndian>()?;
                        let data = reader
                            .extract_bytes(size_of_video_track.map(|s| s.saturating_sub(3)).unwrap_or(reader.remaining()))?;

                        VideoPacketCodedFrames::Hevc {
                            composition_time_offset,
                            data,
                        }
                    }
                    _ => {
                        let data = reader.extract_bytes(size_of_video_track.unwrap_or(reader.remaining()))?;

                        VideoPacketCodedFrames::Other(data)
                    }
                };

                Ok(Self::CodedFrames(coded_frames))
            }
            VideoPacketType::CodedFramesX => {
                let data = reader.extract_bytes(size_of_video_track.unwrap_or(reader.remaining()))?;

                Ok(Self::CodedFramesX { data })
            }
            _ => {
                let data = reader.extract_bytes(size_of_video_track.unwrap_or(reader.remaining()))?;

                Ok(Self::Unknown {
                    video_packet_type: header.video_packet_type,
                    data,
                })
            }
        }
    }
}

/// One video track contained in a multitrack video.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTrack<'a> {
    /// The video FOURCC of this track.
    pub video_four_cc: VideoFourCc,
    /// The video track ID.
    ///
    /// > For identifying the highest priority (a.k.a., default track)
    /// > or highest quality track, it is RECOMMENDED to use trackId
    /// > set to zero. For tracks of lesser priority or quality, use
    /// > multiple instances of trackId with ascending numerical values.
    /// > The concept of priority or quality can have multiple
    /// > interpretations, including but not limited to bitrate,
    /// > resolution, default angle, and language. This recommendation
    /// > serves as a guideline intended to standardize track numbering
    /// > across various applications.
    pub video_track_id: u8,
    /// The video packet contained in this track.
    pub packet: VideoPacket<'a>,
}

/// `ExVideoTagBody`
///
/// Defined by:
/// - Enhanced RTMP spec, page 29-31, ExVideoTagBody
#[derive(Debug, Clone, PartialEq)]
pub enum ExVideoTagBody<'a> {
    /// Empty body because the header contains a [`VideoCommand`](crate::video::header::VideoCommand).
    Command,
    /// The body is not a multitrack body.
    NoMultitrack {
        /// The video FOURCC of this body.
        video_four_cc: VideoFourCc,
        /// The video packet contained in this body.
        packet: VideoPacket<'a>,
    },
    /// The body is a multitrack body.
    ///
    /// This variant contains multiple video tracks.
    /// See [`VideoTrack`] for more information.
    ManyTracks(Vec<VideoTrack<'a>>),
}

impl ExVideoTagBody<'_> {
    /// Demux an [`ExVideoTagBody`] from the given reader.
    ///
    /// This is implemented as per Enhanced RTMP spec, page 29-31, ExVideoTagBody.
    pub fn demux(header: &ExVideoTagHeader, reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
        let mut tracks = Vec::new();

        loop {
            let video_four_cc = match header.content {
                ExVideoTagHeaderContent::VideoCommand(_) => return Ok(ExVideoTagBody::Command),
                ExVideoTagHeaderContent::ManyTracksManyCodecs => {
                    let mut video_four_cc = [0; 4];
                    reader.read_exact(&mut video_four_cc)?;
                    VideoFourCc::from(video_four_cc)
                }
                ExVideoTagHeaderContent::OneTrack(video_four_cc) => video_four_cc,
                ExVideoTagHeaderContent::ManyTracks(video_four_cc) => video_four_cc,
                ExVideoTagHeaderContent::NoMultiTrack(video_four_cc) => video_four_cc,
                ExVideoTagHeaderContent::Unknown { video_four_cc, .. } => video_four_cc,
            };

            let video_track_id = if !matches!(header.content, ExVideoTagHeaderContent::NoMultiTrack(_)) {
                Some(reader.read_u8()?)
            } else {
                None
            };

            let packet = VideoPacket::demux(header, video_four_cc, reader)?;

            if let Some(video_track_id) = video_track_id {
                // video_track_id is only set if this is a multitrack video, in other words, if `isVideoMultitrack` is true
                tracks.push(VideoTrack {
                    video_four_cc,
                    video_track_id,
                    packet,
                });

                // the loop only continues if there is still data to read and this is a video with multiple tracks
                if !matches!(header.content, ExVideoTagHeaderContent::OneTrack(_)) && reader.has_remaining() {
                    continue;
                }

                break;
            } else {
                // exit early if this is a single track video only completing one loop iteration
                return Ok(Self::NoMultitrack { video_four_cc, packet });
            }
        }

        // at this point we know this is a multitrack video because a single track video would have exited early
        Ok(Self::ManyTracks(tracks))
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;

    use crate::common::AvMultitrackType;
    use crate::video::body::enhanced::{
        ExVideoTagBody, VideoPacket, VideoPacketCodedFrames, VideoPacketMpeg2TsSequenceStart, VideoPacketSequenceStart,
        VideoTrack,
    };
    use crate::video::header::VideoCommand;
    use crate::video::header::enhanced::{ExVideoTagHeader, ExVideoTagHeaderContent, VideoFourCc, VideoPacketType};

    #[test]
    fn simple_video_packets_demux() {
        let data = &[42, 42, 42, 42];

        let packet = VideoPacket::demux(
            &ExVideoTagHeader {
                video_packet_mod_exs: vec![],
                video_packet_type: VideoPacketType::SequenceStart,
                content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([0, 0, 0, 0])),
            },
            VideoFourCc([0, 0, 0, 0]),
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(
            packet,
            VideoPacket::SequenceStart(VideoPacketSequenceStart::Other(Bytes::from_static(data))),
        );

        let packet = VideoPacket::demux(
            &ExVideoTagHeader {
                video_packet_mod_exs: vec![],
                video_packet_type: VideoPacketType::CodedFrames,
                content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([0, 0, 0, 0])),
            },
            VideoFourCc([0, 0, 0, 0]),
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(
            packet,
            VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(data))),
        );

        let packet = VideoPacket::demux(
            &ExVideoTagHeader {
                video_packet_mod_exs: vec![],
                video_packet_type: VideoPacketType::SequenceEnd,
                content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([0, 0, 0, 0])),
            },
            VideoFourCc([0, 0, 0, 0]),
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(packet, VideoPacket::SequenceEnd);

        let packet = VideoPacket::demux(
            &ExVideoTagHeader {
                video_packet_mod_exs: vec![],
                video_packet_type: VideoPacketType(8),
                content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([0, 0, 0, 0])),
            },
            VideoFourCc([0, 0, 0, 0]),
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(
            packet,
            VideoPacket::Unknown {
                video_packet_type: VideoPacketType(8),
                data: Bytes::from_static(data),
            },
        );
    }

    #[test]
    fn video_packet_with_size_demux() {
        let data = &[
            0, 0, 5, // size
            0, 0, 1, // composition time offset
            42, 42, // data
            13, 37, // should be ignored
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::CodedFrames,
            content: ExVideoTagHeaderContent::ManyTracks(VideoFourCc::Avc),
        };

        let packet =
            VideoPacket::demux(&header, VideoFourCc::Avc, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            VideoPacket::CodedFrames(VideoPacketCodedFrames::Avc {
                composition_time_offset: 1,
                data: Bytes::from_static(&[42, 42]),
            }),
        );
    }

    #[test]
    fn video_packet_mpeg2_ts_demux() {
        let data = &[
            42, 42, // data
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::Mpeg2TsSequenceStart,
            content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc::Avc),
        };

        let packet =
            VideoPacket::demux(&header, VideoFourCc::Avc, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            VideoPacket::Mpeg2TsSequenceStart(VideoPacketMpeg2TsSequenceStart::Other(Bytes::from_static(data))),
        );
    }

    #[test]
    fn simple_body_demux() {
        let data = &[
            42, 42, // data
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::CodedFrames,
            content: ExVideoTagHeaderContent::NoMultiTrack(VideoFourCc([0, 0, 0, 0])),
        };

        let packet = ExVideoTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExVideoTagBody::NoMultitrack {
                video_four_cc: VideoFourCc([0, 0, 0, 0]),
                packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(data))),
            },
        );
    }

    #[test]
    fn multitrack_many_codecs_body_demux() {
        let data = &[
            0, 0, 0, 0, // video four cc
            1, // video track id
            0, 0, 2, // size
            42, 42, // data
            0, 1, 0, 1, // video four cc
            2, // video track id
            0, 0, 2, // size
            13, 37, // data
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::CodedFrames,
            content: ExVideoTagHeaderContent::ManyTracksManyCodecs,
        };

        let packet = ExVideoTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExVideoTagBody::ManyTracks(vec![
                VideoTrack {
                    video_four_cc: VideoFourCc([0, 0, 0, 0]),
                    video_track_id: 1,
                    packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[42, 42]))),
                },
                VideoTrack {
                    video_four_cc: VideoFourCc([0, 1, 0, 1]),
                    video_track_id: 2,
                    packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[13, 37]))),
                }
            ]),
        );
    }

    #[test]
    fn multitrack_body_demux() {
        let data = &[
            1, // video track id
            0, 0, 2, // size
            42, 42, // data
            2,  // video track id
            0, 0, 2, // size
            13, 37, // data
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::CodedFrames,
            content: ExVideoTagHeaderContent::ManyTracks(VideoFourCc([0, 0, 0, 0])),
        };

        let packet = ExVideoTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExVideoTagBody::ManyTracks(vec![
                VideoTrack {
                    video_four_cc: VideoFourCc([0, 0, 0, 0]),
                    video_track_id: 1,
                    packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[42, 42]))),
                },
                VideoTrack {
                    video_four_cc: VideoFourCc([0, 0, 0, 0]),
                    video_track_id: 2,
                    packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[13, 37]))),
                }
            ]),
        );
    }

    #[test]
    fn multitrack_one_track_body_demux() {
        let data = &[
            1, // video track id
            42, 42, // data
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::CodedFrames,
            content: ExVideoTagHeaderContent::OneTrack(VideoFourCc([0, 0, 0, 0])),
        };

        let packet = ExVideoTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExVideoTagBody::ManyTracks(vec![VideoTrack {
                video_four_cc: VideoFourCc([0, 0, 0, 0]),
                video_track_id: 1,
                packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[42, 42]))),
            }]),
        );
    }

    #[test]
    fn multitrack_unknown_body_demux() {
        let data = &[
            1, // video track id
            0, 0, 2, // size
            42, 42, // data
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::CodedFrames,
            content: ExVideoTagHeaderContent::Unknown {
                video_four_cc: VideoFourCc([0, 0, 0, 0]),
                video_multitrack_type: AvMultitrackType(4),
            },
        };

        let packet = ExVideoTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExVideoTagBody::ManyTracks(vec![VideoTrack {
                video_track_id: 1,
                video_four_cc: VideoFourCc([0, 0, 0, 0]),
                packet: VideoPacket::CodedFrames(VideoPacketCodedFrames::Other(Bytes::from_static(&[42, 42]))),
            }]),
        );
    }

    #[test]
    fn video_command() {
        let data = &[
            42, // should be ignored
        ];

        let header = ExVideoTagHeader {
            video_packet_mod_exs: vec![],
            video_packet_type: VideoPacketType::SequenceStart,
            content: ExVideoTagHeaderContent::VideoCommand(VideoCommand::StartSeek),
        };

        let packet = ExVideoTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(packet, ExVideoTagBody::Command);
    }
}
