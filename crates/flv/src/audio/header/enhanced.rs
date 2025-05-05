//! Enhanced audio header types and functions.

use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use nutype_enum::nutype_enum;
use scuffle_bytes_util::BytesCursorExt;

use crate::common::AvMultitrackType;
use crate::error::FlvError;

nutype_enum! {
    /// Different types of audio packets.
    ///
    /// Defined by:
    /// - Enhanced RTMP spec, page 20-21, Enhanced Audio
    pub enum AudioPacketType(u8) {
        /// Sequence start.
        SequenceStart = 0,
        /// Coded frames.
        CodedFrames = 1,
        /// Sequence end.
        SequenceEnd = 2,
        /// Multichannel configuration.
        MultichannelConfig = 4,
        /// Turns on audio multitrack mode.
        Multitrack = 5,
        /// Modifier extension.
        ModEx = 7,
    }
}

nutype_enum! {
    /// Different types of audio packet modifier extensions.
    pub enum AudioPacketModExType(u8) {
        /// Timestamp offset in nanoseconds.
        TimestampOffsetNano = 0,
    }
}

/// This is a helper enum to represent the different types of audio packet modifier extensions.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioPacketModEx {
    /// Timestamp offset in nanoseconds.
    TimestampOffsetNano {
        /// The timestamp offset in nanoseconds.
        audio_timestamp_nano_offset: u32,
    },
    /// Any other modifier extension.
    Other {
        /// The type of the modifier extension.
        audio_packet_mod_ex_type: AudioPacketModExType,
        /// The data of the modifier extension.
        mod_ex_data: Bytes,
    },
}

impl AudioPacketModEx {
    /// Demux a [`AudioPacketModEx`] from the given reader.
    ///
    /// Returns the demuxed [`AudioPacketModEx`] and the next [`AudioPacketType`], if successful.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<(Self, AudioPacketType), FlvError> {
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
                return Err(FlvError::InvalidModExData { expected_bytes: 3 });
            }

            let mod_ex_data = &mut io::Cursor::new(mod_ex_data);

            Ok((
                Self::TimestampOffsetNano {
                    audio_timestamp_nano_offset: mod_ex_data.read_u24::<BigEndian>()?,
                },
                audio_packet_type,
            ))
        } else {
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
    /// Valid FOURCC values for signaling support of audio codecs in the enhanced FourCC pipeline.
    ///
    /// Defined by:
    /// - Enhanced RTMP spec, page 21-22, Enhanced Audio
    pub enum AudioFourCc([u8; 4]) {
        /// Dolby AC-3
        ///
        /// <https://en.wikipedia.org/wiki/Dolby_Digital>
        Ac3 = *b"ac-3",
        /// Dolby Digital Plus (E-AC-3)
        ///
        /// <https://en.wikipedia.org/wiki/Dolby_Digital>
        Eac3 = *b"ec-3",
        /// Opus audio
        ///
        /// <https://opus-codec.org/>
        Opus = *b"Opus",
        /// Mp3 audio
        ///
        /// <https://en.wikipedia.org/wiki/MP3>
        Mp3 = *b".mp3",
        /// Free Lossless Audio Codec
        ///
        /// <https://xiph.org/flac/format.html>
        Flac = *b"fLaC",
        /// Advanced Audio Coding
        ///
        /// <https://en.wikipedia.org/wiki/Advanced_Audio_Coding>
        Aac = *b"mp4a",
    }
}

/// This is a helper enum to represent the different types of multitrack audio.
#[derive(Debug, Clone, PartialEq)]
pub enum ExAudioTagHeaderContent {
    /// Not multitrack.
    NoMultiTrack(AudioFourCc),
    /// Multirack with one track.
    OneTrack(AudioFourCc),
    /// Multitrack with many tracks of the same codec.
    ManyTracks(AudioFourCc),
    /// Multitrack with many tracks of different codecs.
    ManyTracksManyCodecs,
    /// Unknown multitrack type.
    Unknown {
        /// The type of the multitrack audio.
        audio_multitrack_type: AvMultitrackType,
        /// The FOURCC of the audio codec.
        audio_four_cc: AudioFourCc,
    },
}

/// `ExAudioTagHeader`
///
/// Defined by:
/// - Enhanced RTMP spec, page 20-22, Enhanced Audio
#[derive(Debug, Clone, PartialEq)]
pub struct ExAudioTagHeader {
    /// The modifier extensions of the audio packet.
    ///
    /// This can be empty if there are no modifier extensions.
    pub audio_packet_mod_exs: Vec<AudioPacketModEx>,
    /// The type of the audio packet.
    pub audio_packet_type: AudioPacketType,
    /// The content of the audio packet which contains more information about the multitrack configuration.
    pub content: ExAudioTagHeaderContent,
}

impl ExAudioTagHeader {
    /// Demux an [`ExAudioTagHeader`] from the given reader.
    ///
    /// This is implemented as per Enhanced RTMP spec, page 20-21, ExAudioTagHeader.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
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
                return Err(FlvError::NestedMultitracks);
            }

            let mut audio_four_cc = [0; 4];
            // Only read the FOURCC if it's not ManyTracksManyCodecs
            if audio_multitrack_type != AvMultitrackType::ManyTracksManyCodecs {
                reader.read_exact(&mut audio_four_cc)?;
            }

            let content = match audio_multitrack_type {
                AvMultitrackType::OneTrack => ExAudioTagHeaderContent::OneTrack(AudioFourCc::from(audio_four_cc)),
                AvMultitrackType::ManyTracks => ExAudioTagHeaderContent::ManyTracks(AudioFourCc::from(audio_four_cc)),
                AvMultitrackType::ManyTracksManyCodecs => ExAudioTagHeaderContent::ManyTracksManyCodecs,
                _ => ExAudioTagHeaderContent::Unknown {
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

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;

    use super::AudioPacketModEx;
    use crate::audio::header::enhanced::{
        AudioFourCc, AudioPacketModExType, AudioPacketType, ExAudioTagHeader, ExAudioTagHeaderContent,
    };
    use crate::common::AvMultitrackType;
    use crate::error::FlvError;

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

        assert!(matches!(err, FlvError::InvalidModExData { expected_bytes: 3 },));
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
        assert!(matches!(err, FlvError::NestedMultitracks));
    }
}
