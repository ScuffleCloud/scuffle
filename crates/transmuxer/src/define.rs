use bytes::Bytes;
use scuffle_aac::AudioObjectType;
use scuffle_av1::AV1CodecConfigurationRecord;
use scuffle_flv::audio::header::legacy::{SoundSize, SoundType};
use scuffle_h264::AVCDecoderConfigurationRecord;
use scuffle_h265::HEVCDecoderConfigurationRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// <https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter>
    Avc { profile: u8, constraint_set: u8, level: u8 },
    /// There is barely any documentation on this.
    /// <https://hevcvideo.xp3.biz/html5_video.html>
    Hevc {
        general_profile_space: u8,
        profile_compatibility: scuffle_h265::ProfileCompatibilityFlags,
        profile: u8,
        level: u8,
        tier: bool,
        constraint_indicator: u64,
    },
    /// <https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter#av1>
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

impl std::fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoCodec::Avc {
                profile,
                constraint_set,
                level,
            } => write!(f, "avc1.{profile:02x}{constraint_set:02x}{level:02x}"),
            VideoCodec::Hevc {
                general_profile_space,
                profile,
                level,
                tier,
                profile_compatibility,
                constraint_indicator,
            } => {
                let profile_compatibility = profile_compatibility
                    .bits()
                    .to_be_bytes()
                    .into_iter()
                    .filter(|b| *b != 0)
                    .map(|b| format!("{b:x}"))
                    .collect::<String>();
                let constraint_indicator = constraint_indicator
                    .to_be_bytes()
                    .into_iter()
                    .filter(|b| *b != 0)
                    .map(|b| format!("{b:x}"))
                    .collect::<Vec<_>>()
                    .join(".");

                write!(
                    f,
                    "hev1.{}{:x}.{}.{}{:x}.{}",
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
                )
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    Aac { object_type: AudioObjectType },
    Opus,
}

impl std::fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioCodec::Aac { object_type } => write!(f, "mp4a.40.{}", u16::from(*object_type)),
            AudioCodec::Opus => write!(f, "opus"),
        }
    }
}

pub(crate) enum VideoSequenceHeader<'a> {
    Avc(AVCDecoderConfigurationRecord<'a>),
    Hevc(HEVCDecoderConfigurationRecord<'a>),
    Av1(AV1CodecConfigurationRecord<'a>),
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
    pub timescale: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: u32,
    pub codec: AudioCodec,
    pub timescale: u32,
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
