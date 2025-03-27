//! Writing [`NetConnectionCommand`].

use std::io;

use scuffle_amf0::{Amf0Object, Amf0Value};
use serde::Serialize;

use super::{NetConnectionCommand, NetConnectionCommandConnectResult};
use crate::command_messages::error::CommandError;

impl NetConnectionCommand {
    /// Writes a [`NetConnectionCommand`] to the given writer.
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        let mut serializer = scuffle_amf0::Serializer::new(buf);

        match self {
            Self::ConnectResult(NetConnectionCommandConnectResult {
                fmsver,
                capabilities,
                level,
                code,
                description,
                encoding,
            }) => {
                "_result".serialize(&mut serializer)?;
                transaction_id.serialize(&mut serializer)?;
                [
                    ("fmsVer".into(), Amf0Value::String(fmsver)),
                    ("capabilities".into(), Amf0Value::Number(capabilities)),
                ]
                .into_iter()
                .collect::<Amf0Object>()
                .serialize(&mut serializer)?;

                [
                    ("level".into(), Amf0Value::String(level.as_ref().into())),
                    ("code".into(), Amf0Value::String(code.0.to_string())),
                    ("description".into(), Amf0Value::String(description)),
                    ("objectEncoding".into(), Amf0Value::Number(encoding)),
                ]
                .into_iter()
                .collect::<Amf0Object>()
                .serialize(&mut serializer)?;
            }
            Self::CreateStreamResult { stream_id } => {
                "_result".serialize(&mut serializer)?;
                transaction_id.serialize(&mut serializer)?;
                ().serialize(&mut serializer)?;
                stream_id.serialize(&mut serializer)?;
            }
            Self::Connect { .. } | Self::Call { .. } | Self::Close | Self::CreateStream => {
                return Err(CommandError::NoClientImplementation);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;
    use crate::command_messages::CommandResultLevel;

    #[test]
    fn test_netconnection_connect_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::ConnectResult(NetConnectionCommandConnectResult {
            fmsver: "flashver".into(),
            capabilities: 31.0,
            level: CommandResultLevel::Status,
            code: "idk".into(),
            description: "description".into(),
            encoding: 0.0,
        })
        .write(&mut (&mut buf).writer(), 1.0)
        .expect("write");

        let mut deserializer = scuffle_amf0::Deserializer::new(io::Cursor::new(buf));
        let values = deserializer.deserialize_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(
            values[2],
            Amf0Value::Object(
                [
                    ("fmsVer".into(), Amf0Value::String("flashver".into())),
                    ("capabilities".into(), Amf0Value::Number(31.0)),
                ]
                .into_iter()
                .collect()
            )
        ); // command object
        assert_eq!(
            values[3],
            Amf0Value::Object(
                [
                    ("level".into(), Amf0Value::String("status".into())),
                    ("code".into(), Amf0Value::String("idk".into())),
                    ("description".into(), Amf0Value::String("description".into())),
                    ("objectEncoding".into(), Amf0Value::Number(0.0)),
                ]
                .into_iter()
                .collect()
            )
        ); // info object
    }

    #[test]
    fn test_netconnection_create_stream_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::CreateStreamResult { stream_id: 1.0 }
            .write(&mut (&mut buf).writer(), 1.0)
            .expect("write");

        let mut deserializer = scuffle_amf0::Deserializer::new(io::Cursor::new(buf));
        let values = deserializer.deserialize_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(values[2], Amf0Value::Null); // command object
        assert_eq!(values[3], Amf0Value::Number(1.0)); // stream id
    }
}
