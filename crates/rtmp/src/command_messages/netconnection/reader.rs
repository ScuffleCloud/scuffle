//! Reading [`NetConnectionCommand`].

use bytes::Bytes;
use scuffle_amf0::decoder::Amf0Decoder;
use scuffle_bytes_util::zero_copy::BytesBuf;

use super::NetConnectionCommand;
use crate::command_messages::error::CommandError;

impl NetConnectionCommand<'_> {
    /// Reads a [`NetConnectionCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder<BytesBuf<Bytes>>) -> Result<Option<Self>, CommandError> {
        match command_name {
            "connect" => {
                let command_object = decoder.deserialize()?;
                Ok(Some(Self::Connect(command_object)))
            }
            "call" => Ok(Some(Self::Call {
                command_object: decoder.deserialize()?,
                optional_arguments: decoder.deserialize()?,
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
    use bytes::Bytes;
    use scuffle_amf0::Amf0Object;
    use scuffle_amf0::decoder::Amf0Decoder;
    use scuffle_amf0::encoder::Amf0Encoder;

    use super::NetConnectionCommand;
    use crate::command_messages::error::CommandError;

    #[test]
    fn test_read_no_app() {
        let mut command_object = Vec::new();
        let mut encoder = Amf0Encoder::new(&mut command_object);
        encoder.encode_object(&Amf0Object::new()).unwrap();

        let mut decoder = Amf0Decoder::from_buf(Bytes::from_owner(command_object));
        let result = NetConnectionCommand::read("connect", &mut decoder).unwrap_err();

        assert!(matches!(result, CommandError::Amf0(scuffle_amf0::Amf0Error::Custom(_))));
    }
}
