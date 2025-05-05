//! Types and functions for processing the `onStatus` command.
//!
//! It is not very clear if the onStatus command should be part of the NetConnection or NetStream set of commands.
//! The legacy RTMP spec makes it look like it should be part of the NetStream commands while the enhanced-rtmp-v2 spec
//! is very clear that it should be part of the NetConnection commands.
//! In reality, it is used as a response message to both NetConnection and NetStream commands received from the client.
//! This is why we have decided to put it in its own module.

use nutype_enum::nutype_enum;
use scuffle_amf0::Amf0Object;
use scuffle_bytes_util::StringCow;
use serde_derive::Serialize;

use crate::command_messages::CommandResultLevel;

pub mod writer;

/// The `onStatus` command is used to send status information from the server to the client.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnStatus<'a> {
    /// The status code.
    ///
    /// Refer to the [`OnStatusCode`] enum for a list of common status codes.
    pub code: OnStatusCode,
    /// The description of the status update.
    pub description: Option<StringCow<'a>>,
    /// The level of the status update.
    pub level: CommandResultLevel,
    /// Any other additional information that should be sent as part of the object.
    #[serde(flatten)]
    pub others: Option<Amf0Object<'a>>,
}

nutype_enum! {
    /// Common status codes used in the `onStatus` command.
    #[derive(Serialize)]
    #[serde(transparent)]
    pub enum OnStatusCode(&'static str) {
        /// The `NetConnection.call()` method was not able to invoke the server-side method or command.
        NET_CONNECTION_CALL_FAILED = "NetConnection.Call.Failed",
        /// The application has been shut down (for example, if the application is out of memory resources
        /// and must shut down to prevent the server from crashing) or the server has shut down.
        NET_CONNECTION_CONNECT_APP_SHUTDOWN = "NetConnection.Connect.AppShutdown",
        /// The connection was closed successfully.
        NET_CONNECTION_CONNECT_CLOSED = "NetConnection.Connect.Closed",
        /// The connection attempt failed.
        NET_CONNECTION_CONNECT_FAILED = "NetConnection.Connect.Failed",
        /// The client does not have permission to connect to the application.
        NET_CONNECTION_CONNECT_REJECTED = "NetConnection.Connect.Rejected",
        /// The connection attempt succeeded.
        NET_CONNECTION_CONNECT_SUCCESS = "NetConnection.Connect.Success",
        /// The server is requesting the client to reconnect.
        NET_CONNECTION_CONNECT_RECONNECT_REQUEST = "NetConnection.Connect.ReconnectRequest",
        /// The proxy server is not responding. See the ProxyStream class.
        NET_CONNECTION_PROXY_NOT_RESPONDING = "NetConnection.Proxy.NotResponding",

        /// Publishing has started.
        NET_STREAM_PUBLISH_START = "NetStream.Publish.Start",
        /// Stream was successfully deleted.
        NET_STREAM_DELETE_STREAM_SUCCESS = "NetStream.DeleteStream.Suceess",
    }
}
