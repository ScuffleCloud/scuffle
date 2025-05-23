use std::io::{
    Write, {self},
};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use scuffle_aac::AudioObjectType;
use scuffle_flv::header::FlvHeader;
use scuffle_mp4::codec::{AudioCodec, VideoCodec};

use crate::define::{AudioSettings, VideoSettings};
use crate::{TransmuxResult, Transmuxer};

#[test]
fn test_transmuxer_avc_aac() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let data = std::fs::read(dir.join("avc_aac.flv").to_str().unwrap()).unwrap();

    let mut transmuxer = Transmuxer::new();

    // Read the flv header first
    let mut cursor = io::Cursor::new(data.into());
    FlvHeader::demux(&mut cursor).unwrap();

    let pos = cursor.position() as usize;

    let data = cursor.into_inner().slice(pos..);

    let mut writer = Vec::new();

    transmuxer.demux(data).unwrap();

    while let Some(data) = transmuxer.mux().unwrap() {
        match &data {
            TransmuxResult::InitSegment {
                video_settings,
                audio_settings,
                ..
            } => {
                assert_eq!(
                    video_settings,
                    &VideoSettings {
                        width: 3840,
                        height: 2160,
                        framerate: 60.0,
                        bitrate: 7358243,
                        timescale: 60000,
                        codec: VideoCodec::Avc {
                            profile: 100,
                            level: 51,
                            constraint_set: 0,
                        }
                    }
                );
                assert_eq!(video_settings.codec.to_string(), "avc1.640033");

                assert_eq!(
                    audio_settings,
                    &AudioSettings {
                        sample_rate: 48000,
                        channels: 2,
                        bitrate: 130127,
                        timescale: 48000,
                        codec: AudioCodec::Aac {
                            object_type: AudioObjectType::AacLowComplexity,
                        }
                    }
                );
                assert_eq!(audio_settings.codec.to_string(), "mp4a.40.2");
            }
            _ => {}
        }
        writer.write_all(&data.into_bytes()).unwrap();
    }

    let mut ffprobe = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-fpsprobesize")
        .arg("20000")
        .arg("-show_format")
        .arg("-show_streams")
        .arg("-print_format")
        .arg("json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    ffprobe.stdin.as_mut().unwrap().write_all(&writer).expect("write to stdin");

    let output = ffprobe.wait_with_output().unwrap();
    assert!(output.status.success());

    let output = String::from_utf8(output.stdout).unwrap();

    println!("{output}");

    // Check the output is valid.
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(json["format"]["format_name"], "mov,mp4,m4a,3gp,3g2,mj2");
    assert_eq!(json["format"]["duration"], "1.002667");
    assert_eq!(json["format"]["tags"]["major_brand"], "iso5");
    assert_eq!(json["format"]["tags"]["minor_version"], "512");
    assert_eq!(json["format"]["tags"]["compatible_brands"], "iso5iso6avc1mp41");

    assert_eq!(json["streams"][0]["codec_name"], "h264");
    assert_eq!(json["streams"][0]["codec_type"], "video");
    assert_eq!(json["streams"][0]["width"], 3840);
    assert_eq!(json["streams"][0]["height"], 2160);
    assert_eq!(json["streams"][0]["r_frame_rate"], "60/1");
    assert_eq!(json["streams"][0]["avg_frame_rate"], "60/1");

    assert_eq!(json["streams"][1]["codec_name"], "aac");
    assert_eq!(json["streams"][1]["codec_type"], "audio");
    assert_eq!(json["streams"][1]["sample_rate"], "48000");
    assert_eq!(json["streams"][1]["channels"], 2);
}

