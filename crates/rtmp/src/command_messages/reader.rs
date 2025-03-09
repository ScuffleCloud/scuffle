use std::borrow::Cow;

use bytes::Bytes;
use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Value};

use super::define::{Command, CommandResultLevel, CommandType};
use super::error::CommandError;
use super::netconnection::NetConnectionCommand;
use super::netstream::NetStreamCommand;

impl<'a> Command<'a> {
    pub fn read(payload: &'a Bytes) -> Result<Self, CommandError> {
        let mut amf_reader = Amf0Decoder::new(payload);

        let Amf0Value::String(command_name) = amf_reader.decode_with_type(Amf0Marker::String)? else {
            unreachable!();
        };
        let Amf0Value::Number(transaction_id) = amf_reader.decode_with_type(Amf0Marker::Number)? else {
            unreachable!();
        };

        let net_command = CommandType::read(command_name, &mut amf_reader)?;

        Ok(Self {
            transaction_id,
            net_command,
        })
    }
}

impl<'a> CommandType<'a> {
    fn read(command_name: Cow<'a, str>, decoder: &mut Amf0Decoder<'a>) -> Result<Self, CommandError> {
        if let Some(command) = NetConnectionCommand::read(&command_name, decoder)? {
            return Ok(Self::NetConnection(command));
        }

        if let Some(command) = NetStreamCommand::read(&command_name, decoder)? {
            return Ok(Self::NetStream(command));
        }

        Ok(Self::Unknown { command_name })
    }
}

impl From<&str> for CommandResultLevel {
    fn from(s: &str) -> Self {
        match s {
            "warning" => Self::Warning,
            "status" => Self::Status,
            "error" => Self::Error,
            _ => Self::Unknown(s.to_string()),
        }
    }
}
