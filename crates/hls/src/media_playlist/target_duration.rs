use std::io;

use crate::Tag;

pub struct TargetDuration(pub u64);

impl Tag for TargetDuration {
    const NAME: &'static str = "EXT-X-TARGETDURATION";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":{}", self.0)
    }
}
