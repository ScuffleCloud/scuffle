use std::io;

#[derive(Debug, thiserror::Error)]
pub enum TransmuxError {
    #[error("invalid video dimensions")]
    InvalidVideoDimensions,
    #[error("invalid video frame rate")]
    InvalidVideoFrameRate,
    #[error("invalid audio sample rate")]
    InvalidAudioSampleRate,
    #[error("invalid audio channels")]
    InvalidAudioChannels,
    #[error("invalid audio sample size")]
    InvalidAudioSampleSize,
    #[error("invalid hevc decoder configuration record")]
    InvalidHEVCDecoderConfigurationRecord,
    #[error("invalid av1 decoder configuration record")]
    InvalidAv1DecoderConfigurationRecord,
    #[error("invalid avc decoder configuration record")]
    InvalidAVCDecoderConfigurationRecord,
    #[error("no sequence headers")]
    NoSequenceHeaders,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("flv error: {0}")]
    Flv(#[from] scuffle_flv::error::Error),
}
