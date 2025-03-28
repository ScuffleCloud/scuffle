//! Reading [`NetConnectionCommand`].

use std::io;

use serde::Deserialize;

use super::{NetConnectionCommand, NetConnectionCommandConnect};
use crate::command_messages::error::CommandError;

impl NetConnectionCommand {
    /// Reads a [`NetConnectionCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read<R>(
        command_name: &str,
        deserializer: &mut scuffle_amf0::Deserializer<R>,
    ) -> Result<Option<Self>, CommandError>
    where
        R: io::Read + io::Seek + bytes::Buf,
    {
        match command_name {
            "connect" => {
                let command_object = NetConnectionCommandConnect::deserialize(deserializer)?;
                Ok(Some(Self::Connect(command_object)))
            }
            "call" => Ok(Some(Self::Call {
                command_object: serde::Deserialize::deserialize(&mut *deserializer)?,
                optional_arguments: serde::Deserialize::deserialize(deserializer)?,
            })),
            "close" => Ok(Some(Self::Close)),
            "createStream" => Ok(Some(Self::CreateStream)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use scuffle_amf0::Amf0Object;
    use serde::Serialize;

    use super::NetConnectionCommand;
    use crate::command_messages::error::CommandError;

    #[test]
    fn test_read_no_app() {
        let mut command_object = Vec::new();
        let mut serializer = scuffle_amf0::Serializer::new(&mut command_object);
        Amf0Object::new().serialize(&mut serializer).unwrap();

        let mut decoder = scuffle_amf0::Deserializer::new(io::Cursor::new(command_object));
        let result = NetConnectionCommand::read("connect", &mut decoder).unwrap_err();

        assert!(matches!(result, CommandError::Amf0(scuffle_amf0::Amf0Error::Custom(_))));
    }
}
