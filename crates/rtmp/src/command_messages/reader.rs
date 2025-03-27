//! Reading [`Command`].

use std::convert::Infallible;
use std::io;
use std::str::FromStr;

use bytes::Bytes;
use serde::Deserialize;

use super::error::CommandError;
use super::netconnection::NetConnectionCommand;
use super::netstream::NetStreamCommand;
use super::{Command, CommandResultLevel, CommandType, UnknownCommand};

impl Command {
    /// Reads a [`Command`] from the given payload.
    pub fn read(payload: &Bytes) -> Result<Self, CommandError> {
        let mut deserializer = scuffle_amf0::Deserializer::new(io::Cursor::new(payload));

        let command_name = String::deserialize(&mut deserializer)?;
        let transaction_id = f64::deserialize(&mut deserializer)?;

        let command_type = CommandType::read(command_name, &mut deserializer)?;

        Ok(Self {
            transaction_id,
            command_type,
        })
    }
}

impl CommandType {
    fn read<R>(command_name: String, deserializer: &mut scuffle_amf0::Deserializer<R>) -> Result<Self, CommandError>
    where
        R: io::Read + io::Seek + bytes::Buf,
    {
        if let Some(command) = NetConnectionCommand::read(&command_name, deserializer)? {
            return Ok(Self::NetConnection(command));
        }

        if let Some(command) = NetStreamCommand::read(&command_name, deserializer)? {
            return Ok(Self::NetStream(command));
        }

        let values = deserializer.deserialize_all()?;
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
