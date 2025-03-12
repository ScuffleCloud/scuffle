use std::io;

use bytes::Bytes;
use scuffle_bytes_util::BytesCursorExt;
use scuffle_h264::AVCDecoderConfigurationRecord;

use crate::video::header::legacy::{LegacyVideoTagHeader, LegacyVideoTagHeaderAvcPacket};

/// Legacy FLV `VideoTagBody`
///
/// This is a container for video data.
/// This enum contains the data for the different types of video tags.
///
/// Defined by:
/// - video_file_format_spec_v10.pdf (Chapter 1 - The FLV File Format - Video
///   tags)
/// - video_file_format_spec_v10_1.pdf (Annex E.4.3.1 - VIDEODATA)
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyVideoTagBody {
    /// Empty body because the header contains a [`VideoCommand`](crate::video::header::VideoCommand)
    Command,
    AvcVideoPacketSeqHdr(AVCDecoderConfigurationRecord),
    Other {
        data: Bytes,
    },
}

impl LegacyVideoTagBody {
    /// Demux a video packet from the given reader.
    /// The reader will consume all the data from the reader.
    pub fn demux(header: &LegacyVideoTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        match header {
            LegacyVideoTagHeader::VideoCommand(_) => Ok(Self::Command),
            LegacyVideoTagHeader::AvcPacket(LegacyVideoTagHeaderAvcPacket::SequenceHeader) => {
                // AVCVIDEOPACKET
                let avc_decoder_configuration_record = AVCDecoderConfigurationRecord::parse(reader)?;
                Ok(Self::AvcVideoPacketSeqHdr(avc_decoder_configuration_record))
            }
            _ => Ok(Self::Other {
                data: reader.extract_remaining(),
            }),
        }
    }
}
