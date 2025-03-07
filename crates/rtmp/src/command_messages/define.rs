use std::borrow::Cow;

use super::netconnection::NetConnectionCommand;
use super::netstream::NetStreamCommand;

#[derive(Debug, Clone)]
pub struct Command<'a> {
    pub transaction_id: f64,
    pub net_command: CommandType<'a>,
}

/// This enum wraps the [`NetConnectionCommand`] and [`NetStreamCommand`] enums.
#[derive(Debug, Clone)]
pub enum CommandType<'a> {
    /// NetConnection command
    NetConnection(NetConnectionCommand<'a>),
    /// NetStream command
    NetStream(NetStreamCommand<'a>),
    /// Any unknown command
    ///
    /// e.g. FFmpeg sends some commands that don't appear in any spec, so we need to handle them.
    Unknown { command_name: Cow<'a, str> },
}

/// NetStream onStatus level (7.2.2.) and NetConnection connect result level (7.2.1.1.)
#[derive(Debug, Clone)]
pub enum CommandResultLevel {
    Warning,
    Status,
    Error,
    Unknown(String),
}
