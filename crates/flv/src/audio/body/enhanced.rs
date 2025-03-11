use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::header::{AudioFourCc, AudioPacketType, ExAudioTagHeader, MultitrackTypeAndFourCc};

nutype_enum! {
    pub enum AudioChannelOrder(u8) {
        Unspecified = 0,
        Native = 1,
        Custom = 2,
    }
}

nutype_enum! {
    /// Channel mappings enum
    ///
    /// commonly used speaker configurations
    ///
    /// see - <https://en.wikipedia.org/wiki/Surround_sound#Standard_speaker_channels>
    pub enum AudioChannel(u8) {
        FrontLeft = 0,
        FrontRight = 1,
        FrontCenter = 2,
        LowFrequency1 = 3,
        BackLeft = 4,
        BackRight = 5,
        FrontLeftCenter = 6,
        FrontRightCenter = 7,
        BackCenter = 8,
        SideLeft = 9,
        SideRight = 10,
        TopCenter = 11,
        TopFrontLeft = 12,
        TopFrontCenter = 13,
        TopFrontRight = 14,
        TopBackLeft = 15,
        TopBackCenter = 16,
        TopBackRight = 17,
        LowFrequency2 = 18,
        TopSideLeft = 19,
        TopSideRight = 20,
        BottomFrontCenter = 21,
        BottomFrontLeft = 22,
        BottomFrontRight = 23,
        Unused = 0xfe,
        Unknown = 0xff,
    }
}

#[bitmask_enum::bitmask(u32)]
pub enum AudioChannelMask {
    FrontLeft = 0x000001,
    FrontRight = 0x000002,
    FrontCenter = 0x000004,
    LowFrequency1 = 0x000008,
    BackLeft = 0x000010,
    BackRight = 0x000020,
    FrontLeftCenter = 0x000040,
    FrontRightCenter = 0x000080,
    BackCenter = 0x000100,
    SideLeft = 0x000200,
    SideRight = 0x000400,
    TopCenter = 0x000800,
    TopFrontLeft = 0x001000,
    TopFrontCenter = 0x002000,
    TopFrontRight = 0x004000,
    TopBackLeft = 0x008000,
    TopBackCenter = 0x010000,
    TopBackRight = 0x020000,
    LowFrequency2 = 0x040000,
    TopSideLeft = 0x080000,
    TopSideRight = 0x100000,
    BottomFrontCenter = 0x200000,
    BottomFrontLeft = 0x400000,
    BottomFrontRight = 0x800000,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MultichannelConfig {
    Custom(Vec<AudioChannel>),
    Native(AudioChannelMask),
    Unspecified,
    Unknown(AudioChannelOrder),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioPacket {
    MultichannelConfig {
        channel_count: u8,
        multichannel_config: MultichannelConfig,
    },
    SequenceEnd,
    SequenceStart {
        header_data: Bytes,
    },
    CodedFrames {
        data: Bytes,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTrack {
    pub audio_four_cc: AudioFourCc,
    pub audio_track_id: u8,
    pub packet: AudioPacket,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnhancedAudioTagBody {
    pub tracks: Vec<AudioTrack>,
}

impl EnhancedAudioTagBody {
    pub fn demux(header: &ExAudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let mut tracks = Vec::new();

        let has_one_track = matches!(
            header.multitrack_type_and_four_cc,
            MultitrackTypeAndFourCc::NoMultiTrack(_) | MultitrackTypeAndFourCc::OneTrack(_)
        );

        loop {
            let audio_four_cc = match header.multitrack_type_and_four_cc {
                MultitrackTypeAndFourCc::ManyTracksManyCodecs => {
                    let mut audio_four_cc = [0; 4];
                    reader.read_exact(&mut audio_four_cc)?;
                    AudioFourCc::from(audio_four_cc)
                }
                MultitrackTypeAndFourCc::OneTrack(audio_four_cc) => audio_four_cc,
                MultitrackTypeAndFourCc::ManyTracks(audio_four_cc) => audio_four_cc,
                MultitrackTypeAndFourCc::NoMultiTrack(audio_four_cc) => audio_four_cc,
                MultitrackTypeAndFourCc::Other { audio_four_cc, .. } => audio_four_cc,
            };

            let audio_track_id = reader.read_u8()?;

            let size_of_audio_track = if !has_one_track {
                Some(reader.read_u24::<BigEndian>()?)
            } else {
                None
            };

            match header.audio_packet_type {
                AudioPacketType::MultichannelConfig => {
                    let audio_channel_order = AudioChannelOrder::from(reader.read_u8()?);
                    let channel_count = reader.read_u8()?;

                    let multichannel_config = match audio_channel_order {
                        AudioChannelOrder::Custom => {
                            let channels = reader.extract_bytes(channel_count as usize)?;

                            MultichannelConfig::Custom(channels.into_iter().map(AudioChannel::from).collect())
                        }
                        AudioChannelOrder::Native => {
                            let audio_channel_flags = AudioChannelMask::from(reader.read_u32::<BigEndian>()?);

                            MultichannelConfig::Native(audio_channel_flags)
                        }
                        AudioChannelOrder::Unspecified => MultichannelConfig::Unspecified,
                        _ => MultichannelConfig::Unknown(audio_channel_order),
                    };

                    let packet = AudioPacket::MultichannelConfig {
                        channel_count,
                        multichannel_config,
                    };

                    tracks.push(AudioTrack {
                        audio_four_cc,
                        audio_track_id,
                        packet,
                    });
                }
                AudioPacketType::SeguenceEnd => {
                    tracks.push(AudioTrack {
                        audio_four_cc,
                        audio_track_id,
                        packet: AudioPacket::SequenceEnd,
                    });
                }
                AudioPacketType::SeguenceStart => {
                    let header_data =
                        reader.extract_bytes(size_of_audio_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

                    tracks.push(AudioTrack {
                        audio_four_cc,
                        audio_track_id,
                        packet: AudioPacket::SequenceStart { header_data },
                    });
                }
                AudioPacketType::CodedFrames => {
                    let data =
                        reader.extract_bytes(size_of_audio_track.map(|s| s as usize).unwrap_or(reader.remaining()))?;

                    tracks.push(AudioTrack {
                        audio_four_cc,
                        audio_track_id,
                        packet: AudioPacket::CodedFrames { data },
                    });
                }
                // skip all unhandled packet types
                _ => {}
            }

            if !has_one_track && reader.remaining() > 0 {
                continue;
            }

            break;
        }

        Ok(Self { tracks })
    }
}
