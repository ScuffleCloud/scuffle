use std::fmt;

use aac::AudioObjectType;
use av1::AV1CodecConfigurationRecord;
use bytes::Bytes;
use flv::{SoundSize, SoundType};
use h264::AVCDecoderConfigurationRecord;
use h265::HEVCDecoderConfigurationRecord;

pub(crate) enum VideoSequenceHeader {
    Avc(AVCDecoderConfigurationRecord),
    Hevc(HEVCDecoderConfigurationRecord),
    Av1(AV1CodecConfigurationRecord),
}

pub(crate) struct AudioSequenceHeader {
    pub sound_size: SoundSize,
    pub sound_type: SoundType,
    pub data: AudioSequenceHeaderData,
}

pub(crate) enum AudioSequenceHeaderData {
    Aac(Bytes),
}

#[derive(Debug, Clone)]
pub enum TransmuxResult {
    InitSegment {
        video_settings: VideoSettings,
        audio_settings: AudioSettings,
        data: Bytes,
    },
    MediaSegment(MediaSegment),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoSettings {
    pub width: u32,
    pub height: u32,
    pub framerate: f64,
    pub bitrate: u32,
    pub codec: VideoCodec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter
    Avc {
        profile: u8,
        constraint_set: u8,
        level: u8,
    },
    /// There is barely any documentation on this.
    /// http://hevcvideo.xp3.biz/html5_video.html
    Hevc {
        general_profile_space: u8,
        profile_compatibility: u32,
        profile: u8,
        level: u8,
        tier: bool,
        constraint_indicator: u64,
    },
    /// https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter#av1
    Av1 {
        profile: u8,
        level: u8,
        tier: bool,
        depth: u8,
        monochrome: bool,
        sub_sampling_x: bool,
        sub_sampling_y: bool,
        color_primaries: u8,
        transfer_characteristics: u8,
        matrix_coefficients: u8,
        full_range_flag: bool,
    },
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoCodec::Avc {
                profile,
                constraint_set,
                level,
            } => write!(f, "avc1.{:02x}{:02x}{:02x}", profile, constraint_set, level),
            VideoCodec::Hevc {
                general_profile_space,
                profile,
                level,
                tier,
                profile_compatibility,
                constraint_indicator,
            } => write!(
                f,
                "hev1.{}{:x}.{:x}.{}{:x}.{:x}",
                match general_profile_space {
                    1 => "A",
                    2 => "B",
                    3 => "C",
                    _ => "",
                },
                profile, // 1 or 2 chars (hex)
                profile_compatibility,
                if *tier { 'H' } else { 'L' },
                level, // 1 or 2 chars (hex)
                constraint_indicator,
            ),
            VideoCodec::Av1 {
                profile,
                level,
                tier,
                depth,
                monochrome,
                sub_sampling_x,
                sub_sampling_y,
                color_primaries,
                transfer_characteristics,
                matrix_coefficients,
                full_range_flag,
            } => write!(
                f,
                "av01.{}.{}{}.{:02}.{}.{}{}{}.{:02}.{:02}.{:02}.{}",
                profile,
                level,
                if *tier { 'H' } else { 'M' },
                depth,
                if *monochrome { 1 } else { 0 },
                if *sub_sampling_x { 1 } else { 0 },
                if *sub_sampling_y { 1 } else { 0 },
                if *monochrome { 1 } else { 0 },
                color_primaries,
                transfer_characteristics,
                matrix_coefficients,
                if *full_range_flag { 1 } else { 0 },
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: u32,
    pub codec: AudioCodec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    Aac { object_type: AudioObjectType },
    Opus,
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioCodec::Aac { object_type } => write!(f, "mp4a.40.{}", u16::from(*object_type)),
            AudioCodec::Opus => write!(f, "opus"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
}

#[derive(Debug, Clone)]
pub struct MediaSegment {
    pub data: Bytes,
    pub ty: MediaType,
    pub keyframe: bool,
    pub timestamp: u64,
}

impl TransmuxResult {
    pub fn into_bytes(self) -> Bytes {
        match self {
            TransmuxResult::InitSegment { data, .. } => data,
            TransmuxResult::MediaSegment(data) => data.data,
        }
    }
}
