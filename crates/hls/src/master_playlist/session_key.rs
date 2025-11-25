use std::io;

use crate::Tag;
use crate::media_segment::KeyAes;

pub struct SessionKey {
    pub key: KeyAes,
}

impl Tag for SessionKey {
    const NAME: &'static str = "";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":{}", self.key)
    }
}
