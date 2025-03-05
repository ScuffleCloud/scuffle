use crate::command_messages::define::CommandResultLevel;

/// NetStream commands as defined in 7.2.2.
#[derive(Debug, Clone)]
pub enum NetStreamCommand {
    Play,
    Play2,
    DeleteStream {
        stream_id: f64,
    },
    CloseStream,
    ReceiveAudio,
    ReceiveVideo,
    Publish {
        publishing_name: String,
        publishing_type: NetStreamCommandPublishPublishingType,
    },
    Seek,
    Pause,
    OnStatus {
        level: CommandResultLevel,
        code: String,
        description: String,
    },
}

/// NetStream command publish publishing type
#[derive(Debug, Clone)]
pub enum NetStreamCommandPublishPublishingType {
    Live,
    Record,
    Append,
}
