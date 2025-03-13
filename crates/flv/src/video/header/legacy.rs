use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;

use super::{VideoCommand, VideoFrameType};

nutype_enum! {
    /// FLV Video Codec ID
    ///
    /// Denotes the different types of video codecs that can be used in a FLV file.
    /// This is a legacy enum for older codecs; for modern codecs, the [`EnhancedPacketType`] is used which uses a [`VideoFourCC`] identifier.
    ///
    /// Defined by:
    /// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Video tags)
    /// - video_file_format_spec_v10_1.pdf (Annex E.4.3.1 - VIDEODATA)
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
    /// Defined in the FLV specification. Chapter 1 - AVCVIDEODATA
    /// The AVC packet type is used to determine if the video data is a sequence
    /// header or a NALU.
    pub enum AvcPacketType(u8) {
        SeqHdr = 0,
        Nalu = 1,
        EndOfSequence = 2,
    }
}

/// AVC Packet header
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyVideoTagHeaderAvcPacket {
    /// AVC Sequence Header
    SequenceHeader,
    /// AVC NALU
    Nalu { composition_time: u32 },
    /// AVC End of Sequence
    EndOfSequence,
    /// AVC Unknown (we don't know how to parse it)
    Unknown {
        avc_packet_type: AvcPacketType,
        composition_time: u32,
    },
}

impl LegacyVideoTagHeaderAvcPacket {
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let avc_packet_type = AvcPacketType::from(reader.read_u8()?);
        let composition_time = reader.read_u24::<BigEndian>()?;

        match avc_packet_type {
            AvcPacketType::SeqHdr => Ok(Self::SequenceHeader),
            AvcPacketType::Nalu => Ok(Self::Nalu { composition_time }),
            AvcPacketType::EndOfSequence => Ok(Self::EndOfSequence),
            _ => Ok(Self::Unknown {
                avc_packet_type,
                composition_time,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LegacyVideoTagHeader {
    /// A video command with frame type `VideoFrameType::Command`.
    VideoCommand(VideoCommand),
    AvcPacket(LegacyVideoTagHeaderAvcPacket),
    Other {
        /// The codec id of the video data. (4 bits)
        video_codec_id: VideoCodecId,
    },
}

impl LegacyVideoTagHeader {
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
