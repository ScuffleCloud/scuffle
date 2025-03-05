use crate::command_messages::define::CommandResultLevel;

/// NetConnection commands as defined in 7.2.1.
#[derive(Debug, Clone)]
pub enum NetConnectionCommand {
    Connect {
        app: String,
    },
    ConnectResult {
        fmsver: String,
        capabilities: f64,
        level: CommandResultLevel,
        code: String,
        description: String,
        encoding: f64,
    },
    Call,
    Close,
    CreateStream,
    CreateStreamResult {
        stream_id: f64,
    },
}
