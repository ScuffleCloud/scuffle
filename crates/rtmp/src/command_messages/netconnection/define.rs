use std::borrow::Cow;

use crate::command_messages::define::CommandResultLevel;

/// NetConnection commands as defined in 7.2.1.
#[derive(Debug, Clone)]
pub enum NetConnectionCommand<'a> {
    Connect {
        app: Cow<'a, str>,
    },
    ConnectResult {
        fmsver: Cow<'a, str>,
        capabilities: f64,
        level: CommandResultLevel,
        code: Cow<'a, str>,
        description: Cow<'a, str>,
        encoding: f64,
    },
    Call,
    Close,
    CreateStream,
    CreateStreamResult {
        stream_id: f64,
    },
}
