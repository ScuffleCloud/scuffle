use std::io;

use scuffle_amf0::{Amf0Encoder, Amf0Value};

use super::OnStatus;
use crate::command_messages::CommandError;

impl OnStatus<'_> {
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        Amf0Encoder::encode_string(buf, "onStatus")?;
        Amf0Encoder::encode_number(buf, transaction_id)?;
        // command object
        Amf0Encoder::encode_null(buf)?;

        let mut info_object = vec![
            ("level".into(), Amf0Value::String(self.level.into_string().into())),
            ("code".into(), Amf0Value::String(self.code.clone())),
        ];

        if let Some(description) = self.description.as_ref() {
            info_object.push(("description".into(), Amf0Value::String(description.clone())));
        }

        if let Some(others) = self.others.as_ref() {
            info_object.extend_from_slice(others);
        }

        Amf0Encoder::encode_object(buf, &info_object)?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, BytesMut};
    use scuffle_amf0::{Amf0Decoder, Amf0Value};

    use crate::command_messages::CommandResultLevel;
    use crate::command_messages::on_status::OnStatus;

    #[test]
    fn test_write_on_status() {
        let mut buf = BytesMut::new();

        OnStatus {
            level: CommandResultLevel::Status,
            code: "idk".into(),
            description: Some("description".into()),
            others: None,
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
}
