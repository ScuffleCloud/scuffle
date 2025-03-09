use bytes::Bytes;

use crate::command_messages::Command;
use crate::protocol_control_messages::ProtocolControlMessageSetChunkSize;

#[derive(Debug)]
pub enum MessageData<'a> {
    // Protocol Control Messages
    // The other protocol control messages are not implemented here
    // because they are not needed in this implementation.
    SetChunkSize(ProtocolControlMessageSetChunkSize),
    // RTMP Command Messages
    Amf0Command(Command<'a>),
    Amf0Data {
        data: Bytes,
    },
    AudioData {
        data: Bytes,
    },
    VideoData {
        data: Bytes,
    },
    /// Unknown
    Unknown {
        data: Bytes,
    },
}

nutype_enum::nutype_enum! {
    pub enum MessageType(u8) {
        // Protocol Control Messages
        SetChunkSize = 1,
        Abort = 2,
        Acknowledgement = 3,
        UserControlEvent = 4,
        WindowAcknowledgementSize = 5,
        SetPeerBandwidth = 6,
        // RTMP Command Messages
        Audio = 8,
        Video = 9,
        DataAMF3 = 15,
        SharedObjAMF3 = 16,
        CommandAMF3 = 17,
        DataAMF0 = 18,
        SharedObjAMF0 = 19,
        CommandAMF0 = 20,
        Aggregate = 22,
    }
}
