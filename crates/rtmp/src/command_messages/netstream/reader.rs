use std::str::FromStr;

use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Value};

use super::NetStreamCommand;
use super::define::NetStreamCommandPublishPublishingType;
use crate::command_messages::define::CommandResultLevel;
use crate::command_messages::errors::CommandError;

impl NetStreamCommand {
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder) -> Result<Option<Self>, CommandError> {
        match command_name {
            "play" => Ok(Some(Self::Play)),
            "play2" => Ok(Some(Self::Play2)),
            "deleteStream" => {
                // skip command object
                let Amf0Value::Null = decoder.decode_with_type(Amf0Marker::Null)? else {
                    unreachable!();
                };

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
                let Amf0Value::Null = decoder.decode_with_type(Amf0Marker::Null)? else {
                    unreachable!();
                };

                let Amf0Value::String(publishing_name) = decoder.decode_with_type(Amf0Marker::String)? else {
                    unreachable!();
                };
                let Amf0Value::String(publishing_type) = decoder.decode_with_type(Amf0Marker::String)? else {
                    unreachable!();
                };
                let publishing_type = NetStreamCommandPublishPublishingType::from_str(&publishing_type)?;

                Ok(Some(Self::Publish {
                    publishing_name: publishing_name.to_string(),
                    publishing_type,
                }))
            }
            "seek" => Ok(Some(Self::Seek)),
            "pause" => Ok(Some(Self::Pause)),
            "onStatus" => {
                // skip command object
                let Amf0Value::Null = decoder.decode_with_type(Amf0Marker::Null)? else {
                    unreachable!();
                };

                let Amf0Value::Object(info_object) = decoder.decode_with_type(Amf0Marker::Object)? else {
                    unreachable!();
                };

                let (_, Amf0Value::String(level)) = info_object
                    .iter()
                    .find(|(k, _)| k == "level")
                    .ok_or(CommandError::InvalidOnStatusInfoObject)?
                else {
                    return Err(CommandError::InvalidOnStatusInfoObject);
                };

                let level = CommandResultLevel::from_str(level)?;

                let (_, Amf0Value::String(code)) = info_object
                    .iter()
                    .find(|(k, _)| k == "code")
                    .ok_or(CommandError::InvalidOnStatusInfoObject)?
                else {
                    return Err(CommandError::InvalidOnStatusInfoObject);
                };

                let (_, Amf0Value::String(description)) = info_object
                    .iter()
                    .find(|(k, _)| k == "description")
                    .ok_or(CommandError::InvalidOnStatusInfoObject)?
                else {
                    return Err(CommandError::InvalidOnStatusInfoObject);
                };

                Ok(Some(Self::OnStatus {
                    level,
                    code: code.to_string(),
                    description: description.to_string(),
                }))
            }
            _ => Ok(None),
        }
    }
}

impl FromStr for NetStreamCommandPublishPublishingType {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "live" => Ok(Self::Live),
            "record" => Ok(Self::Record),
            "append" => Ok(Self::Append),
            _ => Err(CommandError::InvalidPublishingType(s.to_string())),
        }
    }
}
