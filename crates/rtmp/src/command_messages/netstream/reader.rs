//! Reading [`NetStreamCommand`].

use bytes::Bytes;
use scuffle_amf0::decoder::Amf0Decoder;
use scuffle_bytes_util::zero_copy::BytesBuf;

use super::NetStreamCommand;
use crate::command_messages::error::CommandError;

impl NetStreamCommand<'_> {
    /// Reads a [`NetStreamCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder<BytesBuf<Bytes>>) -> Result<Option<Self>, CommandError> {
        match command_name {
            "play" => {
                // skip command object
                decoder.decode_null()?;

                let values = decoder.decode_all()?;
                Ok(Some(Self::Play { values }))
            }
            "play2" => {
                // skip command object
                decoder.decode_null()?;

                let parameters = decoder.decode_object()?;
                Ok(Some(Self::Play2 { parameters }))
            }
            "deleteStream" => {
                // skip command object
                decoder.decode_null()?;

                let stream_id = decoder.decode_number()?;
                Ok(Some(Self::DeleteStream { stream_id }))
            }
            "closeStream" => Ok(Some(Self::CloseStream)),
            "receiveAudio" => {
                // skip command object
                decoder.decode_null()?;

                let receive_audio = decoder.decode_boolean()?;
                Ok(Some(Self::ReceiveAudio { receive_audio }))
            }
            "receiveVideo" => {
                // skip command object
                decoder.decode_null()?;

                let receive_video = decoder.decode_boolean()?;
                Ok(Some(Self::ReceiveVideo { receive_video }))
            }
            "publish" => {
                // skip command object
                decoder.decode_null()?;

                let publishing_name = decoder.decode_string()?;
                let publishing_type = decoder.deserialize()?;

                Ok(Some(Self::Publish {
                    publishing_name,
                    publishing_type,
                }))
            }
            "seek" => {
                // skip command object
                decoder.decode_null()?;

                let milliseconds = decoder.decode_number()?;
                Ok(Some(Self::Seek { milliseconds }))
            }
            "pause" => {
                // skip command object
                decoder.decode_null()?;

                let pause = decoder.decode_boolean()?;
                let milliseconds = decoder.decode_number()?;
                Ok(Some(Self::Pause { pause, milliseconds }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;
    use scuffle_amf0::decoder::Amf0Decoder;
    use scuffle_amf0::encoder::Amf0Encoder;
    use scuffle_amf0::{Amf0Marker, Amf0Object};
    use scuffle_bytes_util::StringCow;

    use crate::command_messages::netstream::{NetStreamCommand, NetStreamCommandPublishPublishingType};

    #[test]
    fn test_command_no_payload() {
        let command = NetStreamCommand::read("closeStream", &mut Amf0Decoder::from_buf(Bytes::new()))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::CloseStream);
    }

    #[test]
    fn play_command() {
        let mut payload = Vec::new();

        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();
        encoder.encode_number(0.0).unwrap();
        encoder.encode_string("test").unwrap();

        let command = NetStreamCommand::read("play", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();

        assert_eq!(
            command,
            NetStreamCommand::Play {
                values: vec![0.0f64.into(), StringCow::from("test").into(),],
            }
        );
    }

    #[test]
    fn play2_command() {
        let mut payload = Vec::new();

        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();

        let object: Amf0Object = [
            ("name".into(), StringCow::from("test").into()),
            ("value".into(), 0.0f64.into()),
        ]
        .into_iter()
        .collect();
        encoder.encode_object(&object).unwrap();

        let command = NetStreamCommand::read("play2", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();

        assert_eq!(command, NetStreamCommand::Play2 { parameters: object });
    }

    #[test]
    fn receive_audio() {
        let mut payload = Vec::new();
        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();
        encoder.encode_boolean(true).unwrap();

        let command = NetStreamCommand::read("receiveAudio", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::ReceiveAudio { receive_audio: true });
    }

    #[test]
    fn receive_video() {
        let mut payload = Vec::new();
        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();
        encoder.encode_boolean(true).unwrap();

        let command = NetStreamCommand::read("receiveVideo", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::ReceiveVideo { receive_video: true });
    }

    #[test]
    fn delete_stream() {
        let mut payload = vec![Amf0Marker::Null as u8, Amf0Marker::Number as u8];
        payload.extend_from_slice(0.0f64.to_be_bytes().as_ref());

        let command = NetStreamCommand::read("deleteStream", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::DeleteStream { stream_id: 0.0 });
    }

    #[test]
    fn publish() {
        let mut payload = Vec::new();

        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();
        encoder.encode_string("live").unwrap();
        encoder.serialize(NetStreamCommandPublishPublishingType::Record).unwrap();

        let command = NetStreamCommand::read("publish", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
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
    fn seek() {
        let mut payload = Vec::new();
        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();
        encoder.encode_number(0.0).unwrap();

        let command = NetStreamCommand::read("seek", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::Seek { milliseconds: 0.0 });
    }

    #[test]
    fn pause() {
        let mut payload = Vec::new();
        let mut encoder = Amf0Encoder::new(&mut payload);
        encoder.encode_null().unwrap();
        encoder.encode_boolean(true).unwrap();
        encoder.encode_number(0.0).unwrap();

        let command = NetStreamCommand::read("pause", &mut Amf0Decoder::from_buf(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(
            command,
            NetStreamCommand::Pause {
                pause: true,
                milliseconds: 0.0
            }
        );
    }
}
