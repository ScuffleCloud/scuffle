use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use crate::common::AvMultitrackType;
use crate::error::Error;

nutype_enum! {
    pub enum AudioPacketType(u8) {
        SequenceStart = 0,
        CodedFrames = 1,
        SequenceEnd = 2,
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

impl AudioPacketModEx {
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<(Self, AudioPacketType), Error> {
        let mut mod_ex_data_size = reader.read_u8()? as usize + 1;
        if mod_ex_data_size == 256 {
            mod_ex_data_size = reader.read_u16::<BigEndian>()? as usize + 1;
        }

        let mod_ex_data = reader.extract_bytes(mod_ex_data_size)?;

        let next_byte = reader.read_u8()?;
        let audio_packet_mod_ex_type = AudioPacketModExType::from(next_byte >> 4); // 0b1111_0000
        let audio_packet_type = AudioPacketType::from(next_byte & 0b0000_1111);

        if audio_packet_mod_ex_type == AudioPacketModExType::TimestampOffsetNano {
            if mod_ex_data_size < 3 {
                // too few data bytes for the timestamp offset
                return Err(Error::InvalidModExData { expected_bytes: 3 });
            }

            let mod_ex_data = &mut io::Cursor::new(mod_ex_data);

            Ok((
                Self::TimestampOffsetNano {
                    audio_timestamp_nano_offset: mod_ex_data.read_u24::<BigEndian>()?,
                },
                audio_packet_type,
            ))
        } else {
            tracing::trace!(audio_packet_mod_ex_type = ?audio_packet_mod_ex_type, "unknown audio packet modifier extension type");

            Ok((
                Self::Other {
                    audio_packet_mod_ex_type,
                    mod_ex_data,
                },
                audio_packet_type,
            ))
        }
    }
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
    Unknown {
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
            let (mod_ex, next_audio_packet_type) = AudioPacketModEx::demux(reader)?;
            audio_packet_mod_exs.push(mod_ex);
            audio_packet_type = next_audio_packet_type;
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
                _ => {
                    tracing::warn!(audio_multitrack_type = ?audio_multitrack_type, "unknown audio multitrack type");

                    ExAudioTagHeaderContent::Unknown {
                        audio_multitrack_type,
                        audio_four_cc: AudioFourCc::from(audio_four_cc),
                    }
                }
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

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;

    use super::AudioPacketModEx;
    use crate::audio::header::{
        AudioFourCc, AudioPacketModExType, AudioPacketType, ExAudioTagHeader, ExAudioTagHeaderContent,
    };
    use crate::common::AvMultitrackType;
    use crate::error::Error;

    #[test]
    fn small_mod_ex_demux() {
        let data = &[
            1,  // size 2
            42, // data
            42,
            0b0001_0001, // type 1, next packet 1
        ];

        let (mod_ex, next_packet) = AudioPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            mod_ex,
            AudioPacketModEx::Other {
                audio_packet_mod_ex_type: AudioPacketModExType(1),
                mod_ex_data: Bytes::from_static(&[42, 42])
            }
        );
        assert_eq!(next_packet, AudioPacketType::CodedFrames);
    }

    #[test]
    fn timestamp_offset_mod_ex_demux() {
        let data = &[
            2, // size 3
            0, // data
            0,
            1,
            0b0000_0000, // type 0, next packet 0
        ];

        let (mod_ex, next_packet) = AudioPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            mod_ex,
            AudioPacketModEx::TimestampOffsetNano {
                audio_timestamp_nano_offset: 1
            },
        );
        assert_eq!(next_packet, AudioPacketType::SequenceStart);
    }

    #[test]
    fn big_mod_ex_demux() {
        let data = &[
            255, // size 2
            0,
            1,
            42, // data
            42,
            0b0001_0001, // type 1, next packet 1
        ];

        let (mod_ex, next_packet) = AudioPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(
            mod_ex,
            AudioPacketModEx::Other {
                audio_packet_mod_ex_type: AudioPacketModExType(1),
                mod_ex_data: Bytes::from_static(&[42, 42])
            }
        );
        assert_eq!(next_packet, AudioPacketType::CodedFrames);
    }

    #[test]
    fn mod_ex_demux_error() {
        let data = &[
            0, // size 1
            42,
            0b0000_0010, // type 0, next packet 2
        ];

        let err = AudioPacketModEx::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap_err();

        assert!(matches!(err, Error::InvalidModExData { expected_bytes: 3 },));
    }

    #[test]
    fn minimal_header() {
        let data = &[
            0b0000_0000, // type 0
            b'm',        // four cc
            b'p',
            b'4',
            b'a',
        ];

        let header = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.audio_packet_mod_exs.len(), 0);
        assert_eq!(header.audio_packet_type, AudioPacketType::SequenceStart);
        assert_eq!(header.content, ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac));
    }

    #[test]
    fn header_small_mod_ex() {
        let data = &[
            0b0000_0111, // type 7
            1,           // modex size 2
            42,          // modex data
            42,
            0b0001_0001, // type 1, next packet 1
            b'm',        // four cc
            b'p',
            b'4',
            b'a',
        ];

        let header = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.audio_packet_mod_exs.len(), 1);
        assert_eq!(
            header.audio_packet_mod_exs[0],
            AudioPacketModEx::Other {
                audio_packet_mod_ex_type: AudioPacketModExType(1),
                mod_ex_data: Bytes::from_static(&[42, 42])
            }
        );
        assert_eq!(header.audio_packet_type, AudioPacketType::CodedFrames);
        assert_eq!(header.content, ExAudioTagHeaderContent::NoMultiTrack(AudioFourCc::Aac));
    }

    #[test]
    fn header_multitrack_one_track() {
        let data = &[
            0b0000_0101, // type 5
            0b0000_0000, // one track, type 0
            b'm',        // four cc
            b'p',
            b'4',
            b'a',
        ];

        let header = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.audio_packet_mod_exs.len(), 0);
        assert_eq!(header.audio_packet_type, AudioPacketType::SequenceStart);
        assert_eq!(header.content, ExAudioTagHeaderContent::OneTrack(AudioFourCc::Aac));
    }

    #[test]
    fn header_multitrack_many_tracks() {
        let data = &[
            0b0000_0101, // type 5
            0b0001_0000, // many tracks, type 0
            b'm',        // four cc
            b'p',
            b'4',
            b'a',
        ];

        let header = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.audio_packet_mod_exs.len(), 0);
        assert_eq!(header.audio_packet_type, AudioPacketType::SequenceStart);
        assert_eq!(header.content, ExAudioTagHeaderContent::ManyTracks(AudioFourCc::Aac));
    }

    #[test]
    fn header_multitrack_many_tracks_many_codecs() {
        let data = &[
            0b0000_0101, // type 5
            0b0010_0000, // many tracks many codecs, type 0
            b'm',        // four cc, should be ignored
            b'p',
            b'4',
            b'a',
        ];

        let header = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.audio_packet_mod_exs.len(), 0);
        assert_eq!(header.audio_packet_type, AudioPacketType::SequenceStart);
        assert_eq!(header.content, ExAudioTagHeaderContent::ManyTracksManyCodecs);
    }

    #[test]
    fn header_multitrack_unknown() {
        let data = &[
            0b0000_0101, // type 5
            0b0011_0000, // unknown, type 0
            b'm',        // four cc
            b'p',
            b'4',
            b'a',
        ];

        let header = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap();

        assert_eq!(header.audio_packet_mod_exs.len(), 0);
        assert_eq!(header.audio_packet_type, AudioPacketType::SequenceStart);
        assert_eq!(
            header.content,
            ExAudioTagHeaderContent::Unknown {
                audio_multitrack_type: AvMultitrackType(3),
                audio_four_cc: AudioFourCc::Aac
            }
        );
    }

    #[test]
    fn nested_multitrack_error() {
        let data = &[
            0b0000_0101, // type 5
            0b0000_0101, // one track, type 5
        ];

        let err = ExAudioTagHeader::demux(&mut std::io::Cursor::new(Bytes::from_static(data))).unwrap_err();
        assert!(matches!(err, Error::NestedMultitracks));
    }
}
