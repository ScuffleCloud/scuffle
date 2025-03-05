use super::netconnection::NetConnectionCommand;
use super::netstream::NetStreamCommand;

#[derive(Debug, Clone)]
pub struct Command {
    pub transaction_id: f64,
    pub net_command: CommandType,
}

/// This enum wraps the [`NetConnectionCommand`] and [`NetStreamCommand`] enums.
#[derive(Debug, Clone)]
pub enum CommandType {
    /// NetConnection command
    NetConnection(NetConnectionCommand),
    /// NetStream command
    NetStream(NetStreamCommand),
    /// Any unknown command
    ///
    /// e.g. FFmpeg sends some commands that don't appear in any spec, so we need to handle them.
    Unknown { command_name: String },
}

/// NetStream onStatus level (7.2.2.) and NetConnection connect result level (7.2.1.1.)
#[derive(Debug, Clone)]
pub enum CommandResultLevel {
    Warning,
    Status,
    Error,
}
