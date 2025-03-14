use std::borrow::Cow;

use crate::command_messages::define::CommandResultLevel;

/// NetStream commands as defined in 7.2.2.
#[derive(Debug, Clone, PartialEq)]
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
        tc_url: Option<Cow<'a, str>>,
        code: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
        level: CommandResultLevel,
    },
}

/// NetStream command publish publishing type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetStreamCommandPublishPublishingType {
    Live,
    Record,
    Append,
    Unknown(String),
}
