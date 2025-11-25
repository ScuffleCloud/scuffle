use crate::Tag;

pub enum PlaylistType {
    Event,
    Vod,
}

impl Tag for PlaylistType {
    const NAME: &'static str = "EXT-X-PLAYLIST-TYPE";

    fn write_value(&self, mut writer: impl std::io::Write) -> Result<(), std::io::Error> {
        match self {
            PlaylistType::Event => write!(writer, ":EVENT"),
            PlaylistType::Vod => write!(writer, ":VOD"),
        }
    }
}
