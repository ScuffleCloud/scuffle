//! Reading [`Command`].

use std::convert::Infallible;
use std::str::FromStr;

use bytes::Bytes;
use scuffle_amf0::decoder::Amf0Decoder;
use scuffle_bytes_util::StringCow;
use scuffle_bytes_util::zero_copy::BytesBuf;

use super::error::CommandError;
use super::netconnection::NetConnectionCommand;
use super::netstream::NetStreamCommand;
use super::{Command, CommandResultLevel, CommandType, UnknownCommand};

impl Command<'_> {
    /// Reads a [`Command`] from the given payload.
    pub fn read(payload: Bytes) -> Result<Self, CommandError> {
        let mut decoder = Amf0Decoder::from_buf(payload);

        let command_name = decoder.decode_string()?;
        let transaction_id = decoder.decode_number()?;

        let command_type = CommandType::read(command_name, &mut decoder)?;

        Ok(Self {
            transaction_id,
            command_type,
        })
    }
}

impl<'a> CommandType<'a> {
    fn read(command_name: StringCow<'a>, decoder: &mut Amf0Decoder<BytesBuf<Bytes>>) -> Result<Self, CommandError> {
        if let Some(command) = NetConnectionCommand::read(command_name.as_str(), decoder)? {
            return Ok(Self::NetConnection(command));
        }

        if let Some(command) = NetStreamCommand::read(command_name.as_str(), decoder)? {
            return Ok(Self::NetStream(command));
        }

        let values = decoder.decode_all()?;
        Ok(Self::Unknown(UnknownCommand { command_name, values }))
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
