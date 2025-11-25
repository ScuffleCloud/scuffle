use std::fmt::Display;
use std::io;

use crate::Tag;

pub enum MediaDefault {
    Yes,
    No,
}

impl Display for MediaDefault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaDefault::Yes => write!(f, "YES"),
            MediaDefault::No => write!(f, "NO"),
        }
    }
}

pub enum MediaAutoSelect {
    Yes,
    No,
}

impl Display for MediaAutoSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAutoSelect::Yes => write!(f, "YES"),
            MediaAutoSelect::No => write!(f, "NO"),
        }
    }
}

pub enum MediaForced {
    Yes,
    No,
}

impl Display for MediaForced {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaForced::Yes => write!(f, "YES"),
            MediaForced::No => write!(f, "NO"),
        }
    }
}

pub enum MediaInStreamId {
    CC1,
    CC2,
    CC3,
    CC4,
    Service(u8),
}

impl Display for MediaInStreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaInStreamId::CC1 => write!(f, "CC1"),
            MediaInStreamId::CC2 => write!(f, "CC2"),
            MediaInStreamId::CC3 => write!(f, "CC3"),
            MediaInStreamId::CC4 => write!(f, "CC4"),
            MediaInStreamId::Service(n) => write!(f, "SERVICE{n}"),
        }
    }
}

pub enum MediaType {
    Audio {
        uri: Option<url::Url>,
    },
    Video {
        uri: Option<url::Url>,
    },
    Subtitles {
        uri: url::Url,
        forced: Option<MediaForced>,
    },
    ClosedCaptions {
        in_stream_id: MediaInStreamId,
    },
}

impl Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Audio { .. } => write!(f, "AUDIO"),
            MediaType::Video { .. } => write!(f, "VIDEO"),
            MediaType::Subtitles { .. } => write!(f, "SUBTITLES"),
            MediaType::ClosedCaptions { .. } => write!(f, "CLOSED-CAPTIONS"),
        }
    }
}

pub struct Media {
    pub typ: MediaType,
    pub group_id: String,
    pub language: Option<String>,
    pub assoc_language: Option<String>,
    pub name: String,
    pub default: Option<MediaDefault>,
    pub auto_select: Option<MediaAutoSelect>,
    pub characteristics: Option<String>,
    pub channels: Option<String>,
}

impl Tag for Media {
    const NAME: &'static str = "EXT-X-MEDIA";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":TYPE={}", self.typ)?;

        if let MediaType::Audio { uri: Some(uri) } | MediaType::Video { uri: Some(uri) } | MediaType::Subtitles { uri, .. } =
            &self.typ
        {
            write!(writer, ",URI=\"{}\"", uri)?;
        }

        write!(writer, ",GROUP-ID=\"{}\"", self.group_id)?;

        if let Some(lang) = self.language.as_ref() {
            write!(writer, ",LANGUAGE=\"{}\"", lang)?;
        }

        if let Some(lang) = self.assoc_language.as_ref() {
            write!(writer, ",ASSOC-LANGUAGE=\"{}\"", lang)?;
        }

        write!(writer, ",NAME=\"{}\"", self.name)?;

        if let Some(default) = self.default.as_ref() {
            write!(writer, ",DEFAULT={}", default)?;
        }

        if let Some(auto_select) = self.auto_select.as_ref() {
            write!(writer, ",AUTO-SELECT={}", auto_select)?;
        }

        if let MediaType::Subtitles {
            forced: Some(forced), ..
        } = &self.typ
        {
            write!(writer, ",FORCED={}", forced)?;
        }

        if let MediaType::ClosedCaptions { in_stream_id } = &self.typ {
            write!(writer, ",INSTREAM-ID=\"{}\"", in_stream_id)?;
        }

        if let Some(characteristics) = self.characteristics.as_ref() {
            write!(writer, ",CHARACTERISTICS=\"{}\"", characteristics)?;
        }

        if let Some(channels) = self.channels.as_ref() {
            write!(writer, ",CHANNELS=\"{}\"", channels)?;
        }

        Ok(())
    }
}
