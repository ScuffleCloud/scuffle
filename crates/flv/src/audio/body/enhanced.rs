use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::header::{AudioFourCc, AudioPacketType, ExAudioTagHeader, ExAudioTagHeaderContent};

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
    Unknown {
        data: Bytes,
    },
}

impl AudioPacket {
    pub fn demux(header: &ExAudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let has_multiple_tracks = !matches!(
            header.content,
            ExAudioTagHeaderContent::NoMultiTrack(_) | ExAudioTagHeaderContent::OneTrack(_)
        );

        let size_of_audio_track = if has_multiple_tracks {
            Some(reader.read_u24::<BigEndian>()? as usize)
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

                Ok(Self::MultichannelConfig {
                    channel_count,
                    multichannel_config,
                })
            }
            AudioPacketType::SequenceEnd => Ok(Self::SequenceEnd),
            AudioPacketType::SequenceStart => {
                let header_data = reader.extract_bytes(size_of_audio_track.unwrap_or(reader.remaining()))?;

                Ok(Self::SequenceStart { header_data })
            }
            AudioPacketType::CodedFrames => {
                let data = reader.extract_bytes(size_of_audio_track.unwrap_or(reader.remaining()))?;

                Ok(Self::CodedFrames { data })
            }
            _ => {
                tracing::warn!(audio_packet_type = ?header.audio_packet_type, "unknown audio packet type");

                let data = reader.extract_bytes(size_of_audio_track.unwrap_or(reader.remaining()))?;

                Ok(Self::Unknown { data })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTrack {
    pub audio_four_cc: AudioFourCc,
    pub audio_track_id: u8,
    pub packet: AudioPacket,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExAudioTagBody {
    NoMultitrack {
        audio_four_cc: AudioFourCc,
        packet: AudioPacket,
    },
    ManyTracks(Vec<AudioTrack>),
}

impl ExAudioTagBody {
    pub fn demux(header: &ExAudioTagHeader, reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        let mut tracks = Vec::new();

        loop {
            let audio_four_cc = match header.content {
                ExAudioTagHeaderContent::ManyTracksManyCodecs => {
                    let mut audio_four_cc = [0; 4];
                    reader.read_exact(&mut audio_four_cc)?;
                    AudioFourCc::from(audio_four_cc)
                }
                ExAudioTagHeaderContent::OneTrack(audio_four_cc) => audio_four_cc,
                ExAudioTagHeaderContent::ManyTracks(audio_four_cc) => audio_four_cc,
                ExAudioTagHeaderContent::NoMultiTrack(audio_four_cc) => audio_four_cc,
                ExAudioTagHeaderContent::Unknown { audio_four_cc, .. } => audio_four_cc,
            };

            // if isAudioMultitrack
            let audio_track_id = if !matches!(header.content, ExAudioTagHeaderContent::NoMultiTrack(_)) {
                Some(reader.read_u8()?)
            } else {
                None
            };

            let packet = AudioPacket::demux(header, reader)?;

            if let Some(audio_track_id) = audio_track_id {
                // audio_track_id is only set if this is a multitrack audio, in other words, if `isAudioMultitrack` is true
                tracks.push(AudioTrack {
                    audio_four_cc,
                    audio_track_id,
                    packet,
                });

                // the loop only continues if there is still data to read and this is a audio with multiple tracks
                if !matches!(header.content, ExAudioTagHeaderContent::OneTrack(_)) && reader.has_remaining() {
                    continue;
                }

                break;
            } else {
                // exit early if this is a single track audio only completing one loop iteration
                return Ok(Self::NoMultitrack { audio_four_cc, packet });
            }
        }

        // at this point we know this is a multitrack audio because a single track audio would have exited early
        Ok(Self::ManyTracks(tracks))
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;

    use super::AudioPacket;
    use crate::audio::body::{
        AudioChannel, AudioChannelMask, AudioChannelOrder, AudioTrack, ExAudioTagBody, MultichannelConfig,
    };
    use crate::audio::header::{AudioFourCc, AudioPacketType, ExAudioTagHeader, ExAudioTagHeaderContent};
    use crate::common::AvMultitrackType;

    #[test]
    fn simple_audio_packets_demux() {
        let data = &[42, 42, 42, 42];

        let packet = AudioPacket::demux(
            &ExAudioTagHeader {
                audio_packet_mod_exs: vec![],
                audio_packet_type: AudioPacketType::SequenceStart,
                content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
            },
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(
            packet,
            AudioPacket::SequenceStart {
                header_data: Bytes::from_static(data)
            }
        );

        let packet = AudioPacket::demux(
            &ExAudioTagHeader {
                audio_packet_mod_exs: vec![],
                audio_packet_type: AudioPacketType::CodedFrames,
                content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
            },
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(
            packet,
            AudioPacket::CodedFrames {
                data: Bytes::from_static(data)
            }
        );

        let packet = AudioPacket::demux(
            &ExAudioTagHeader {
                audio_packet_mod_exs: vec![],
                audio_packet_type: AudioPacketType::SequenceEnd,
                content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
            },
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(packet, AudioPacket::SequenceEnd,);

        let packet = AudioPacket::demux(
            &ExAudioTagHeader {
                audio_packet_mod_exs: vec![],
                audio_packet_type: AudioPacketType(8),
                content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
            },
            &mut std::io::Cursor::new(Bytes::from_static(data)),
        )
        .unwrap();
        assert_eq!(
            packet,
            AudioPacket::Unknown {
                data: Bytes::from_static(data)
            },
        );
    }

    #[test]
    fn audio_packet_with_size_demux() {
        let data = &[
            0, 0, 2, // size
            42, 42, // data
            13, 37, // should be ignored
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::CodedFrames,
            content: ExAudioTagHeaderContent::ManyTracks(AudioFourCc::Aac),
        };

        let packet = AudioPacket::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            AudioPacket::CodedFrames {
                data: Bytes::from_static(&[42, 42])
            },
        );
    }

    #[test]
    fn audio_packet_custom_multichannel_config_demux() {
        let data = &[
            2, // channel order custom
            2, // channel count
            0, 1, // channels
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::MultichannelConfig,
            content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
        };

        let packet = AudioPacket::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            AudioPacket::MultichannelConfig {
                channel_count: 2,
                multichannel_config: MultichannelConfig::Custom(vec![AudioChannel::FrontLeft, AudioChannel::FrontRight])
            },
        );
    }

    #[test]
    fn audio_packet_native_multichannel_config_demux() {
        let data = &[
            1, // channel order native
            2, // channel count
            0, 0, 0, 3, // channels
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::MultichannelConfig,
            content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
        };

        let packet = AudioPacket::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            AudioPacket::MultichannelConfig {
                channel_count: 2,
                multichannel_config: MultichannelConfig::Native(AudioChannelMask::FrontLeft | AudioChannelMask::FrontRight)
            },
        );
    }

    #[test]
    fn audio_packet_other_multichannel_config_demux() {
        let data = &[
            0, // channel order unspecified
            2, // channel count
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::MultichannelConfig,
            content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
        };

        let packet = AudioPacket::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            AudioPacket::MultichannelConfig {
                channel_count: 2,
                multichannel_config: MultichannelConfig::Unspecified,
            },
        );

        let data = &[
            4, // channel order unknown
            2, // channel count
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::MultichannelConfig,
            content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
        };

        let packet = AudioPacket::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            AudioPacket::MultichannelConfig {
                channel_count: 2,
                multichannel_config: MultichannelConfig::Unknown(AudioChannelOrder(4)),
            },
        );
    }

    #[test]
    fn simple_body_demux() {
        let data = &[
            42, 42, // data
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::CodedFrames,
            content: ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac),
        };

        let packet = ExAudioTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExAudioTagBody::NoMultitrack {
                audio_four_cc: AudioFourCc::Aac,
                packet: AudioPacket::CodedFrames {
                    data: Bytes::from_static(&[42, 42])
                },
            },
        );
    }

    #[test]
    fn multitrack_many_codecs_body_demux() {
        let data = &[
            b'm', b'p', b'4', b'a', // audio four cc
            1,    // audio track id
            0, 0, 2, // size
            42, 42, // data
            b'O', b'p', b'u', b's', // audio four cc
            2,    // audio track id
            0, 0, 2, // size
            13, 37, // data
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::CodedFrames,
            content: ExAudioTagHeaderContent::ManyTracksManyCodecs,
        };

        let packet = ExAudioTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExAudioTagBody::ManyTracks(vec![
                AudioTrack {
                    audio_four_cc: AudioFourCc::Aac,
                    audio_track_id: 1,
                    packet: AudioPacket::CodedFrames {
                        data: Bytes::from_static(&[42, 42])
                    },
                },
                AudioTrack {
                    audio_four_cc: AudioFourCc::Opus,
                    audio_track_id: 2,
                    packet: AudioPacket::CodedFrames {
                        data: Bytes::from_static(&[13, 37])
                    },
                }
            ]),
        );
    }

