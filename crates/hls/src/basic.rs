use std::io;

use crate::Tag;

pub struct ExtM3u;

impl Tag for ExtM3u {
    const NAME: &'static str = "EXTM3U";
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtVersion(pub u64);

impl Default for ExtVersion {
    fn default() -> Self {
        Self(1)
    }
}

impl Tag for ExtVersion {
    const NAME: &'static str = "EXT-X-VERSION";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":{}", self.0)
    }
}
