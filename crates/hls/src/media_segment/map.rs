use std::io;

use crate::{Tag, basic::ExtVersion, media_segment::ByteRange};

#[derive(Debug)]
pub struct Map {
    uri: url::Url,
    byte_range: Option<ByteRange>,
}

impl Tag for Map {
    const NAME: &'static str = "EXT-X-MAP";

    fn min_version(&self) -> ExtVersion {
        // TODO: Check if 5 is sufficient

        // Use of the EXT-X-MAP tag in a Media Playlist that contains the EXT-X-I-FRAMES-ONLY
        // tag REQUIRES a compatibility version number of 5 or greater.
        // Use of the EXT-X-MAP tag in a Media Playlist that DOES NOT
        // contain the EXT-X-I-FRAMES-ONLY tag REQUIRES a compatibility version
        // number of 6 or greater.
        ExtVersion(6)
    }

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":URI=\"{}\"", self.uri)?;
        if let Some(byte_range) = self.byte_range.as_ref() {
            write!(writer, ",BYTERANGE=\"{byte_range}\"")?;
        }
        Ok(())
    }
}
