//! Enhanced audio tag body
//!
//! Types and functions defined by the enhanced RTMP spec, page 19, ExAudioTagBody.

use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::header::enhanced::{AudioFourCc, AudioPacketType, ExAudioTagHeader, ExAudioTagHeaderContent};

nutype_enum! {
    /// Audio channel order
    ///
    /// Defined by:
    /// - Enhanced RTMP spec, page 22-23, ExAudioTagBody
    pub enum AudioChannelOrder(u8) {
        /// Only the channel count is specified, without any further information about the channel order.
        Unspecified = 0,
        /// The native channel order (i.e., the channels are in the same order in which as defined in the [`AudioChannel`] enum).
        Native = 1,
        /// The channel order does not correspond to any predefined order and is stored as an explicit map.
        Custom = 2,
    }
}

nutype_enum! {
    /// Channel mappings enum
    ///
    /// See <https://en.wikipedia.org/wiki/Surround_sound#Standard_speaker_channels> and
    /// <https://en.wikipedia.org/wiki/22.2_surround_sound> for more information.
    pub enum AudioChannel(u8) {
        // Commonly used speaker configurations:

        /// Front left
        FrontLeft = 0,
        /// Front right
        FrontRight = 1,
        /// Front center
        FrontCenter = 2,
        /// Low frequency
        LowFrequency1 = 3,
        /// Back left
        BackLeft = 4,
        /// Back right
        BackRight = 5,
        /// Front left of center
        FrontLeftCenter = 6,
        /// Front right of center
        FrontRightCenter = 7,
        /// Back center
        BackCenter = 8,
        /// Side left
        SideLeft = 9,
        /// Side right
        SideRight = 10,
        /// Top center
        TopCenter = 11,
        /// Front left height
        TopFrontLeft = 12,
        /// Front center height
        TopFrontCenter = 13,
        /// Front right height
        TopFrontRight = 14,
        /// Rear left height
        TopBackLeft = 15,
        /// Rear center height
        TopBackCenter = 16,
        /// Rear right height
        TopBackRight = 17,

        // Mappings to complete 22.2 multichannel audio, as standardized in SMPTE ST2036-2-2008:

        /// Low frequency 2
        LowFrequency2 = 18,
        /// Top side left
        TopSideLeft = 19,
        /// Top side right
        TopSideRight = 20,
        /// Bottom front center
        BottomFrontCenter = 21,
        /// Bottom front left
        BottomFrontLeft = 22,
        /// Bottom front right
        BottomFrontRight = 23,
        /// Channel is empty and can be safely skipped.
        Unused = 0xfe,
        /// Channel contains data, but its speaker configuration is unknown.
        Unknown = 0xff,
    }
}

/// Mask used to indicate which channels are present in the stream.
///
/// See <https://en.wikipedia.org/wiki/Surround_sound#Standard_speaker_channels> and
/// <https://en.wikipedia.org/wiki/22.2_surround_sound> for more information.
#[bitmask_enum::bitmask(u32)]
pub enum AudioChannelMask {
    // Masks for commonly used speaker configurations:
    /// Front left
    FrontLeft = 0x000001,
    /// Front right
    FrontRight = 0x000002,
    /// Front center
    FrontCenter = 0x000004,
    /// Low frequency
    LowFrequency1 = 0x000008,
    /// Back left
    BackLeft = 0x000010,
    /// Back right
    BackRight = 0x000020,
    /// Front left of center
    FrontLeftCenter = 0x000040,
    /// Front right of center
    FrontRightCenter = 0x000080,
    /// Back center
    BackCenter = 0x000100,
    /// Side left
    SideLeft = 0x000200,
    /// Side right
    SideRight = 0x000400,
    /// Top center
    TopCenter = 0x000800,
    /// Front left height
    TopFrontLeft = 0x001000,
    /// Front center height
    TopFrontCenter = 0x002000,
    /// Front right height
    TopFrontRight = 0x004000,
    /// Rear left height
    TopBackLeft = 0x008000,
    /// Rear center height
    TopBackCenter = 0x010000,
    /// Rear right height
    TopBackRight = 0x020000,

    // Completes 22.2 multichannel audio, as
    // standardized in SMPTE ST2036-2-2008:
    /// Low frequency 2
    LowFrequency2 = 0x040000,
    /// Top side left
    TopSideLeft = 0x080000,
    /// Top side right
    TopSideRight = 0x100000,
    /// Bottom front center
    BottomFrontCenter = 0x200000,
    /// Bottom front left
    BottomFrontLeft = 0x400000,
    /// Bottom front right
    BottomFrontRight = 0x800000,
}

