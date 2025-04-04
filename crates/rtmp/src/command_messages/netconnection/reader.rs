//! Reading [`NetConnectionCommand`].

use scuffle_amf0::{Amf0Decoder, Amf0Value};

use super::{CapsExMask, NetConnectionCommand, NetConnectionCommandConnect};
use crate::command_messages::error::CommandError;

impl<'a> NetConnectionCommand<'a> {
    /// Reads a [`NetConnectionCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder<'a>) -> Result<Option<Self>, CommandError> {
        match command_name {
            "connect" => {
                let Amf0Value::Object(command_object) = decoder.decode_with_type(scuffle_amf0::Amf0Marker::Object)? else {
                    // TODO: CLOUD-91
                    unreachable!();
                };

                let mut command_object = command_object.into_owned();

                let (_, Amf0Value::String(app)) = command_object.remove(
                    command_object
                        .iter()
                        .position(|(k, _)| k == "app")
                        .ok_or(CommandError::NoAppName)?,
                ) else {
                    return Err(CommandError::NoAppName);
                };

                let caps_ex = command_object
                    .iter()
                    .position(|(k, _)| k == "capsEx")
                    .map(|idx| command_object.remove(idx).1);

                let caps_ex = if let Some(caps_ex) = caps_ex {
                    Some(CapsExMask::from(caps_ex.as_number()? as u8))
                } else {
                    None
                };

                Ok(Some(Self::Connect(NetConnectionCommandConnect {
                    app,
                    caps_ex,
                    others: command_object.into(),
                })))
            }
            "call" => {
                let command_object = match decoder.decode()? {
                    Amf0Value::Object(command_object) => Some(command_object),
                    _ => None,
                };

                let optional_arguments = match decoder.decode()? {
                    Amf0Value::Object(optional_arguments) => Some(optional_arguments),
                    _ => None,
                };

                Ok(Some(Self::Call {
                    command_object,
                    optional_arguments,
                }))
            }
            "close" => Ok(Some(Self::Close)),
            "createStream" => Ok(Some(Self::CreateStream)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_amf0::{Amf0Decoder, Amf0Encoder};

    use super::NetConnectionCommand;
    use crate::command_messages::error::CommandError;

    #[test]
    fn test_read_no_app() {
        let mut command_object = Vec::new();
        Amf0Encoder::encode_object(&mut command_object, &[]).unwrap();

        let mut decoder = Amf0Decoder::new(&command_object);
        let result = NetConnectionCommand::read("connect", &mut decoder).unwrap_err();

        assert!(matches!(result, CommandError::NoAppName));
    }
}
