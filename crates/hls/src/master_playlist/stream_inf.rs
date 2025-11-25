use std::fmt::Display;
use std::io;

use crate::Tag;

pub enum StreamInfHdcpLevel {
    Type0,
    None,
}

impl Display for StreamInfHdcpLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamInfHdcpLevel::Type0 => write!(f, "TYPE-0"),
            StreamInfHdcpLevel::None => write!(f, "NONE"),
        }
    }
}

pub enum StreamInfClosedCaptions {
    None,
    ClosedCaptions(String),
}

pub struct StreamInf {
    pub bandwidth: u64,
    pub average_bandwidth: Option<u64>,
    pub codecs: Vec<String>,
    pub resolution: Option<(u64, u64)>,
    pub frame_rate: Option<f64>,
    pub hdcp_level: Option<StreamInfHdcpLevel>,
    pub audio: Option<String>,
    pub video: Option<String>,
    pub subtitles: Option<String>,
    pub closed_captions: Option<StreamInfClosedCaptions>,
    pub uri: url::Url,
}

impl Tag for StreamInf {
    const NAME: &'static str = "EXT-X-STREAM-INF";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":BANDWIDTH={}", self.bandwidth)?;

        if let Some(average_bandwidth) = self.average_bandwidth {
            write!(writer, ",AVERAGE-BANDWIDTH={}", average_bandwidth)?;
        }

        write!(writer, ",CODECS=\"{}\"", self.codecs.join(","))?;

        if let Some((w, h)) = self.resolution {
            write!(writer, ",RESOLUTION={}x{}", w, h)?;
        }

        if let Some(frame_rate) = self.frame_rate {
            write!(writer, ",FRAME-RATE={:.3}", frame_rate)?;
        }

        if let Some(hdcp_level) = self.hdcp_level.as_ref() {
            write!(writer, ",HDCP-LEVEL={}", hdcp_level)?;
        }

        if let Some(audio) = self.audio.as_ref() {
            write!(writer, ",AUDIO=\"{}\"", audio)?;
        }

        if let Some(video) = self.video.as_ref() {
            write!(writer, ",VIDEO=\"{}\"", video)?;
        }

        if let Some(subtitles) = self.subtitles.as_ref() {
            write!(writer, ",SUBTITLES=\"{}\"", subtitles)?;
        }

        match self.closed_captions.as_ref() {
            Some(StreamInfClosedCaptions::None) => write!(writer, ",CLOSED-CAPTIONS=NONE")?,
            Some(StreamInfClosedCaptions::ClosedCaptions(v)) => write!(writer, ",CLOSED-CAPTIONS=\"{}\"", v)?,
            None => {}
        }

        write!(writer, "\n{}", self.uri)?;

        Ok(())
    }
}
