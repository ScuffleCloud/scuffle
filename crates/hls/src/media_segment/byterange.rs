use std::fmt::Display;

use crate::Tag;
use crate::basic::Version;

#[derive(Debug)]
pub struct ByteRange {
    pub length: u64,
    pub start: Option<u64>,
}

impl Display for ByteRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.length)?;
        if let Some(start) = self.start {
            write!(f, "@{start}")?;
        }

        Ok(())
    }
}

impl Tag for ByteRange {
    const NAME: &'static str = "EXT-X-BYTERANGE";

    fn min_version(&self) -> Version {
        Version(4)
    }

    fn write_value(&self, mut writer: impl std::io::Write) -> Result<(), std::io::Error> {
        write!(writer, ":{}", self)
    }
}
