use std::convert::Infallible;
use std::str::FromStr;

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
                let publishing_type = NetStreamCommandPublishPublishingType::from_str(&publishing_type).unwrap();

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
            _ => Ok(None),
        }
    }
}

impl FromStr for NetStreamCommandPublishPublishingType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "live" => Ok(Self::Live),
            "record" => Ok(Self::Record),
            "append" => Ok(Self::Append),
            _ => Ok(Self::Unknown(s.to_string())),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::str::FromStr;

    use scuffle_amf0::{Amf0Decoder, Amf0Encoder, Amf0Marker};

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
    fn test_parse_publishing_type() {
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("live").unwrap(),
            NetStreamCommandPublishPublishingType::Live
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("record").unwrap(),
            NetStreamCommandPublishPublishingType::Record
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("append").unwrap(),
            NetStreamCommandPublishPublishingType::Append
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("unknown").unwrap(),
            NetStreamCommandPublishPublishingType::Unknown("unknown".to_string())
        );
    }
}
