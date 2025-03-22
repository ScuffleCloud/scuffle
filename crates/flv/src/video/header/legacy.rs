//! Legacy video header types and functions.

use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;

use super::{VideoCommand, VideoFrameType};

nutype_enum! {
    /// FLV Video Codec ID
    ///
    /// Denotes the different types of video codecs.
    ///
    /// Defined by:
    /// - Legacy FLV spec, Annex E.4.3.1
    pub enum VideoCodecId(u8) {
        /// Sorenson H.263
        SorensonH263 = 2,
        /// Screen Video
        ScreenVideo = 3,
        /// On2 VP6
        On2VP6 = 4,
        /// On2 VP6 with alpha channel
        On2VP6WithAlphaChannel = 5,
        /// Screen Video Version 2
        ScreenVideoVersion2 = 6,
        /// AVC (H.264)
        Avc = 7,
    }
}

nutype_enum! {
    /// FLV AVC Packet Type
    ///
    /// The AVC packet type is used to determine if the video data is a sequence
    /// header or a NALU.
    ///
    /// Defined by:
    /// - Legacy FLV spec, Annex E.4.3.1
    pub enum AvcPacketType(u8) {
        /// AVC sequence header
        SeqHdr = 0,
        /// AVC NALU
        Nalu = 1,
        /// AVC end of sequence (lower level NALU sequence ender is not required or supported)
        EndOfSequence = 2,
    }
}

/// AVC packet header
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyVideoTagHeaderAvcPacket {
    /// AVC sequence header
    SequenceHeader,
    /// AVC NALU
    Nalu {
        /// The composition time offset of the NALU.
        composition_time_offset: u32,
    },
    /// AVC end of sequence
    EndOfSequence,
    /// Unknown
    Unknown {
        /// The AVC packet type.
        avc_packet_type: AvcPacketType,
        /// The composition time offset of the packet.
        composition_time_offset: u32,
    },
}

impl LegacyVideoTagHeaderAvcPacket {
    /// Demux the AVC packet header from the given reader.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let avc_packet_type = AvcPacketType::from(reader.read_u8()?);
        let composition_time_offset = reader.read_u24::<BigEndian>()?;

        match avc_packet_type {
            AvcPacketType::SeqHdr => Ok(Self::SequenceHeader),
            AvcPacketType::Nalu => Ok(Self::Nalu { composition_time_offset }),
            AvcPacketType::EndOfSequence => Ok(Self::EndOfSequence),
            _ => Ok(Self::Unknown {
                avc_packet_type,
                composition_time_offset,
            }),
        }
    }
}

/// FLV `VideoTagHeader`
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.3.1
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyVideoTagHeader {
    /// A video command with frame type [`VideoFrameType::Command`].
    VideoCommand(VideoCommand),
    /// AVC video packet.
    AvcPacket(LegacyVideoTagHeaderAvcPacket),
    /// Any other video data.
    Other {
        /// The codec id of the video data.
        video_codec_id: VideoCodecId,
    },
}

impl LegacyVideoTagHeader {
    /// Demux the video tag header from the given reader.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let first_byte = reader.read_u8()?;
        let frame_type = VideoFrameType::from(first_byte >> 4); // 0b1111_0000
        let video_codec_id = VideoCodecId::from(first_byte & 0b0000_1111);

        if video_codec_id == VideoCodecId::Avc {
            let avc_packet = LegacyVideoTagHeaderAvcPacket::demux(reader)?;
            return Ok(Self::AvcPacket(avc_packet));
        }

        if frame_type == VideoFrameType::Command {
            return Ok(Self::VideoCommand(VideoCommand::from(reader.read_u8()?)));
        }

        Ok(Self::Other { video_codec_id })
    }
}
