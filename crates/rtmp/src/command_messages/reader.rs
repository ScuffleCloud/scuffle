use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;

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

        let command_type = CommandType::read(command_name, &mut amf_reader)?;

        Ok(Self {
            transaction_id,
            command_type,
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

impl FromStr for CommandResultLevel {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "warning" => Ok(Self::Warning),
            "status" => Ok(Self::Status),
            "error" => Ok(Self::Error),
            _ => Ok(Self::Unknown(s.to_string())),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::CommandResultLevel;

    #[test]
    fn test_command_result_level() {
        assert_eq!("warning".parse::<CommandResultLevel>().unwrap(), CommandResultLevel::Warning);
        assert_eq!("status".parse::<CommandResultLevel>().unwrap(), CommandResultLevel::Status);
        assert_eq!("error".parse::<CommandResultLevel>().unwrap(), CommandResultLevel::Error);
        assert_eq!(
            "unknown".parse::<CommandResultLevel>().unwrap(),
            CommandResultLevel::Unknown("unknown".to_string())
        );
    }
}
