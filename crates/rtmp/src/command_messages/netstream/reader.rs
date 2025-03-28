//! Reading [`NetStreamCommand`].

use std::io;

use scuffle_amf0::Amf0Object;
use serde::Deserialize;

use super::{NetStreamCommand, NetStreamCommandPublishPublishingType};
use crate::command_messages::error::CommandError;

impl NetStreamCommand {
    /// Reads a [`NetStreamCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read<R>(
        command_name: &str,
        deserializer: &mut scuffle_amf0::Deserializer<R>,
    ) -> Result<Option<Self>, CommandError>
    where
        R: io::Read + io::Seek + bytes::Buf,
    {
        match command_name {
            "play" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let values = deserializer.deserialize_all()?;
                Ok(Some(Self::Play { values }))
            }
            "play2" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let parameters = Amf0Object::deserialize(deserializer)?;
                Ok(Some(Self::Play2 { parameters }))
            }
            "deleteStream" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let stream_id = f64::deserialize(deserializer)?;
                Ok(Some(Self::DeleteStream { stream_id }))
            }
            "closeStream" => Ok(Some(Self::CloseStream)),
            "receiveAudio" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let receive_audio = bool::deserialize(deserializer)?;
                Ok(Some(Self::ReceiveAudio { receive_audio }))
            }
            "receiveVideo" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let receive_video = bool::deserialize(deserializer)?;
                Ok(Some(Self::ReceiveVideo { receive_video }))
            }
            "publish" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let publishing_name = String::deserialize(&mut *deserializer)?;
                let publishing_type = NetStreamCommandPublishPublishingType::deserialize(deserializer)?;

                Ok(Some(Self::Publish {
                    publishing_name,
                    publishing_type,
                }))
            }
            "seek" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let milliseconds = f64::deserialize(deserializer)?;
                Ok(Some(Self::Seek { milliseconds }))
            }
            "pause" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let pause = bool::deserialize(&mut *deserializer)?;
                let milliseconds = f64::deserialize(deserializer)?;
                Ok(Some(Self::Pause { pause, milliseconds }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use scuffle_amf0::Amf0Marker;
    use serde::Serialize;

    use super::NetStreamCommandPublishPublishingType;
    use crate::command_messages::netstream::NetStreamCommand;

    #[test]
    fn test_command_no_payload() {
        let command = NetStreamCommand::read("closeStream", &mut scuffle_amf0::Deserializer::new(io::Cursor::new(&[])))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::CloseStream);
    }

    #[test]
    fn test_command_delete_stream() {
        let mut payload = vec![Amf0Marker::Null as u8, Amf0Marker::Number as u8];
        payload.extend_from_slice(0.0f64.to_be_bytes().as_ref());

        let command = NetStreamCommand::read(
            "deleteStream",
            &mut scuffle_amf0::Deserializer::new(io::Cursor::new(&payload)),
        )
        .unwrap()
        .unwrap();
        assert_eq!(command, NetStreamCommand::DeleteStream { stream_id: 0.0 });
    }

    #[test]
    fn test_command_publish() {
        let mut payload = Vec::new();

        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        "live".serialize(&mut serializer).unwrap();
        NetStreamCommandPublishPublishingType::Record
            .serialize(&mut serializer)
            .unwrap();

        let command = NetStreamCommand::read("publish", &mut scuffle_amf0::Deserializer::new(io::Cursor::new(&payload)))
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
}
