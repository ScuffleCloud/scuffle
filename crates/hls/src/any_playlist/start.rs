use std::fmt::Display;
use std::io;

use crate::Tag;

pub enum StartPrecise {
    Yes,
    No,
}

impl Display for StartPrecise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartPrecise::Yes => write!(f, "YES"),
            StartPrecise::No => write!(f, "NO"),
        }
    }
}

pub struct Start {
    pub time_offset: f64,
    pub precise: Option<StartPrecise>,
}

impl Tag for Start {
    const NAME: &'static str = "EXT-X-START";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":TIME-OFFSET={}", self.time_offset)?;

        if let Some(precise) = self.precise.as_ref() {
            write!(writer, ",PRECISE={}", precise)?;
        }

        Ok(())
    }
}
