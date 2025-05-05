//! Writing [`NetConnectionCommand`].

use std::io;

use scuffle_amf0::encoder::Amf0Encoder;

use super::{NetConnectionCommand, NetConnectionCommandConnectResult};
use crate::command_messages::error::CommandError;

impl NetConnectionCommand<'_> {
    /// Writes a [`NetConnectionCommand`] to the given writer.
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        let mut encoder = Amf0Encoder::new(buf);

        match self {
            Self::ConnectResult(NetConnectionCommandConnectResult { properties, information }) => {
                encoder.encode_string("_result")?;
                encoder.encode_number(transaction_id)?;
                encoder.serialize(&properties)?;
                encoder.serialize(&information)?;
            }
            Self::CreateStreamResult { stream_id } => {
                encoder.encode_string("_result")?;
                encoder.encode_number(transaction_id)?;
                encoder.encode_null()?;
                encoder.encode_number(stream_id)?;
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
    use scuffle_amf0::Amf0Value;
    use scuffle_amf0::decoder::Amf0Decoder;

    use super::*;

    #[test]
    fn test_netconnection_connect_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::ConnectResult(NetConnectionCommandConnectResult::default())
            .write(&mut (&mut buf).writer(), 1.0)
            .expect("write");

        let mut deserializer = Amf0Decoder::from_buf(buf.freeze());
        let values = deserializer.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(
            values[2],
            Amf0Value::Object(
                [
                    ("fmsVer".into(), Amf0Value::String("FMS/3,0,1,123".into())),
                    ("capabilities".into(), Amf0Value::Number(31.0)),
                ]
                .into_iter()
                .collect()
            )
        );
        assert_eq!(
            values[3],
            Amf0Value::Object(
                [
                    ("level".into(), Amf0Value::String("status".into())),
                    ("code".into(), Amf0Value::String("NetConnection.Connect.Success".into())),
                    ("description".into(), Amf0Value::String("Connection Succeeded.".into())),
                    ("objectEncoding".into(), Amf0Value::Number(0.0)),
                ]
                .into_iter()
                .collect()
            )
        );
    }

    #[test]
    fn test_netconnection_create_stream_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::CreateStreamResult { stream_id: 1.0 }
            .write(&mut (&mut buf).writer(), 1.0)
            .expect("write");

        let mut deserializer = Amf0Decoder::from_buf(buf.freeze());
        let values = deserializer.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(values[2], Amf0Value::Null); // command object
        assert_eq!(values[3], Amf0Value::Number(1.0)); // stream id
    }
}
