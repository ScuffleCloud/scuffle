use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use metadata::VideoPacketMetadataEntry;
use scuffle_amf0::Amf0Decoder;
use scuffle_av1::{AV1CodecConfigurationRecord, AV1VideoDescriptor};
use scuffle_bytes_util::BytesCursorExt;
use scuffle_h264::AVCDecoderConfigurationRecord;
use scuffle_h265::HEVCDecoderConfigurationRecord;

use crate::error::Error;
use crate::video::header::enhanced::{ExVideoTagHeader, ExVideoTagHeaderContent, VideoFourCc, VideoPacketType};

pub mod metadata;

#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketSequenceStart {
    Av1(AV1CodecConfigurationRecord),
    Avc(AVCDecoderConfigurationRecord),
    Hevc(HEVCDecoderConfigurationRecord),
    /// For unsupported codecs like VP8 and VP9
    Other(Bytes),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketMpeg2TsSequenceStart {
    Av1(AV1VideoDescriptor),
    Other(Bytes),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketCodedFrames {
    Avc { composition_time_offset: i32, data: Bytes },
    Hevc { composition_time_offset: i32, data: Bytes },
    Other(Bytes),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacket {
    Metadata(Vec<VideoPacketMetadataEntry>),
    SequenceEnd,
    SequenceStart(VideoPacketSequenceStart),
    Mpeg2TsSequenceStart(VideoPacketMpeg2TsSequenceStart),
    CodedFrames(VideoPacketCodedFrames),
    CodedFramesX(Bytes),
    Other { packet_type: VideoPacketType, data: Bytes },
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoTrack {
    pub video_four_cc: VideoFourCc,
    pub video_track_id: u8,
    pub packet: VideoPacket,
}

/// An Enhanced FLV Packet
///
/// This is a container for enhanced video packets.
/// The enchanced spec adds modern codecs to the FLV file format.
///
/// Defined by:
/// - enhanced_rtmp-v1.pdf (Defining Additional Video Codecs)
/// - enhanced_rtmp-v2.pdf (Enhanced Video)
#[derive(Debug, Clone, PartialEq)]
pub enum ExVideoTagBody {
    /// Empty body because the header contains a [`VideoCommand`](crate::video::header::VideoCommand)
    Command,
    NoMultitrack {
        video_four_cc: VideoFourCc,
        packet: VideoPacket,
    },
    ManyTracks(Vec<VideoTrack>),
}

impl ExVideoTagBody {
    pub fn demux(header: &ExVideoTagHeader, reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let mut tracks = Vec::new();

        let is_video_multitrack = !matches!(header.content, ExVideoTagHeaderContent::NoMultiTrack(_));

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
                ExVideoTagHeaderContent::Other { video_four_cc, .. } => video_four_cc,
            };

            let video_track_id = if is_video_multitrack { Some(reader.read_u8()?) } else { None };

            let size_of_video_track =
                if is_video_multitrack && !matches!(header.content, ExVideoTagHeaderContent::OneTrack(_)) {
                    Some(reader.read_u24::<BigEndian>()?)
                } else {
                    None
                };

            let packet = match header.video_packet_type {
                VideoPacketType::Metadata => {
                    let data =
                        reader.extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;
                    let mut amf_reader = Amf0Decoder::new(&data);

                    let mut metadata = Vec::new();

                    while !amf_reader.is_empty() {
                        metadata.push(metadata::VideoPacketMetadataEntry::read(&mut amf_reader)?);
                    }

                    VideoPacket::Metadata(metadata)
                }
                VideoPacketType::SequenceEnd => VideoPacket::SequenceEnd,
                VideoPacketType::SequenceStart => {
                    let data =
                        reader.extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

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

                    VideoPacket::SequenceStart(seq_start)
                }
                VideoPacketType::Mpeg2TsSequenceStart => {
                    let data =
                        reader.extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

                    let seq_start = match video_four_cc {
                        VideoFourCc::Av1 => {
                            let descriptor = AV1VideoDescriptor::demux(&mut io::Cursor::new(data))?;
                            VideoPacketMpeg2TsSequenceStart::Av1(descriptor)
                        }
                        _ => VideoPacketMpeg2TsSequenceStart::Other(data),
                    };

                    VideoPacket::Mpeg2TsSequenceStart(seq_start)
                }
                VideoPacketType::CodedFrames => {
                    let coded_frames = match video_four_cc {
                        VideoFourCc::Avc => {
                            let composition_time_offset = reader.read_i24::<BigEndian>()?;
                            let data = reader
                                .extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

                            VideoPacketCodedFrames::Avc {
                                composition_time_offset,
                                data,
                            }
                        }
                        VideoFourCc::Hevc => {
                            let composition_time_offset = reader.read_i24::<BigEndian>()?;
                            let data = reader
                                .extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

                            VideoPacketCodedFrames::Hevc {
                                composition_time_offset,
                                data,
                            }
                        }
                        _ => {
                            let data = reader
                                .extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

                            VideoPacketCodedFrames::Other(data)
                        }
                    };

                    VideoPacket::CodedFrames(coded_frames)
                }
                VideoPacketType::CodedFramesX => {
                    let data =
                        reader.extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;
                    VideoPacket::CodedFramesX(data)
                }
                packet_type => {
                    let data =
                        reader.extract_bytes(size_of_video_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;
                    VideoPacket::Other { packet_type, data }
                }
            };

            if let Some(video_track_id) = video_track_id {
                // video_track_id is only set if this is a multitrack video, in other words, if this is true:
                // `isVideoMultitrack && videoMultitrackType != AvMultitrackType.OneTrack`
                tracks.push(VideoTrack {
                    video_four_cc,
                    video_track_id,
                    packet,
                });
            } else {
                // exit early if this is a single track video only completing one loop iteration
                return Ok(Self::NoMultitrack { video_four_cc, packet });
            }

            // the loop only continues if there is still data to read
            if !reader.has_remaining() {
                break;
            }
        }

        // at this point we know this is a multitrack video because a single track video would have exited early
        Ok(Self::ManyTracks(tracks))
    }
}
