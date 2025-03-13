use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Value};

use super::NetStreamCommand;
use super::define::NetStreamCommandPublishPublishingType;
use crate::command_messages::error::CommandError;

impl<'a> NetStreamCommand<'a> {
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder<'a>) -> Result<Option<Self>, CommandError> {
        match command_name {
            "play" => Ok(Some(Self::Play)),
            "play2" => Ok(Some(Self::Play2)),
            "deleteStream" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let Amf0Value::Number(stream_id) = decoder.decode_with_type(Amf0Marker::Number)? else {
                    unreachable!();
                };

                Ok(Some(Self::DeleteStream { stream_id }))
            }
            "closeStream" => Ok(Some(Self::CloseStream)),
            "receiveAudio" => Ok(Some(Self::ReceiveAudio)),
            "receiveVideo" => Ok(Some(Self::ReceiveVideo)),
            "publish" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let Amf0Value::String(publishing_name) = decoder.decode_with_type(Amf0Marker::String)? else {
                    unreachable!();
                };
                let Amf0Value::String(publishing_type) = decoder.decode_with_type(Amf0Marker::String)? else {
                    unreachable!();
                };
                let publishing_type = From::<&str>::from(&publishing_type);

                if let NetStreamCommandPublishPublishingType::Unknown(publishing_type) = &publishing_type {
                    tracing::warn!(publishing_type = ?publishing_type, "unknown publishing type in publish command");
                }

                Ok(Some(Self::Publish {
                    publishing_name,
                    publishing_type,
                }))
            }
            "seek" => Ok(Some(Self::Seek)),
            "pause" => Ok(Some(Self::Pause)),
            "onStatus" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let Amf0Value::Object(info_object) = decoder.decode_with_type(Amf0Marker::Object)? else {
                    unreachable!();
                };
                // we have to get ownership here because we have to own the inner Cows
                let mut info_object = info_object.into_owned();

                let (_, Amf0Value::String(level)) = info_object
                    .iter()
                    .find(|(k, _)| k == "level")
                    .ok_or(CommandError::InvalidOnStatusInfoObject)?
                else {
                    return Err(CommandError::InvalidOnStatusInfoObject);
                };

                let level = From::<&str>::from(level);

                let (_, Amf0Value::String(code)) = info_object.remove(
                    info_object
                        .iter()
                        .position(|(k, _)| k == "code")
                        .ok_or(CommandError::InvalidOnStatusInfoObject)?,
                ) else {
                    return Err(CommandError::InvalidOnStatusInfoObject);
                };

                let (_, Amf0Value::String(description)) = info_object.remove(
                    info_object
                        .iter()
                        .position(|(k, _)| k == "description")
                        .ok_or(CommandError::InvalidOnStatusInfoObject)?,
                ) else {
                    return Err(CommandError::InvalidOnStatusInfoObject);
                };

                Ok(Some(Self::OnStatus {
                    level,
                    code,
                    description,
                }))
            }
            _ => Ok(None),
        }
    }
}

impl From<&str> for NetStreamCommandPublishPublishingType {
    fn from(s: &str) -> Self {
        match s {
            "live" => Self::Live,
            "record" => Self::Record,
            "append" => Self::Append,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_amf0::{Amf0Decoder, Amf0Encoder, Amf0Marker, Amf0Value};

    use super::NetStreamCommandPublishPublishingType;
    use crate::command_messages::netstream::NetStreamCommand;

    #[test]
    fn test_command_no_payload() {
        let command = NetStreamCommand::read("play", &mut Amf0Decoder::new(&[])).unwrap().unwrap();
        assert_eq!(command, NetStreamCommand::Play);

        let command = NetStreamCommand::read("play2", &mut Amf0Decoder::new(&[])).unwrap().unwrap();
        assert_eq!(command, NetStreamCommand::Play2);

        let command = NetStreamCommand::read("closeStream", &mut Amf0Decoder::new(&[]))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::CloseStream);

        let command = NetStreamCommand::read("receiveAudio", &mut Amf0Decoder::new(&[]))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::ReceiveAudio);

        let command = NetStreamCommand::read("receiveVideo", &mut Amf0Decoder::new(&[]))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::ReceiveVideo);

        let command = NetStreamCommand::read("seek", &mut Amf0Decoder::new(&[])).unwrap().unwrap();
        assert_eq!(command, NetStreamCommand::Seek);

        let command = NetStreamCommand::read("pause", &mut Amf0Decoder::new(&[])).unwrap().unwrap();
        assert_eq!(command, NetStreamCommand::Pause);
    }

    #[test]
    fn test_command_delete_stream() {
        let mut payload = vec![Amf0Marker::Null as u8, Amf0Marker::Number as u8];
        payload.extend_from_slice(0.0f64.to_be_bytes().as_ref());

        let command = NetStreamCommand::read("deleteStream", &mut Amf0Decoder::new(&payload))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::DeleteStream { stream_id: 0.0 });
    }

    #[test]
    fn test_command_publish() {
        let mut payload = Vec::new();

        Amf0Encoder::encode_null(&mut payload).unwrap();
        Amf0Encoder::encode_string(&mut payload, "live").unwrap();
        Amf0Encoder::encode_string(&mut payload, "record").unwrap();

        let command = NetStreamCommand::read("publish", &mut Amf0Decoder::new(&payload))
            .unwrap()
            .unwrap();

        assert_eq!(
            command,
            NetStreamCommand::Publish {
                publishing_name: "live".into(),
                publishing_type: NetStreamCommandPublishPublishingType::Record
            }
        );
    }

    #[test]
    fn test_command_on_status() {
        let mut payload = Vec::new();

        Amf0Encoder::encode_null(&mut payload).unwrap();
        Amf0Encoder::encode_object(
            &mut payload,
            &[
                ("level".into(), Amf0Value::String("error".into())),
                ("code".into(), Amf0Value::String("NetStream.Play.StreamNotFound".into())),
                ("description".into(), Amf0Value::String("Stream not found".into())),
            ],
        )
        .unwrap();

        let command = NetStreamCommand::read("onStatus", &mut Amf0Decoder::new(&payload))
            .unwrap()
            .unwrap();

        assert_eq!(
            command,
            NetStreamCommand::OnStatus {
                level: "error".into(),
                code: "NetStream.Play.StreamNotFound".into(),
                description: "Stream not found".into()
            }
        );
    }

    #[test]
    fn test_parse_publishing_type() {
        assert_eq!(
            NetStreamCommandPublishPublishingType::from("live"),
            NetStreamCommandPublishPublishingType::Live
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from("record"),
            NetStreamCommandPublishPublishingType::Record
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from("append"),
            NetStreamCommandPublishPublishingType::Append
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from("unknown"),
            NetStreamCommandPublishPublishingType::Unknown("unknown".to_string())
        );
    }
}
