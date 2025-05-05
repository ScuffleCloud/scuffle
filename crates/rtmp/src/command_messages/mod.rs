//! Command messages.

use netconnection::NetConnectionCommand;
use netstream::NetStreamCommand;
use on_status::OnStatus;
use scuffle_amf0::Amf0Value;
use scuffle_bytes_util::StringCow;
use serde_derive::Serialize;

pub mod error;
pub mod netconnection;
pub mod netstream;
pub mod on_status;
pub mod reader;
pub mod writer;

/// Command message.
///
/// > The client and the server exchange commands which are AMF encoded.
/// > The sender sends a command message that consists of command name,
/// > transaction ID, and command object that contains related parameters.
///
/// Defined by:
/// - Legacy RTMP spec, section 7.1.1
/// - Legacy RTMP spec, section 7.2
#[derive(Debug, Clone)]
pub struct Command<'a> {
    /// Transaction ID.
    ///
    /// > The receiver processes the command and sends back the response with the
    /// > same transaction ID.
    pub transaction_id: f64,
    /// Command type.
    pub command_type: CommandType<'a>,
}

/// This enum wraps the [`NetConnectionCommand`], [`NetStreamCommand`] and [`OnStatus`] enums.
#[derive(Debug, Clone)]
pub enum CommandType<'a> {
    /// NetConnection command
    NetConnection(NetConnectionCommand<'a>),
    /// NetStream command
    NetStream(NetStreamCommand<'a>),
    /// onStatus command
    OnStatus(OnStatus<'a>),
    /// Any unknown command
    ///
    /// e.g. FFmpeg sends some commands that don't appear in any spec, so we need to handle them.
    Unknown(UnknownCommand<'a>),
}

/// Any unknown command
///
/// e.g. FFmpeg sends some commands that don't appear in any spec, so we need to handle them.
#[derive(Debug, Clone)]
pub struct UnknownCommand<'a> {
    /// Name of the unknown command.
    pub command_name: StringCow<'a>,
    /// All other values of the command including the command object.
    pub values: Vec<Amf0Value<'static>>,
}

/// NetStream onStatus level (7.2.2.) and NetConnection connect result level (7.2.1.1.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandResultLevel {
    /// Warning level.
    ///
    /// Not further explained in any spec.
    Warning,
    /// Status level.
    ///
    /// Used by [`OnStatus`] commands.
    Status,
    /// Error level.
    ///
    /// Not further explained in any spec.
    Error,
    /// Any other level.
    #[serde(untagged)]
    Unknown(String),
}
