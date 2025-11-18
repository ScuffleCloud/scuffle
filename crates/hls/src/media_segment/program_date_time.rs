use std::io;

use crate::Tag;

pub struct ProgramDateTime {
    pub date_time_msec: chrono::DateTime<chrono::Utc>,
}

impl Tag for ProgramDateTime {
    const NAME: &'static str = "EXT-X-PROGRAM-DATE-TIME";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":{}", self.date_time_msec.format("%+"))
    }
}
