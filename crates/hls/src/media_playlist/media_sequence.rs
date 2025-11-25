use std::io;

use crate::Tag;

pub struct MediaSequence(pub u64);

impl Tag for MediaSequence {
    const NAME: &'static str = "EXT-X-MEDIA-SEQUENCE";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":{}", self.0)
    }
}
