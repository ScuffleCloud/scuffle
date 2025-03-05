use scuffle_amf0::{Amf0Decoder, Amf0Value};

use super::NetConnectionCommand;
use crate::command_messages::errors::CommandError;

impl NetConnectionCommand {
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder) -> Result<Option<Self>, CommandError> {
        match command_name {
            "connect" => {
                let Amf0Value::Object(command_object) = decoder.decode_with_type(scuffle_amf0::Amf0Marker::Object)? else {
                    unreachable!();
                };

                let (_, Amf0Value::String(app)) = command_object
                    .iter()
                    .find(|(key, _)| key == "app")
                    .ok_or(CommandError::NoAppName)?
                else {
                    return Err(CommandError::NoAppName);
                };

                Ok(Some(Self::Connect { app: app.to_string() }))
            }
            "call" => Ok(Some(Self::Call)),
            "close" => Ok(Some(Self::Close)),
            "createStream" => Ok(Some(Self::CreateStream)),
            _ => Ok(None),
        }
    }
}
