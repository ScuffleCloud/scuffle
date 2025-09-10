//! NetConnection command messages.

use std::collections::HashMap;

use scuffle_amf0::{Amf0Object, Amf0Value};
use scuffle_bytes_util::StringCow;
use serde_derive::{Deserialize, Serialize};

use super::on_status::{OnStatus, OnStatusCode};
use crate::command_messages::CommandResultLevel;

pub mod reader;
pub mod writer;

/// NetConnection command `connect`.
///
/// Defined by:
/// - Legacy RTMP spec, 7.2.1.1
/// - Enhanced RTMP spec, page 36-37, Enhancing NetConnection connect Command
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(bound = "'a: 'de")]
pub struct NetConnectionCommandConnect<'a> {
    /// Tells the server application name the client is connected to.
    pub app: StringCow<'a>,
    /// represents capability flags which can be combined via a
    /// Bitwise OR to indicate which extended set of capabilities (i.e.,
    /// beyond the legacy RTMP specification) are supported via E-RTMP.
    /// See enum [`CapsExMask`] for the enumerated values representing the
    /// assigned bits.
    #[serde(default)]
    pub caps_ex: Option<CapsExMask>,
    /// All other parameters.
    ///
    /// Defined by:
    /// - Legacy RTMP spec, page 30
    /// - Enhanced RTMP spec, page 36-37
    #[serde(flatten, borrow)]
    pub others: HashMap<StringCow<'a>, Amf0Value<'a>>,
}

/// Extended capabilities mask used by the [enhanced connect command](NetConnectionCommandConnect).
#[derive(Deserialize)]
#[serde(from = "u8", into = "u8")]
#[bitmask_enum::bitmask(u8)]
pub enum CapsExMask {
    /// Support for reconnection
    Reconnect = 0x01,
    /// Support for multitrack
    Multitrack = 0x02,
    /// Can parse ModEx signal
    ModEx = 0x04,
    /// Support for nano offset
    TimestampNanoOffset = 0x08,
}

/// NetConnection command `connect` result.
///
/// Defined by:
/// - Legacy RTMP spec, 7.2.1.1
#[derive(Debug, Clone, PartialEq)]
pub struct NetConnectionCommandConnectResult<'a> {
    /// Properties of the connection.
    pub properties: NetConnectionCommandConnectResultProperties<'a>,
    /// Information about the connection.
    pub information: OnStatus<'a>,
}

/// NetConnection command `connect` result properties.
///
/// > Name-value pairs that describe the properties(fmsver etc.) of the connection.
///
/// Defined by:
/// - Legacy RTMP spec, 7.2.1.1
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetConnectionCommandConnectResultProperties<'a> {
    /// Flash Media Server version.
    ///
    /// Usually set to "FMS/3,0,1,123".
    pub fms_ver: StringCow<'a>,
    /// No idea what this means, but it is used by other media servers as well.
    ///
    /// Usually set to 31.0.
    pub capabilities: f64,
}

impl Default for NetConnectionCommandConnectResultProperties<'static> {
    fn default() -> Self {
        Self {
            fms_ver: "FMS/3,0,1,123".into(),
            capabilities: 31.0,
        }
    }
}

impl Default for NetConnectionCommandConnectResult<'_> {
    fn default() -> Self {
        Self {
            information: OnStatus {
                level: CommandResultLevel::Status,
                code: OnStatusCode::NET_CONNECTION_CONNECT_SUCCESS,
                description: Some("Connection Succeeded.".into()),
                others: Some([("objectEncoding".into(), Amf0Value::Number(0.0))].into_iter().collect()),
            },
            properties: Default::default(),
        }
    }
}

/// NetConnection commands as defined in 7.2.1.
#[derive(Debug, Clone, PartialEq)]
pub enum NetConnectionCommand<'a> {
    /// Connect command.
    Connect(NetConnectionCommandConnect<'a>),
    /// Connect result.
    ///
    /// Sent from server to client in response to [`NetConnectionCommand::Connect`].
    ConnectResult(NetConnectionCommandConnectResult<'a>),
    /// Call command.
    Call {
        /// The command object.
        command_object: Option<Amf0Object<'a>>,
        /// Any optional arguments.
        optional_arguments: Option<Amf0Object<'a>>,
    },
    /// Close command.
    Close,
    /// Create stream command.
    CreateStream,
    /// Create stream result.
    ///
    /// Sent from server to client in response to [`NetConnectionCommand::CreateStream`].
    CreateStreamResult {
        /// ID of the created stream.
        stream_id: f64,
    },
}
