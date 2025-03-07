use std::borrow::Cow;

use crate::command_messages::define::CommandResultLevel;

/// NetStream commands as defined in 7.2.2.
#[derive(Debug, Clone)]
pub enum NetStreamCommand<'a> {
    Play,
    Play2,
    DeleteStream {
        stream_id: f64,
    },
    CloseStream,
    ReceiveAudio,
    ReceiveVideo,
    Publish {
        publishing_name: Cow<'a, str>,
        publishing_type: NetStreamCommandPublishPublishingType,
    },
    Seek,
    Pause,
    OnStatus {
        level: CommandResultLevel,
        code: Cow<'a, str>,
        description: Cow<'a, str>,
    },
}

/// NetStream command publish publishing type
#[derive(Debug, Clone)]
pub enum NetStreamCommandPublishPublishingType {
    Live,
    Record,
    Append,
    Unknown(String),
}
