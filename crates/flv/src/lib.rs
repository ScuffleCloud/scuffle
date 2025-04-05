//! A pure Rust implementation of the FLV format, allowing for demuxing of FLV
//! files and streams.
//!
//! ## Specifications
//!
//! | Name | Version | Link | Comments |
//! | --- | --- | --- | --- |
//! | Video File Format Specification | `10` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/video-file-format-v10-0-spec.pdf> | |
//! | Adobe Flash Video File Format Specification | `10.1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/video-file-format-v10-1-spec.pdf> | Refered to as 'Legacy FLV spec' in this documentation |
//! | Enhancing RTMP, FLV | `v1-2024-02-29-r1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v1.pdf> | |
//! | Enhanced RTMP | `v2-2024-10-22-b1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v2.pdf> | Refered to as 'Enhanced RTMP spec' in this documentation |
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

pub mod audio;
pub mod common;
pub mod error;
pub mod file;
pub mod header;
pub mod script;
pub mod tag;
pub mod video;

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use bytes::Bytes;
    use scuffle_aac::{AudioObjectType, PartialAudioSpecificConfig};
    use scuffle_amf0::Amf0Value;
    use scuffle_av1::ObuHeader;
    use scuffle_av1::seq::SequenceHeaderObu;
    use scuffle_bytes_util::StringCow;
    use scuffle_h264::Sps;

    use crate::audio::AudioData;
    use crate::audio::body::AudioTagBody;
    use crate::audio::body::legacy::LegacyAudioTagBody;
    use crate::audio::body::legacy::aac::AacAudioData;
    use crate::audio::header::AudioTagHeader;
    use crate::audio::header::legacy::{LegacyAudioTagHeader, SoundFormat, SoundRate, SoundSize, SoundType};
    use crate::file::FlvFile;
    use crate::script::{OnMetaDataAudioCodecId, OnMetaDataVideoCodecId, ScriptData};
    use crate::tag::FlvTagData;
    use crate::video::VideoData;
    use crate::video::body::VideoTagBody;
    use crate::video::body::enhanced::{ExVideoTagBody, VideoPacket, VideoPacketSequenceStart};
    use crate::video::body::legacy::LegacyVideoTagBody;
    use crate::video::header::enhanced::VideoFourCc;
    use crate::video::header::legacy::{LegacyVideoTagHeader, LegacyVideoTagHeaderAvcPacket, VideoCodecId};
    use crate::video::header::{VideoFrameType, VideoTagHeader, VideoTagHeaderData};

    #[test]
    fn test_demux_flv_avc_aac() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");

        let data = Bytes::from(std::fs::read(dir.join("avc_aac.flv")).expect("failed to read file"));
        let mut reader = io::Cursor::new(data);

        let flv = FlvFile::demux(&mut reader).expect("failed to demux flv");

        assert_eq!(flv.header.version, 1);
        assert!(flv.header.is_audio_present);
        assert!(flv.header.is_video_present);
        assert_eq!(flv.header.extra.len(), 0);

        let mut tags = flv.tags.into_iter();

        // Metadata tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            // This is a metadata tag
            let on_meta_data = match tag.data {
                FlvTagData::ScriptData(ScriptData::OnMetaData(data)) => data,
                _ => panic!("expected script data"),
            };

            assert_eq!(on_meta_data.audiosamplesize, Some(16.0));
            assert_eq!(on_meta_data.audiosamplerate, Some(48000.0));
            assert_eq!(on_meta_data.stereo, Some(true));
            assert_eq!(
                on_meta_data.audiocodecid,
                Some(OnMetaDataAudioCodecId::Legacy(SoundFormat::Aac))
            ); // AAC
            assert_eq!(
                on_meta_data.videocodecid,
                Some(OnMetaDataVideoCodecId::Legacy(VideoCodecId::Avc))
            ); // AVC
            assert_eq!(on_meta_data.duration, Some(1.088)); // 1.088 seconds
            assert_eq!(on_meta_data.width, Some(3840.0));
            assert_eq!(on_meta_data.height, Some(2160.0));
            assert_eq!(on_meta_data.framerate, Some(60.0));
            assert!(on_meta_data.videodatarate.is_some());
            assert!(on_meta_data.audiodatarate.is_some());

            // Should have a minor version property
            let minor_version = match on_meta_data.other.get(&StringCow::from_static("minor_version")) {
                Some(Amf0Value::String(number)) => number,
                _ => panic!("expected minor version"),
            };

            assert_eq!(minor_version, "512");

            // Should have a major brand property
            let major_brand = match on_meta_data.other.get(&StringCow::from_static("major_brand")) {
                Some(Amf0Value::String(string)) => string,
                _ => panic!("expected major brand"),
            };

            assert_eq!(major_brand, "iso5");

            // Should have a compatible_brands property
            let compatible_brands = match on_meta_data.other.get(&StringCow::from_static("compatible_brands")) {
                Some(Amf0Value::String(string)) => string,
                _ => panic!("expected compatible brands"),
            };

            assert_eq!(compatible_brands, "iso5iso6mp41");
        }

        // Video Sequence Header Tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            // This is a video tag
            let (frame_type, avc_decoder_configuration_record) = match tag.data {
                FlvTagData::Video(VideoData {
                    header: VideoTagHeader { frame_type, .. },
                    body: VideoTagBody::Legacy(LegacyVideoTagBody::AvcVideoPacketSeqHdr(avc_decoder_configuration_record)),
                }) => (frame_type, avc_decoder_configuration_record),
                _ => panic!("expected video data"),
            };

            assert_eq!(frame_type, VideoFrameType::KeyFrame);

            // The avc sequence header should be able to be decoded into an avc decoder
            // configuration record
            assert_eq!(avc_decoder_configuration_record.profile_indication, 100);
            assert_eq!(avc_decoder_configuration_record.profile_compatibility, 0);
            assert_eq!(avc_decoder_configuration_record.level_indication, 51); // 5.1
            assert_eq!(avc_decoder_configuration_record.length_size_minus_one, 3);
            assert_eq!(avc_decoder_configuration_record.sps.len(), 1);
            assert_eq!(avc_decoder_configuration_record.pps.len(), 1);
            assert_eq!(avc_decoder_configuration_record.extended_config, None);

            let sps =
                Sps::parse_with_emulation_prevention(&mut std::io::Cursor::new(&avc_decoder_configuration_record.sps[0]))
                    .expect("expected sequence parameter set");

            insta::assert_debug_snapshot!(sps, @r"
            Sps {
                nal_ref_idc: 3,
                nal_unit_type: NALUnitType::SPS,
                profile_idc: 100,
                constraint_set0_flag: false,
                constraint_set1_flag: false,
                constraint_set2_flag: false,
                constraint_set3_flag: false,
                constraint_set4_flag: false,
                constraint_set5_flag: false,
                level_idc: 51,
                seq_parameter_set_id: 0,
                ext: Some(
                    SpsExtended {
                        chroma_format_idc: 1,
                        separate_color_plane_flag: false,
                        bit_depth_luma_minus8: 0,
                        bit_depth_chroma_minus8: 0,
                        qpprime_y_zero_transform_bypass_flag: false,
                        scaling_matrix: [],
                    },
                ),
                log2_max_frame_num_minus4: 0,
                pic_order_cnt_type: 0,
                log2_max_pic_order_cnt_lsb_minus4: Some(
                    4,
                ),
                pic_order_cnt_type1: None,
                max_num_ref_frames: 4,
                gaps_in_frame_num_value_allowed_flag: false,
                pic_width_in_mbs_minus1: 239,
                pic_height_in_map_units_minus1: 134,
                mb_adaptive_frame_field_flag: None,
                direct_8x8_inference_flag: true,
                frame_crop_info: None,
                sample_aspect_ratio: Some(
                    SarDimensions {
                        aspect_ratio_idc: AspectRatioIdc::Square,
                        sar_width: 0,
                        sar_height: 0,
                    },
                ),
                overscan_appropriate_flag: None,
                color_config: None,
                chroma_sample_loc: None,
                timing_info: Some(
                    TimingInfo {
                        num_units_in_tick: 1,
                        time_scale: 120,
                    },
                ),
            }
            ");
        }

        // Audio Sequence Header Tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            let (data, sound_rate, sound_size, sound_type) = match tag.data {
                FlvTagData::Audio(AudioData {
                    header:
                        AudioTagHeader::Legacy(LegacyAudioTagHeader {
                            sound_rate,
                            sound_size,
                            sound_type,
                            ..
                        }),
                    body,
                }) => (body, sound_rate, sound_size, sound_type),
                _ => panic!("expected audio data"),
            };

            assert_eq!(sound_rate, SoundRate::Hz44000);
            assert_eq!(sound_size, SoundSize::Bit16);
            assert_eq!(sound_type, SoundType::Stereo);

            // Audio data should be an AAC sequence header
            let data = match data {
                AudioTagBody::Legacy(LegacyAudioTagBody::Aac(AacAudioData::SequenceHeader(data))) => data,
                _ => panic!("expected aac sequence header"),
            };

            // The aac sequence header should be able to be decoded into an aac decoder
            // configuration record
            let aac_decoder_configuration_record =
                PartialAudioSpecificConfig::parse(&data).expect("expected aac decoder configuration record");

            assert_eq!(
                aac_decoder_configuration_record.audio_object_type,
                AudioObjectType::AacLowComplexity
            );
            assert_eq!(aac_decoder_configuration_record.sampling_frequency, 48000);
            assert_eq!(aac_decoder_configuration_record.channel_configuration, 2);
        }

        // Rest of the tags should be video / audio data
        let mut last_timestamp = 0;
        let mut read_seq_end = false;
        for tag in tags {
            assert!(tag.timestamp_ms >= last_timestamp);
            assert_eq!(tag.stream_id, 0);

            last_timestamp = tag.timestamp_ms;

            match tag.data {
                FlvTagData::Audio(AudioData {
                    body,
                    header:
                        AudioTagHeader::Legacy(LegacyAudioTagHeader {
                            sound_rate,
                            sound_size,
                            sound_type,
                            ..
                        }),
                }) => {
                    assert_eq!(sound_rate, SoundRate::Hz44000);
                    assert_eq!(sound_size, SoundSize::Bit16);
                    assert_eq!(sound_type, SoundType::Stereo);
                    match body {
                        AudioTagBody::Legacy(LegacyAudioTagBody::Aac(AacAudioData::Raw(data))) => data,
                        _ => panic!("expected aac raw packet"),
                    };
                }
                FlvTagData::Video(VideoData {
                    header:
                        VideoTagHeader {
                            frame_type,
                            data: VideoTagHeaderData::Legacy(data),
                        },
                    ..
                }) => {
                    match frame_type {
                        VideoFrameType::KeyFrame => (),
                        VideoFrameType::InterFrame => (),
                        _ => panic!("expected keyframe or interframe"),
                    }

                    match data {
                        LegacyVideoTagHeader::AvcPacket(LegacyVideoTagHeaderAvcPacket::Nalu { .. }) => {
                            assert!(!read_seq_end)
                        }
                        LegacyVideoTagHeader::AvcPacket(LegacyVideoTagHeaderAvcPacket::EndOfSequence) => {
                            assert!(!read_seq_end);
                            read_seq_end = true;
                        }
                        _ => panic!("expected avc nalu packet: {:?}", data),
                    }
                }
                _ => panic!("unexpected data"),
            };
        }

        assert!(read_seq_end);
    }

    #[test]
    fn test_demux_flv_av1_aac() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");

        let data = Bytes::from(std::fs::read(dir.join("av1_aac.flv")).expect("failed to read file"));
        let mut reader = io::Cursor::new(data);

        let flv = FlvFile::demux(&mut reader).expect("failed to demux flv");

        assert_eq!(flv.header.version, 1);
        assert!(flv.header.is_audio_present);
        assert!(flv.header.is_video_present);
        assert_eq!(flv.header.extra.len(), 0);

        let mut tags = flv.tags.into_iter();

        // Metadata tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            // This is a metadata tag
            let on_meta_data = match tag.data {
                FlvTagData::ScriptData(ScriptData::OnMetaData(data)) => data,
                _ => panic!("expected script data"),
            };

            assert_eq!(on_meta_data.audiosamplesize, Some(16.0));
            assert_eq!(on_meta_data.audiosamplerate, Some(48000.0));
            assert_eq!(on_meta_data.stereo, Some(true));
            assert_eq!(
                on_meta_data.audiocodecid,
                Some(OnMetaDataAudioCodecId::Legacy(SoundFormat::Aac))
            ); // AAC
            assert_eq!(
                on_meta_data.videocodecid,
                Some(OnMetaDataVideoCodecId::Legacy(VideoCodecId::Avc))
            ); // AVC
            assert_eq!(on_meta_data.duration, Some(0.0)); // 0 seconds (this was a live stream)
            assert_eq!(on_meta_data.width, Some(2560.0));
            assert_eq!(on_meta_data.height, Some(1440.0));
            assert_eq!(on_meta_data.framerate, Some(144.0));
            assert!(on_meta_data.videodatarate.is_some());
            assert!(on_meta_data.audiodatarate.is_some());
        }

        // Audio Sequence Header Tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            let (body, sound_rate, sound_size, sound_type) = match tag.data {
                FlvTagData::Audio(AudioData {
                    body,
                    header:
                        AudioTagHeader::Legacy(LegacyAudioTagHeader {
                            sound_rate,
                            sound_size,
                            sound_type,
                            ..
                        }),
                }) => (body, sound_rate, sound_size, sound_type),
                _ => panic!("expected audio data"),
            };

            assert_eq!(sound_rate, SoundRate::Hz44000);
            assert_eq!(sound_size, SoundSize::Bit16);
            assert_eq!(sound_type, SoundType::Stereo);

            // Audio data should be an AAC sequence header
            let data = match body {
                AudioTagBody::Legacy(LegacyAudioTagBody::Aac(AacAudioData::SequenceHeader(data))) => data,
                _ => panic!("expected aac sequence header"),
            };

            // The aac sequence header should be able to be decoded into an aac decoder
            // configuration record
            let aac_decoder_configuration_record =
                PartialAudioSpecificConfig::parse(&data).expect("expected aac decoder configuration record");

            assert_eq!(
                aac_decoder_configuration_record.audio_object_type,
                AudioObjectType::AacLowComplexity
            );
            assert_eq!(aac_decoder_configuration_record.sampling_frequency, 48000);
            assert_eq!(aac_decoder_configuration_record.channel_configuration, 2);
        }

        // Video Sequence Header Tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            // This is a video tag
            let frame_type = match tag.data {
                FlvTagData::Video(VideoData {
                    header: VideoTagHeader { frame_type, .. },
                    ..
                }) => frame_type,
                _ => panic!("expected video data"),
            };

            assert_eq!(frame_type, VideoFrameType::KeyFrame);

            // Video data should be an AVC sequence header
            let config = match tag.data {
                FlvTagData::Video(VideoData {
                    body:
                        VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                            video_four_cc: VideoFourCc::Av1,
                            packet: VideoPacket::SequenceStart(VideoPacketSequenceStart::Av1(config)),
                        }),
                    ..
                }) => config,
                _ => panic!("expected video data"),
            };

            assert_eq!(config.chroma_sample_position, 0);
            assert!(config.chroma_subsampling_x); // 5.1
            assert!(config.chroma_subsampling_y);
            assert!(!config.high_bitdepth);
            assert!(!config.twelve_bit);

            let mut reader = std::io::Cursor::new(config.config_obu);

            let header = ObuHeader::parse(&mut reader).expect("expected obu header");

            let seq_obu = SequenceHeaderObu::parse(header, &mut reader).expect("expected sequence obu");

            assert_eq!(seq_obu.max_frame_height, 1440);
            assert_eq!(seq_obu.max_frame_width, 2560);
        }

        // Rest of the tags should be video / audio data
        let mut last_timestamp = 0;
        let mut read_seq_end = false;
        for tag in tags {
            assert!(tag.timestamp_ms >= last_timestamp || tag.timestamp_ms == 0); // Timestamps should be monotonically increasing or 0
            assert_eq!(tag.stream_id, 0);

            if tag.timestamp_ms != 0 {
                last_timestamp = tag.timestamp_ms;
            }

            match tag.data {
                FlvTagData::Audio(AudioData {
                    body,
                    header:
                        AudioTagHeader::Legacy(LegacyAudioTagHeader {
                            sound_rate,
                            sound_size,
                            sound_type,
                            ..
                        }),
                }) => {
                    assert_eq!(sound_rate, SoundRate::Hz44000);
                    assert_eq!(sound_size, SoundSize::Bit16);
                    assert_eq!(sound_type, SoundType::Stereo);
                    match body {
                        AudioTagBody::Legacy(LegacyAudioTagBody::Aac(AacAudioData::Raw(data))) => data,
                        _ => panic!("expected aac raw packet"),
                    };
                }
                FlvTagData::Video(VideoData {
                    header: VideoTagHeader { frame_type, .. },
                    body: VideoTagBody::Enhanced(body),
                }) => {
                    match frame_type {
                        VideoFrameType::KeyFrame => (),
                        VideoFrameType::InterFrame => (),
                        _ => panic!("expected keyframe or interframe"),
                    }

                    match body {
                        ExVideoTagBody::NoMultitrack {
                            video_four_cc: VideoFourCc::Av1,
                            packet: VideoPacket::CodedFrames(_),
                        } => {
                            assert!(!read_seq_end);
                        }
                        ExVideoTagBody::NoMultitrack {
                            video_four_cc: VideoFourCc::Av1,
                            packet: VideoPacket::CodedFramesX { .. },
                        } => {
                            assert!(!read_seq_end);
                        }
                        ExVideoTagBody::ManyTracks(tracks) => {
                            assert!(!read_seq_end);
                            assert!(tracks.is_empty());
                        }
                        ExVideoTagBody::NoMultitrack {
                            video_four_cc: VideoFourCc::Av1,
                            packet: VideoPacket::SequenceEnd,
                        } => {
                            assert!(!read_seq_end);
                            read_seq_end = true;
                        }
                        _ => panic!("expected av1 raw packet: {:?}", body),
                    };
                }
                _ => panic!("unexpected data"),
            };
        }

        assert!(read_seq_end);
    }

    #[test]
    fn test_demux_flv_hevc_aac() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");

        let data = Bytes::from(std::fs::read(dir.join("hevc_aac.flv")).expect("failed to read file"));
        let mut reader = io::Cursor::new(data);

        let flv = FlvFile::demux(&mut reader).expect("failed to demux flv");

        assert_eq!(flv.header.version, 1);
        assert!(flv.header.is_audio_present);
        assert!(flv.header.is_video_present);
        assert_eq!(flv.header.extra.len(), 0);

        let mut tags = flv.tags.into_iter();

        // Metadata tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            let on_meta_data = match tag.data {
                FlvTagData::ScriptData(ScriptData::OnMetaData(data)) => data,
                _ => panic!("expected script data"),
            };

            assert_eq!(on_meta_data.audiosamplesize, Some(16.0));
            assert_eq!(on_meta_data.audiosamplerate, Some(48000.0));
            assert_eq!(on_meta_data.stereo, Some(true));
            assert_eq!(
                on_meta_data.audiocodecid,
                Some(OnMetaDataAudioCodecId::Legacy(SoundFormat::Aac))
            ); // AAC
            assert_eq!(
                on_meta_data.videocodecid,
                Some(OnMetaDataVideoCodecId::Legacy(VideoCodecId::Avc))
            ); // AVC
            assert_eq!(on_meta_data.duration, Some(0.0)); // 0 seconds (this was a live stream)
            assert_eq!(on_meta_data.width, Some(2560.0));
            assert_eq!(on_meta_data.height, Some(1440.0));
            assert_eq!(on_meta_data.framerate, Some(144.0));
            assert!(on_meta_data.videodatarate.is_some());
            assert!(on_meta_data.audiodatarate.is_some());
        }

        // Audio Sequence Header Tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            let (body, sound_rate, sound_size, sound_type) = match tag.data {
                FlvTagData::Audio(AudioData {
                    body,
                    header:
                        AudioTagHeader::Legacy(LegacyAudioTagHeader {
                            sound_rate,
                            sound_size,
                            sound_type,
                            ..
                        }),
                }) => (body, sound_rate, sound_size, sound_type),
                _ => panic!("expected audio data"),
            };

            assert_eq!(sound_rate, SoundRate::Hz44000);
            assert_eq!(sound_size, SoundSize::Bit16);
            assert_eq!(sound_type, SoundType::Stereo);

            // Audio data should be an AAC sequence header
            let data = match body {
                AudioTagBody::Legacy(LegacyAudioTagBody::Aac(AacAudioData::SequenceHeader(data))) => data,
                _ => panic!("expected aac sequence header"),
            };

            // The aac sequence header should be able to be decoded into an aac decoder
            // configuration record
            let aac_decoder_configuration_record =
                PartialAudioSpecificConfig::parse(&data).expect("expected aac decoder configuration record");

            assert_eq!(
                aac_decoder_configuration_record.audio_object_type,
                AudioObjectType::AacLowComplexity
            );
            assert_eq!(aac_decoder_configuration_record.sampling_frequency, 48000);
            assert_eq!(aac_decoder_configuration_record.channel_configuration, 2);
        }

        // Video Sequence Header Tag
        {
            let tag = tags.next().expect("expected tag");
            assert_eq!(tag.timestamp_ms, 0);
            assert_eq!(tag.stream_id, 0);

            // This is a video tag
            let (frame_type, config) = match tag.data {
                FlvTagData::Video(VideoData {
                    header: VideoTagHeader { frame_type, .. },
                    body:
                        VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                            video_four_cc: VideoFourCc::Hevc,
                            packet: VideoPacket::SequenceStart(VideoPacketSequenceStart::Hevc(config)),
                        }),
                }) => (frame_type, config),
                _ => panic!("expected video data"),
            };

            assert_eq!(frame_type, VideoFrameType::KeyFrame);

            assert_eq!(config.configuration_version, 1);
            assert_eq!(config.avg_frame_rate, 0);
            assert_eq!(config.constant_frame_rate, 0);
            assert_eq!(config.num_temporal_layers, 1);

            // We should be able to find a SPS NAL unit in the sequence header
            let Some(sps) = config
                .arrays
                .iter()
                .find(|a| a.nal_unit_type == scuffle_h265::NaluType::Sps)
                .and_then(|v| v.nalus.first())
            else {
                panic!("expected sps");
            };

            // We should be able to find a PPS NAL unit in the sequence header
            let Some(_) = config
                .arrays
                .iter()
                .find(|a| a.nal_unit_type == scuffle_h265::NaluType::Pps)
                .and_then(|v| v.nalus.first())
            else {
                panic!("expected pps");
            };

            // We should be able to decode the SPS NAL unit
            let sps = scuffle_h265::Sps::parse(sps.clone()).expect("expected sps");

            assert_eq!(sps.frame_rate, 144.0);
            assert_eq!(sps.width, 2560);
            assert_eq!(sps.height, 1440);
            assert_eq!(
                sps.color_config,
                Some(scuffle_h265::ColorConfig {
                    full_range: false,
                    color_primaries: 1,
                    transfer_characteristics: 1,
                    matrix_coefficients: 1,
                })
            )
        }

        // Rest of the tags should be video / audio data
        let mut last_timestamp = 0;
        let mut read_seq_end = false;
        for tag in tags {
            assert!(tag.timestamp_ms >= last_timestamp || tag.timestamp_ms == 0); // Timestamps should be monotonically increasing or 0
            assert_eq!(tag.stream_id, 0);

            if tag.timestamp_ms != 0 {
                last_timestamp = tag.timestamp_ms;
            }

            match tag.data {
                FlvTagData::Audio(AudioData {
                    body,
                    header:
                        AudioTagHeader::Legacy(LegacyAudioTagHeader {
                            sound_rate,
                            sound_size,
                            sound_type,
                            ..
                        }),
                }) => {
                    assert_eq!(sound_rate, SoundRate::Hz44000);
                    assert_eq!(sound_size, SoundSize::Bit16);
                    assert_eq!(sound_type, SoundType::Stereo);
                    match body {
                        AudioTagBody::Legacy(LegacyAudioTagBody::Aac(AacAudioData::Raw(data))) => data,
                        _ => panic!("expected aac raw packet"),
                    };
                }
                FlvTagData::Video(VideoData {
                    header: VideoTagHeader { frame_type, .. },
                    body:
                        VideoTagBody::Enhanced(ExVideoTagBody::NoMultitrack {
                            video_four_cc: VideoFourCc::Hevc,
                            packet,
                        }),
                }) => {
                    match frame_type {
                        VideoFrameType::KeyFrame => (),
                        VideoFrameType::InterFrame => (),
                        _ => panic!("expected keyframe or interframe"),
                    }

                    match packet {
                        VideoPacket::CodedFrames(_) => assert!(!read_seq_end),
                        VideoPacket::CodedFramesX { .. } => assert!(!read_seq_end),
                        VideoPacket::SequenceEnd => {
                            assert!(!read_seq_end);
                            read_seq_end = true;
                        }
                        _ => panic!("expected hevc nalu packet: {:?}", packet),
                    };
                }
                _ => panic!("unexpected data"),
            };
        }

        assert!(read_seq_end);
    }
}
