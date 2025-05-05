//! User control messages.
//!
//! Defined by:
//! - Legacy RTMP spec, 6.2

pub mod writer;

nutype_enum::nutype_enum! {
    /// The type of user control message event.
    pub enum EventType(u16) {
        /// > The server sends this event to notify the client
        /// > that a stream has become functional and can be
        /// > used for communication. By default, this event
        /// > is sent on ID 0 after the application connect
        /// > command is successfully received from the
        /// > client. The event data is 4-byte and represents
        /// > the stream ID of the stream that became
        /// > functional.
        StreamBegin = 0,
        /// > The server sends this event to notify the client
        /// > that the playback of data is over as requested
        /// > on this stream. No more data is sent without
        /// > issuing additional commands. The client discards
        /// > the messages received for the stream. The
        /// > 4 bytes of event data represent the ID of the
        /// > stream on which playback has ended.
        StreamEOF = 1,
        /// > The server sends this event to notify the client
        /// > that there is no more data on the stream. If the
        /// > server does not detect any message for a time
        /// > period, it can notify the subscribed clients
        /// > that the stream is dry. The 4 bytes of event
        /// > data represent the stream ID of the dry stream.
        StreamDry = 2,
        /// > The client sends this event to inform the server
        /// > of the buffer size (in milliseconds) that is
        /// > used to buffer any data coming over a stream.
        /// > This event is sent before the server starts
        /// > processing the stream. The first 4 bytes of the
        /// > event data represent the stream ID and the next
        /// > 4 bytes represent the buffer length, in milliseconds.
        SetBufferLength = 3,
        /// > The server sends this event to notify the client
        /// > that the stream is a recorded stream. The
        /// > 4 bytes event data represent the stream ID of
        /// > the recorded stream.
        StreamIsRecorded = 4,
        /// > The server sends this event to test whether the
        /// > client is reachable. Event data is a 4-byte
        /// > timestamp, representing the local server time
        /// > when the server dispatched the command. The
        /// > client responds with PingResponse on receiving
        /// > MsgPingRequest.
        PingRequest = 6,
        /// > The client sends this event to the server in
        /// > response to the ping request. The event data is
        /// > a 4-byte timestamp, which was received with the
        /// > PingRequest request.
        PingResponse = 7,
    }
}

/// > The server sends this event to notify the client
/// > that a stream has become functional and can be
/// > used for communication. By default, this event
/// > is sent on ID 0 after the application connect
/// > command is successfully received from the
/// > client. The event data is 4-byte and represents
/// > the stream ID of the stream that became
/// > functional.
pub struct EventMessageStreamBegin {
    /// The stream ID of the stream that became functional.
    pub stream_id: u32,
}
