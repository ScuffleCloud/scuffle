use crate::Tag;
use crate::basic::Version;

pub struct IFramesOnly;

impl Tag for IFramesOnly {
    const NAME: &'static str = "EXT-X-I-FRAMES-ONLY";

    fn min_version(&self) -> Version {
        Version(4)
    }
}
