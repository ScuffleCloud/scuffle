use std::borrow::Cow;

use scuffle_amf0::Amf0Object;

use crate::command_messages::CommandResultLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct OnStatus<'a> {
    pub code: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub level: CommandResultLevel,
    pub others: Option<Amf0Object<'a>>,
}

// We can't use a nutype enum here because it would have to wrap a Cow<'a, str>.
#[allow(unused)]
pub mod codes {
    pub const NET_CONNECTION_CALL_FAILED: &str = "NetConnection.Call.Failed";
    pub const NET_CONNECTION_CONNECT_APP_SHUTDOWN: &str = "NetConnection.Connect.AppShutdown";
    pub const NET_CONNECTION_CONNECT_CLOSED: &str = "NetConnection.Connect.Closed";
    pub const NET_CONNECTION_CONNECT_FAILED: &str = "NetConnection.Connect.Failed";
    pub const NET_CONNECTION_CONNECT_REJECTED: &str = "NetConnection.Connect.Rejected";
    pub const NET_CONNECTION_CONNECT_SUCCESS: &str = "NetConnection.Connect.Success";
    pub const NET_CONNECTION_CONNECT_RECONNECT_REQUEST: &str = "NetConnection.Connect.ReconnectRequest";
    pub const NET_CONNECTION_PROXY_NOT_RESPONDING: &str = "NetConnection.Proxy.NotResponding";

    pub const NET_STREAM_PUBLISH_START: &str = "NetStream.Publish.Start";
    pub const NET_STREAM_DELETE_STREAM_SUCCESS: &str = "NetStream.DeleteStream.Suceess";
}