#[test]
fn test_transmuxer_av1_aac() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let data = std::fs::read(dir.join("av1_aac.flv").to_str().unwrap()).unwrap();

    let mut transmuxer = Transmuxer::new();

    // Read the flv header first
    let mut cursor = io::Cursor::new(data.into());
    FlvHeader::demux(&mut cursor).unwrap();

    let pos = cursor.position() as usize;

    let data = cursor.into_inner().slice(pos..);

    let mut writer = Vec::new();

    transmuxer.demux(data).unwrap();

    while let Some(data) = transmuxer.mux().unwrap() {
        match &data {
            TransmuxResult::InitSegment {
                video_settings,
                audio_settings,
                ..
            } => {
                assert_eq!(
                    video_settings,
                    &VideoSettings {
                        width: 2560,
                        height: 1440,
                        framerate: 144.0,
                        bitrate: 2560000,
                        timescale: 144000,
                        codec: VideoCodec::Av1 {
                            profile: 0,
                            level: 13,
                            tier: false,
                            depth: 8,
                            sub_sampling_x: true,
                            sub_sampling_y: true,
                            monochrome: false,
                            full_range_flag: false,
                            color_primaries: 1,
                            transfer_characteristics: 1,
                            matrix_coefficients: 1,
                        }
                    }
                );
                assert_eq!(video_settings.codec.to_string(), "av01.0.13M.08.0.110.01.01.01.0");

                assert_eq!(
                    audio_settings,
                    &AudioSettings {
                        sample_rate: 48000,
                        bitrate: 163840,
                        channels: 2,
                        timescale: 48000,
                        codec: AudioCodec::Aac {
                            object_type: AudioObjectType::AacLowComplexity,
                        }
                    }
                );
                assert_eq!(audio_settings.codec.to_string(), "mp4a.40.2");
            }
            _ => {}
        }

        writer.write_all(&data.into_bytes()).unwrap();
    }

    let mut ffprobe = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-fpsprobesize")
        .arg("20000")
        .arg("-show_format")
        .arg("-show_streams")
        .arg("-print_format")
        .arg("json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    ffprobe.stdin.as_mut().unwrap().write_all(&writer).unwrap();

    let output = ffprobe.wait_with_output().unwrap();
    assert!(output.status.success());

    // Check the output is valid.
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["format"]["format_name"], "mov,mp4,m4a,3gp,3g2,mj2");
    assert_eq!(json["format"]["tags"]["major_brand"], "iso5");
    assert_eq!(json["format"]["tags"]["minor_version"], "512");
    assert_eq!(json["format"]["duration"], "2.816000");
    assert_eq!(json["format"]["tags"]["compatible_brands"], "iso5iso6av01mp41");

    assert_eq!(json["streams"][0]["codec_name"], "av1");
    assert_eq!(json["streams"][0]["codec_type"], "video");
    assert_eq!(json["streams"][0]["width"], 2560);
    assert_eq!(json["streams"][0]["height"], 1440);
    assert_eq!(json["streams"][0]["r_frame_rate"], "144/1");

    assert_eq!(json["streams"][1]["codec_name"], "aac");
    assert_eq!(json["streams"][1]["codec_type"], "audio");
    assert_eq!(json["streams"][1]["sample_rate"], "48000");
    assert_eq!(json["streams"][1]["channels"], 2);
}

#[test]
fn test_transmuxer_hevc_aac() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let data = std::fs::read(dir.join("hevc_aac.flv").to_str().unwrap()).unwrap();

    let mut transmuxer = Transmuxer::new();

    // Read the flv header first
    let mut cursor = io::Cursor::new(data.into());
    FlvHeader::demux(&mut cursor).unwrap();

    let pos = cursor.position() as usize;

    let data = cursor.into_inner().slice(pos..);

    let mut writer = Vec::new();

    transmuxer.demux(data).unwrap();

    while let Some(data) = transmuxer.mux().unwrap() {
        match &data {
            TransmuxResult::InitSegment {
                video_settings,
                audio_settings,
                ..
            } => {
                assert_eq!(
                    video_settings,
                    &VideoSettings {
                        width: 3840,
                        height: 2160,
                        framerate: 60.0,
                        bitrate: 389904,
                        timescale: 60000,
                        codec: VideoCodec::Hevc {
                            general_profile_space: 0,
                            profile_compatibility: scuffle_h265::ProfileCompatibilityFlags::MainProfile
                                | scuffle_h265::ProfileCompatibilityFlags::Main10Profile,
                            profile: 1,
                            level: 153,
                            tier: false,
                            constraint_indicator: (1 << 47) | (1 << 44), // 1. bit and 4. bit,
                        }
                    }
                );
                assert_eq!(video_settings.codec.to_string(), "hev1.1.60.L99.90");

                assert_eq!(
                    audio_settings,
                    &AudioSettings {
                        sample_rate: 48000,
                        channels: 2,
                        bitrate: 135360,
                        timescale: 48000,
                        codec: AudioCodec::Aac {
                            object_type: AudioObjectType::AacLowComplexity,
                        }
                    }
                );
                assert_eq!(audio_settings.codec.to_string(), "mp4a.40.2");
            }
            _ => {}
        }

        writer.write_all(&data.into_bytes()).unwrap();
    }

    let mut ffprobe = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-fpsprobesize")
        .arg("20000")
        .arg("-show_format")
        .arg("-show_streams")
        .arg("-print_format")
        .arg("json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    ffprobe.stdin.as_mut().unwrap().write_all(&writer).expect("write to stdin");

    let output = ffprobe.wait_with_output().unwrap();
    assert!(output.status.success());

    // Check the output is valid.
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["format"]["format_name"], "mov,mp4,m4a,3gp,3g2,mj2");
    assert_eq!(json["format"]["duration"], "2.026667");
    assert_eq!(json["format"]["tags"]["major_brand"], "iso5");
    assert_eq!(json["format"]["tags"]["minor_version"], "512");
    assert_eq!(json["format"]["tags"]["compatible_brands"], "iso5iso6hev1mp41");

    assert_eq!(json["streams"][0]["codec_name"], "hevc");
    assert_eq!(json["streams"][0]["codec_type"], "video");
    assert_eq!(json["streams"][0]["width"], 3840);
    assert_eq!(json["streams"][0]["height"], 2160);
    assert_eq!(json["streams"][0]["r_frame_rate"], "60/1");

    assert_eq!(json["streams"][1]["codec_name"], "aac");
    assert_eq!(json["streams"][1]["codec_type"], "audio");
    assert_eq!(json["streams"][1]["sample_rate"], "48000");
    assert_eq!(json["streams"][1]["channels"], 2);
}
