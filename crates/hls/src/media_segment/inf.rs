use std::fmt::Display;

use crate::{Tag, basic::ExtVersion};

#[derive(Debug)]
pub enum InfDuration {
    Int(u64),
    Float(f64),
}

impl From<f64> for InfDuration {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<u64> for InfDuration {
    fn from(value: u64) -> Self {
        Self::Int(value)
    }
}

impl Display for InfDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(v) => v.fmt(f),
            Self::Int(v) => v.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Inf {
    pub duration: InfDuration,
    pub title: Option<String>,
}

impl Tag for Inf {
    const NAME: &'static str = "EXTINF";

    fn min_version(&self) -> ExtVersion {
        match self.duration {
            InfDuration::Float(_) => ExtVersion(3),
            InfDuration::Int(_) => ExtVersion::default(),
        }
    }

    fn write_value(&self, mut writer: impl std::io::Write) -> Result<(), std::io::Error> {
        write!(writer, ":{}", self.duration)?;

        if let Some(title) = self.title.as_ref() {
            write!(writer, ",{title}")?;
        }

        Ok(())
    }
}