    #[test]
    fn multitrack_body_demux() {
        let data = &[
            1, // audio track id
            0, 0, 2, // size
            42, 42, // data
            2,  // audio track id
            0, 0, 2, // size
            13, 37, // data
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::CodedFrames,
            content: ExAudioTagHeaderContent::ManyTracks(AudioFourCc::Aac),
        };

        let packet = ExAudioTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExAudioTagBody::ManyTracks(vec![
                AudioTrack {
                    audio_four_cc: AudioFourCc::Aac,
                    audio_track_id: 1,
                    packet: AudioPacket::CodedFrames {
                        data: Bytes::from_static(&[42, 42])
                    },
                },
                AudioTrack {
                    audio_four_cc: AudioFourCc::Aac,
                    audio_track_id: 2,
                    packet: AudioPacket::CodedFrames {
                        data: Bytes::from_static(&[13, 37])
                    },
                }
            ]),
        );
    }

    #[test]
    fn multitrack_one_track_body_demux() {
        let data = &[
            1, // audio track id
            42, 42, // data
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::CodedFrames,
            content: ExAudioTagHeaderContent::OneTrack(AudioFourCc::Aac),
        };

        let packet = ExAudioTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExAudioTagBody::ManyTracks(vec![AudioTrack {
                audio_four_cc: AudioFourCc::Aac,
                audio_track_id: 1,
                packet: AudioPacket::CodedFrames {
                    data: Bytes::from_static(&[42, 42])
                },
            }]),
        );
    }

    #[test]
    fn multitrack_unknown_body_demux() {
        let data = &[
            1, // audio track id
            0, 0, 2, // size
            42, 42, // data
        ];

        let header = ExAudioTagHeader {
            audio_packet_mod_exs: vec![],
            audio_packet_type: AudioPacketType::CodedFrames,
            content: ExAudioTagHeaderContent::Unknown {
                audio_four_cc: AudioFourCc::Aac,
                audio_multitrack_type: AvMultitrackType(4),
            },
        };

        let packet = ExAudioTagBody::demux(&header, &mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            packet,
            ExAudioTagBody::ManyTracks(vec![AudioTrack {
                audio_track_id: 1,
                audio_four_cc: AudioFourCc::Aac,
                packet: AudioPacket::CodedFrames {
                    data: Bytes::from_static(&[42, 42])
                }
            }]),
        );
    }
}
