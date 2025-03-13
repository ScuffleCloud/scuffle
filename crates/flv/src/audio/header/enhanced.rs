use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use crate::common::AvMultitrackType;
use crate::error::Error;

nutype_enum! {
    pub enum AudioPacketType(u8) {
        SeguenceStart = 0,
        CodedFrames = 1,
        SeguenceEnd = 2,
        MultichannelConfig = 4,
        Multitrack = 5,
        ModEx = 7,
    }
}

nutype_enum! {
    pub enum AudioPacketModExType(u8) {
        TimestampOffsetNano = 0,
    }
}

/// This is a helper enum to represent the different types of audio packet modifier extensions.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioPacketModEx {
    TimestampOffsetNano {
        audio_timestamp_nano_offset: u32,
    },
    Other {
        audio_packet_mod_ex_type: AudioPacketModExType,
        mod_ex_data: Bytes,
    },
}

nutype_enum! {
    pub enum AudioFourCc([u8; 4]) {
        Ac3 = *b"ac-3",
        Eac3 = *b"ec-3",
        Opus = *b"Opus",
        Mp3 = *b".mp3",
        Flac = *b"fLaC",
        Aac = *b"mp4a",
    }
}

/// This is a helper enum to represent the different types of multitrack audio.
#[derive(Debug, Clone, PartialEq)]
pub enum ExAudioTagHeaderContent {
    NoMultiTrack(AudioFourCc),
    OneTrack(AudioFourCc),
    ManyTracks(AudioFourCc),
    ManyTracksManyCodecs,
    Other {
        audio_multitrack_type: AvMultitrackType,
        audio_four_cc: AudioFourCc,
    },
}

/// FLV `ExAudioTagHeader`
///
/// Defined by Enhanced RTMP v2 (Enhanced Audio section).
#[derive(Debug, Clone, PartialEq)]
pub struct ExAudioTagHeader {
    pub audio_packet_mod_exs: Vec<AudioPacketModEx>,
    pub audio_packet_type: AudioPacketType,
    pub content: ExAudioTagHeaderContent,
}

impl ExAudioTagHeader {
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let mut audio_packet_type = AudioPacketType::from(reader.read_u8()? & 0b0000_1111);

        let mut audio_packet_mod_exs = Vec::new();

        while audio_packet_type == AudioPacketType::ModEx {
            let mut mod_ex_data_size = reader.read_u8()? as usize + 1;
            if mod_ex_data_size == 256 {
                mod_ex_data_size = reader.read_u16::<BigEndian>()? as usize + 1;
            }

            let mod_ex_data = reader.extract_bytes(mod_ex_data_size)?;

            let next_byte = reader.read_u8()?;
            let audio_packet_mod_ex_type = AudioPacketModExType::from(next_byte >> 4); // 0b1111_0000
            audio_packet_type = AudioPacketType::from(next_byte & 0b0000_1111);

            if audio_packet_mod_ex_type == AudioPacketModExType::TimestampOffsetNano {
                if mod_ex_data_size < 3 {
                    // too few data bytes for the timestamp offset
                    return Err(Error::InvalidModExData { expected_bytes: 3 });
                }

                let mod_ex_data = &mut io::Cursor::new(mod_ex_data);
                audio_packet_mod_exs.push(AudioPacketModEx::TimestampOffsetNano {
                    audio_timestamp_nano_offset: mod_ex_data.read_u24::<BigEndian>()?,
                });
            } else {
                audio_packet_mod_exs.push(AudioPacketModEx::Other {
                    audio_packet_mod_ex_type,
                    mod_ex_data,
                });
            }
        }

        if audio_packet_type == AudioPacketType::Multitrack {
            let byte = reader.read_u8()?;
            let audio_multitrack_type = AvMultitrackType::from(byte >> 4); // 0b1111_0000
            audio_packet_type = AudioPacketType::from(byte & 0b0000_1111);

            if audio_packet_type == AudioPacketType::Multitrack {
                // nested multitracks are not allowed
                return Err(Error::NestedMultitracks);
            }

            let mut audio_four_cc = [0; 4];
            // Only read the four cc if it's not ManyTracksManyCodecs
            if audio_multitrack_type != AvMultitrackType::ManyTracksManyCodecs {
                reader.read_exact(&mut audio_four_cc)?;
            }

            let content = match audio_multitrack_type {
                AvMultitrackType::OneTrack => ExAudioTagHeaderContent::OneTrack(AudioFourCc::from(audio_four_cc)),
                AvMultitrackType::ManyTracks => ExAudioTagHeaderContent::ManyTracks(AudioFourCc::from(audio_four_cc)),
                AvMultitrackType::ManyTracksManyCodecs => ExAudioTagHeaderContent::ManyTracksManyCodecs,
                _ => ExAudioTagHeaderContent::Other {
                    audio_multitrack_type,
                    audio_four_cc: AudioFourCc::from(audio_four_cc),
                },
            };

            Ok(Self {
                audio_packet_mod_exs,
                audio_packet_type,
                content,
            })
        } else {
            let mut audio_four_cc = [0; 4];
            reader.read_exact(&mut audio_four_cc)?;
            let audio_four_cc = AudioFourCc::from(audio_four_cc);

            Ok(Self {
                audio_packet_mod_exs,
                audio_packet_type,
                content: ExAudioTagHeaderContent::NoMultiTrack(audio_four_cc),
            })
        }
    }
}
