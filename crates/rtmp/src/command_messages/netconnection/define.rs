use std::borrow::Cow;

use scuffle_amf0::Amf0Object;

use crate::command_messages::define::CommandResultLevel;

/// NetConnection commands as defined in 7.2.1.
#[derive(Debug, Clone, PartialEq)]
pub enum NetConnectionCommand<'a> {
    Connect {
        app: Cow<'a, str>,
        /// All other parameters.
        ///
        /// See
        /// - Legacy RTMP spec (rtmp_specification_1.0) (page 30)
        /// - Enhanced RTMP spec (enhanced-rtmp-v2) (page 36,37)
        others: Amf0Object<'a>,
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
