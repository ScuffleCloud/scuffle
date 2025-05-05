//! Message types and definitions.

use bytes::Bytes;

use crate::command_messages::Command;
use crate::protocol_control_messages::{
    ProtocolControlMessageSetChunkSize, ProtocolControlMessageWindowAcknowledgementSize,
};

pub mod reader;

/// Different types of messages that can be sent or received.
///
/// Defined by:
/// - Legacy RTMP spec, 5.4
#[derive(Debug)]
pub enum MessageData<'a> {
    // Protocol Control Messages
    /// Set Chunk Size message
    SetChunkSize(ProtocolControlMessageSetChunkSize),
    /// Abort message
    ///
    /// Not implemented.
    Abort,
    /// Acknowledgement message
    ///
    /// Read not implemented.
    Acknowledgement,
    /// User Control Event message
    ///
    /// Not implemented.
    UserControlEvent,
    /// Set Acknowledgement Window Size message
    SetAcknowledgementWindowSize(ProtocolControlMessageWindowAcknowledgementSize),
    /// Set Peer Bandwidth message
    ///
    /// Read not implemented.
    SetPeerBandwidth,

    // RTMP Command Messages
    /// Audio message
    ///
    /// > The client or the server sends this message to send audio data to the peer.
    ///
    /// Usually contains FLV AUDIODATA.
    AudioData {
        /// The audio data.
        data: Bytes,
    },
    /// Video message
    ///
    /// > The client or the server sends this message to send video data to the peer.
    ///
    /// Usually contains FLV VIDEODATA.
    VideoData {
        /// The video data.
        data: Bytes,
    },
    /// Amf3 metadata message
    ///
    /// Not implemented.
    DataAmf3,
    /// Amf3 shared object message
    ///
    /// Not implemented.
    SharedObjAmf3,
    /// Amf3 command message
    ///
    /// Not implemented.
    CommandAmf3,
    /// Amf0 metadata message
    ///
    /// > The client or the server sends this message to send Metadata or any
    /// > user data to the peer. Metadata includes details about the
    /// > data(audio, video etc.) like creation time, duration, theme and so on.
    DataAmf0 {
        /// The metadata.
        data: Bytes,
    },
    /// Amf0 shared object message
    ///
    /// Not implemented.
    SharedObjAmf0,
    /// Amf0 command message
    ///
    /// > Command messages carry the AMF-encoded commands between the client and the server.
    Amf0Command(Command<'a>),
    /// Aggregate message
    ///
    /// Not implemented.
    Aggregate,
    /// Any other undefined messages.
    Unknown(UnknownMessage),
}

/// Any undefined message.
#[derive(Debug)]
pub struct UnknownMessage {
    /// The message type ID.
    pub msg_type_id: MessageType,
    /// The message data.
    pub data: Bytes,
}

nutype_enum::nutype_enum! {
    /// One byte field to represent the message type.
    ///
    /// A range of type IDs (1-6) are reserved for protocol control messages.
    pub enum MessageType(u8) {
        // Protocol Control Messages
        /// Set Chunk Size
        SetChunkSize = 1,
        /// Abort
        Abort = 2,
        /// Acknowledgement
        Acknowledgement = 3,
        /// User Control Messages
        UserControlEvent = 4,
        /// Window Acknowledgement Size
        WindowAcknowledgementSize = 5,
        /// Set Peer Bandwidth
        SetPeerBandwidth = 6,
        // RTMP Command Messages
        /// Audio Data
        Audio = 8,
        /// Video Data
        Video = 9,
        /// Amf3-encoded Metadata
        DataAMF3 = 15,
        /// Amf3-encoded Shared Object
        SharedObjAMF3 = 16,
        /// Amf3-encoded Command
        CommandAMF3 = 17,
        /// Amf0-encoded Metadata
        DataAMF0 = 18,
        /// Amf0-encoded Shared Object
        SharedObjAMF0 = 19,
        /// Amf0-encoded Command
        CommandAMF0 = 20,
        /// Aggregate Message
        Aggregate = 22,
    }
}
