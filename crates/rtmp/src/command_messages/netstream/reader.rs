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
