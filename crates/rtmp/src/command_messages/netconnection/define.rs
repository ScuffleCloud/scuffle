use std::borrow::Cow;

use scuffle_amf0::Amf0Object;

use crate::command_messages::define::CommandResultLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct NetConnectionCommandConnect<'a> {
    pub app: Cow<'a, str>,
    pub caps_ex: Option<CapsExMask>,
    /// All other parameters.
    ///
    /// See
    /// - Legacy RTMP spec (rtmp_specification_1.0) (page 30)
    /// - Enhanced RTMP spec (enhanced-rtmp-v2) (page 36,37)
    pub others: Amf0Object<'a>,
}

#[bitmask_enum::bitmask(u8)]
pub enum CapsExMask {
    Reconnect = 0x01,
    Multitrack = 0x02,
    ModEx = 0x04,
    TimestampNanoOffset = 0x08,
}

/// NetConnection commands as defined in 7.2.1.
#[derive(Debug, Clone, PartialEq)]
pub enum NetConnectionCommand<'a> {
    Connect(NetConnectionCommandConnect<'a>),
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
