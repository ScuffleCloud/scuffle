use std::io;

use crate::Tag;

pub struct DiscontinuitySequence(pub u64);

impl Tag for DiscontinuitySequence {
    const NAME: &'static str = "EXT-X-DISCONTINUITY-SEQUENCE";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":{}", self.0)
    }
}
