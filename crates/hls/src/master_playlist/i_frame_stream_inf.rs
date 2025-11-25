use std::io;

use crate::Tag;
use crate::master_playlist::StreamInfHdcpLevel;

pub struct IFrameStreamInf {
    pub bandwidth: u64,
    pub average_bandwidth: Option<u64>,
    pub codecs: Vec<String>,
    pub resolution: Option<(u64, u64)>,
    pub hdcp_level: Option<StreamInfHdcpLevel>,
    pub video: Option<String>,
    pub uri: url::Url,
}

impl Tag for IFrameStreamInf {
    const NAME: &'static str = "EXT-X-I-FRAME-STREAM-INF";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":BANDWIDTH={}", self.bandwidth)?;

        if let Some(average_bandwidth) = self.average_bandwidth {
            write!(writer, ",AVERAGE-BANDWIDTH={}", average_bandwidth)?;
        }

        write!(writer, ",CODECS=\"{}\"", self.codecs.join(","))?;

        if let Some((w, h)) = self.resolution {
            write!(writer, ",RESOLUTION={}x{}", w, h)?;
        }

        if let Some(hdcp_level) = self.hdcp_level.as_ref() {
            write!(writer, ",HDCP-LEVEL={}", hdcp_level)?;
        }

        if let Some(video) = self.video.as_ref() {
            write!(writer, ",VIDEO=\"{}\"", video)?;
        }

        write!(writer, ",URI=\"{}\"", self.uri)?;

        Ok(())
    }
}
