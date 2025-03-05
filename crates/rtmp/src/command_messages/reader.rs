use std::str::FromStr;

use bytes::Bytes;
use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Value};

use super::define::{Command, CommandResultLevel, CommandType};
use super::errors::CommandError;
use super::netconnection::NetConnectionCommand;
use super::netstream::NetStreamCommand;

impl Command {
    pub fn read(payload: &Bytes) -> Result<Command, CommandError> {
        let mut amf_reader = Amf0Decoder::new(payload);
        let Amf0Value::String(command_name) = amf_reader.decode_with_type(Amf0Marker::String)? else {
            unreachable!();
        };
        let Amf0Value::Number(transaction_id) = amf_reader.decode_with_type(Amf0Marker::Number)? else {
            unreachable!();
        };

        let net_command = CommandType::read(&command_name, &mut amf_reader)?;

        Ok(Command {
            transaction_id,
            net_command,
        })
    }
}

impl CommandType {
    fn read(command_name: &str, decoder: &mut Amf0Decoder) -> Result<Self, CommandError> {
        if let Some(command) = NetConnectionCommand::read(command_name, decoder)? {
            return Ok(Self::NetConnection(command));
        }

        if let Some(command) = NetStreamCommand::read(command_name, decoder)? {
            return Ok(Self::NetStream(command));
        }

        Ok(Self::Unknown {
            command_name: command_name.to_string(),
        })
    }
}

impl FromStr for CommandResultLevel {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "warning" => Ok(Self::Warning),
            "status" => Ok(Self::Status),
            "error" => Ok(Self::Error),
            _ => Err(CommandError::InvalidCommandResultLevel(s.to_string())),
        }
    }
}
