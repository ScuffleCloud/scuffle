use std::io;

use scuffle_amf0::{Amf0Encoder, Amf0Value};

use super::NetStreamCommand;
use crate::command_messages::CommandError;

impl NetStreamCommand<'_> {
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        match self {
            Self::OnStatus {
                level,
                code,
                description,
            } => {
                Amf0Encoder::encode_string(buf, "onStatus")?;
                Amf0Encoder::encode_number(buf, transaction_id)?;
                Amf0Encoder::encode_null(buf)?;
                Amf0Encoder::encode_object(
                    buf,
                    &[
                        ("level".into(), Amf0Value::String(level.to_str().into())),
                        ("code".into(), Amf0Value::String(code)),
                        ("description".into(), Amf0Value::String(description)),
                    ],
                )?;
            }
            Self::Play
            | Self::Play2
            | Self::DeleteStream { .. }
            | Self::CloseStream
            | Self::ReceiveAudio
            | Self::ReceiveVideo
            | Self::Publish { .. }
            | Self::Seek
            | Self::Pause => return Err(CommandError::NoClientImplementation),
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, BytesMut};
    use scuffle_amf0::Amf0Decoder;

    use super::*;
    use crate::command_messages::define::CommandResultLevel;

    #[test]
    fn test_netstream_write_on_status() {
        let mut buf = BytesMut::new();

        NetStreamCommand::OnStatus {
            level: CommandResultLevel::Status,
            code: "idk".into(),
            description: "description".into(),
        }
        .write(&mut (&mut buf).writer(), 1.0)
        .expect("write");

        let mut amf0_reader = Amf0Decoder::new(&buf);
        let values = amf0_reader.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("onStatus".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(values[2], Amf0Value::Null); // command object
        assert_eq!(
            values[3],
            Amf0Value::Object(
                vec![
                    ("level".into(), Amf0Value::String("status".into())),
                    ("code".into(), Amf0Value::String("idk".into())),
                    ("description".into(), Amf0Value::String("description".into())),
                ]
                .into()
            )
        ); // info object
    }

    #[test]
    fn test_not_implemented() {
        let err = NetStreamCommand::Play
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::Play2
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::DeleteStream { stream_id: 1.0 }
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::CloseStream
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::ReceiveAudio
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::ReceiveVideo
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::Publish {
            publishing_name: "name".into(),
            publishing_type: "type".into(),
        }
        .write(&mut (&mut BytesMut::new()).writer(), 1.0)
        .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::Seek
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));

        let err = NetStreamCommand::Pause
            .write(&mut (&mut BytesMut::new()).writer(), 1.0)
            .unwrap_err();
        assert!(matches!(err, CommandError::NoClientImplementation));
    }
}