/// Multichannel configuration
///
/// Describes the configuration of the audio channels in a multichannel audio stream.
///
/// Contained in an [`AudioPacket::MultichannelConfig`].
#[derive(Debug, Clone, PartialEq)]
pub enum MultichannelConfigOrder {
    /// Custom channel order
    ///
    /// The channels have a custom order that is explicitly defined by this packet.
    Custom(Vec<AudioChannel>),
    /// Native channel order
    ///
    /// Only the channels flagged in this packet are present in the stream
    /// in the order they are defined by the [`AudioChannelMask`].
    ///
    /// > You can perform a Bitwise AND
    /// > (i.e., audioChannelFlags & AudioChannelMask.xxx) to see if a
    /// > specific audio channel is present.
    Native(AudioChannelMask),
    /// The channel order is unspecified, only the channel count is known.
    Unspecified,
    /// An unknown channel order.
    ///
    /// Neither [`Unspecified`](AudioChannelOrder::Unspecified), [`Native`](AudioChannelOrder::Native),
    /// nor [`Custom`](AudioChannelOrder::Custom).
    Unknown(AudioChannelOrder),
}

/// Audio packet
///
/// Appears as part of the [`ExAudioTagBody`].
///
/// Defined by:
/// - Enhanced RTMP spec, page 23-25, ExAudioTagBody
#[derive(Debug, Clone, PartialEq)]
pub enum AudioPacket {
    /// Multichannel configuration
    ///
    /// > Specify a speaker for a channel as it appears in the bitstream.
    /// > This is needed if the codec is not self-describing for channel mapping.
    MultichannelConfig {
        /// The number of channels in the audio stream.
        channel_count: u8,
        /// The multichannel configuration.
        ///
        /// Specifies the order of the channels in the audio stream.
        multichannel_config: MultichannelConfigOrder,
    },
    /// Indicates the end of a sequence of audio packets.
    SequenceEnd,
    /// Indicates the start of a sequence of audio packets.
    SequenceStart {
        /// The header data for the sequence.
        header_data: Bytes,
    },
    /// Coded audio frames.
    CodedFrames {
        /// The audio data.
        data: Bytes,
    },
    /// An unknown [`AudioPacketType`].
    Unknown {
        /// The unknown packet type.
        audio_packet_type: AudioPacketType,
        /// The data.
        data: Bytes,
    },
}

impl AudioPacket {
    /// Demux an [`AudioPacket`] from the given reader.
    ///
    /// This is implemented as per spec, Enhanced RTMP page 23-25, ExAudioTagBody.
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

                        MultichannelConfigOrder::Custom(channels.into_iter().map(AudioChannel::from).collect())
                    }
                    AudioChannelOrder::Native => {
                        let audio_channel_flags = AudioChannelMask::from(reader.read_u32::<BigEndian>()?);

                        MultichannelConfigOrder::Native(audio_channel_flags)
                    }
                    AudioChannelOrder::Unspecified => MultichannelConfigOrder::Unspecified,
                    _ => MultichannelConfigOrder::Unknown(audio_channel_order),
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
                let data = reader.extract_bytes(size_of_audio_track.unwrap_or(reader.remaining()))?;

                Ok(Self::Unknown {
                    audio_packet_type: header.audio_packet_type,
                    data,
                })
            }
        }
    }
}

/// One audio track contained in a multitrack audio.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioTrack {
    /// The audio FOURCC of this track.
    pub audio_four_cc: AudioFourCc,
    /// The audio track ID.
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
    pub audio_track_id: u8,
    /// The audio packet contained in this track.
    pub packet: AudioPacket,
}

/// `ExAudioTagBody`
///
/// Defined by:
/// - Enhanced RTMP spec, page 22-25, ExAudioTagBody
#[derive(Debug, Clone, PartialEq)]
pub enum ExAudioTagBody {
    /// The body is not a multitrack body.
    NoMultitrack {
        /// The audio FOURCC of this body.
        audio_four_cc: AudioFourCc,
        /// The audio packet contained in this body.
        packet: AudioPacket,
    },
    /// The body is a multitrack body.
    ///
    /// This variant contains multiple audio tracks.
    /// See [`AudioTrack`] for more information.
    ManyTracks(Vec<AudioTrack>),
}

impl ExAudioTagBody {
    /// Demux an [`ExAudioTagBody`] from the given reader.
    ///
    /// This is implemented as per Enhanced RTMP spec, page 22-25, ExAudioTagBody.
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
    use crate::audio::body::enhanced::{
        AudioChannel, AudioChannelMask, AudioChannelOrder, AudioTrack, ExAudioTagBody, MultichannelConfigOrder,
    };
    use crate::audio::header::enhanced::{AudioFourCc, AudioPacketType, ExAudioTagHeader, ExAudioTagHeaderContent};
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
                audio_packet_type: AudioPacketType(8),
                data: Bytes::from_static(data),
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
                multichannel_config: MultichannelConfigOrder::Custom(vec![
                    AudioChannel::FrontLeft,
                    AudioChannel::FrontRight
                ])
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
                multichannel_config: MultichannelConfigOrder::Native(
                    AudioChannelMask::FrontLeft | AudioChannelMask::FrontRight
                )
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
                multichannel_config: MultichannelConfigOrder::Unspecified,
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
                multichannel_config: MultichannelConfigOrder::Unknown(AudioChannelOrder(4)),
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
                    data: Bytes::from_static(data)
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
