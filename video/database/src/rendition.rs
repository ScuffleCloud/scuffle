use sqlx::postgres::PgHasArrayType;

use sqlx::Type;

#[derive(Debug, sqlx::Type, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(type_name = "rendition")]
pub enum Rendition {
    #[sqlx(rename = "VIDEO_SOURCE")]
    VideoSource,
    #[sqlx(rename = "VIDEO_HD")]
    VideoHd,
    #[sqlx(rename = "VIDEO_SD")]
    VideoSd,
    #[sqlx(rename = "VIDEO_LD")]
    VideoLd,
    #[sqlx(rename = "AUDIO_SOURCE")]
    AudioSource,
}

impl PgHasArrayType for Rendition {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        <Self as Type<sqlx::Postgres>>::type_info()
    }
}

impl From<Rendition> for pb::scuffle::video::v1::types::Rendition {
    fn from(value: Rendition) -> Self {
        match value {
            Rendition::VideoSource => Self::VideoSource,
            Rendition::VideoHd => Self::VideoHd,
            Rendition::VideoSd => Self::VideoSd,
            Rendition::VideoLd => Self::VideoLd,
            Rendition::AudioSource => Self::AudioSource,
        }
    }
}

impl From<pb::scuffle::video::v1::types::Rendition> for Rendition {
    fn from(value: pb::scuffle::video::v1::types::Rendition) -> Self {
        match value {
            pb::scuffle::video::v1::types::Rendition::VideoSource => Self::VideoSource,
            pb::scuffle::video::v1::types::Rendition::VideoHd => Self::VideoHd,
            pb::scuffle::video::v1::types::Rendition::VideoSd => Self::VideoSd,
            pb::scuffle::video::v1::types::Rendition::VideoLd => Self::VideoLd,
            pb::scuffle::video::v1::types::Rendition::AudioSource => Self::AudioSource,
        }
    }
}

impl std::fmt::Display for Rendition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VideoSource => write!(f, "video_source"),
            Self::VideoHd => write!(f, "video_hd"),
            Self::VideoSd => write!(f, "video_sd"),
            Self::VideoLd => write!(f, "video_ld"),
            Self::AudioSource => write!(f, "audio_source"),
        }
    }
}

impl std::str::FromStr for Rendition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "video_source" => Ok(Self::VideoSource),
            "video_hd" => Ok(Self::VideoHd),
            "video_sd" => Ok(Self::VideoSd),
            "video_ld" => Ok(Self::VideoLd),
            "audio_source" => Ok(Self::AudioSource),
            _ => Err(()),
        }
    }
}
