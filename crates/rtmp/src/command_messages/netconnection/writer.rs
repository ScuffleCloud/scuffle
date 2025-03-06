use std::io;

use scuffle_amf0::{Amf0Encoder, Amf0Value};

use super::NetConnectionCommand;
use crate::command_messages::CommandError;

impl NetConnectionCommand<'_> {
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        match self {
            Self::ConnectResult {
                fmsver,
                capabilities,
                level,
                code,
                description,
                encoding,
            } => {
                Amf0Encoder::encode_string(buf, "_result")?;
                Amf0Encoder::encode_number(buf, transaction_id)?;
                Amf0Encoder::encode_object(
                    buf,
                    &[
                        ("fmsVer".into(), Amf0Value::String(fmsver)),
                        ("capabilities".into(), Amf0Value::Number(capabilities)),
                    ],
                )?;
                Amf0Encoder::encode_object(
                    buf,
                    &[
                        ("level".into(), Amf0Value::String(level.to_str().into())),
                        ("code".into(), Amf0Value::String(code)),
                        ("description".into(), Amf0Value::String(description)),
                        ("objectEncoding".into(), Amf0Value::Number(encoding)),
                    ],
                )?;
            }
            Self::CreateStreamResult { stream_id } => {
                Amf0Encoder::encode_string(buf, "_result")?;
                Amf0Encoder::encode_number(buf, transaction_id)?;
                Amf0Encoder::encode_null(buf)?;
                Amf0Encoder::encode_number(buf, stream_id)?;
            }
            _ => unimplemented!("the rtmp client is not implemented yet"),
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::borrow::Cow;

    use bytes::{BufMut, BytesMut};
    use scuffle_amf0::Amf0Decoder;

    use super::*;
    use crate::command_messages::define::CommandResultLevel;

    #[test]
    fn test_netconnection_connect_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::ConnectResult {
            fmsver: "flashver".into(),
            capabilities: 31.0,
            level: CommandResultLevel::Status,
            code: "idk".into(),
            description: "description".into(),
            encoding: 0.0,
        }
        .write(&mut (&mut buf).writer(), 1.0)
        .expect("write");

        let mut amf0_reader = Amf0Decoder::new(&buf);
        let values = amf0_reader.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(
            values[2],
            Amf0Value::Object(Cow::Owned(vec![
                ("fmsVer".into(), Amf0Value::String("flashver".into())),
                ("capabilities".into(), Amf0Value::Number(31.0)),
            ]))
        ); // command object
        assert_eq!(
            values[3],
            Amf0Value::Object(Cow::Owned(vec![
                ("level".into(), Amf0Value::String("status".into())),
                ("code".into(), Amf0Value::String("idk".into())),
                ("description".into(), Amf0Value::String("description".into())),
                ("objectEncoding".into(), Amf0Value::Number(0.0)),
            ]))
        ); // info object
    }

    #[test]
    fn test_netconnection_create_stream_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::CreateStreamResult { stream_id: 1.0 }
            .write(&mut (&mut buf).writer(), 1.0)
            .expect("write");

        let mut amf0_reader = Amf0Decoder::new(&buf);
        let values = amf0_reader.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(values[2], Amf0Value::Null); // command object
        assert_eq!(values[3], Amf0Value::Number(1.0)); // stream id
    }
}
