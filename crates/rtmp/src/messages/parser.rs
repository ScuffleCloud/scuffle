use super::define::{MessageData, MessageType};
use super::errors::MessageError;
use crate::chunk::Chunk;
use crate::command_messages::Command;
use crate::protocol_control_messages::ProtocolControlMessageSetChunkSize;

impl MessageData<'_> {
    pub fn parse(chunk: &Chunk) -> Result<Option<MessageData>, MessageError> {
        match chunk.message_header.msg_type_id {
            // Protocol Control Messages
            MessageType::SetChunkSize => {
                let data = ProtocolControlMessageSetChunkSize::read(&chunk.payload)?;
                Ok(Some(MessageData::SetChunkSize(data)))
            }
            // RTMP Command Messages
            MessageType::CommandAMF0 => Ok(Some(MessageData::Amf0Command(Command::read(&chunk.payload)?))),
            // Metadata
            MessageType::DataAMF0 => Ok(Some(MessageData::Amf0Data {
                data: chunk.payload.clone(),
            })),
            MessageType::Audio => Ok(Some(MessageData::AudioData {
                data: chunk.payload.clone(),
            })),
            MessageType::Video => Ok(Some(MessageData::VideoData {
                data: chunk.payload.clone(),
            })),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;
    use scuffle_amf0::{Amf0Encoder, Amf0Value};

    use super::*;
    use crate::command_messages::CommandType;
    use crate::command_messages::netconnection::NetConnectionCommand;

    #[test]
    fn test_parse_command() {
        let mut amf0_writer = Vec::new();

        Amf0Encoder::encode_string(&mut amf0_writer, "connect").unwrap();
        Amf0Encoder::encode_number(&mut amf0_writer, 1.0).unwrap();
        Amf0Encoder::encode_object(&mut amf0_writer, &[("app".into(), Amf0Value::String("testapp".into()))]).unwrap();

        let amf_data = Bytes::from(amf0_writer);

        let chunk = Chunk::new(0, 0, MessageType::CommandAMF0, 0, amf_data);

        let message = MessageData::parse(&chunk).expect("no errors").expect("message");
        match message {
            MessageData::Amf0Command(command) => {
                let Command {
                    transaction_id,
                    net_command,
                } = command;
                assert_eq!(transaction_id, 1.0);

                let CommandType::NetConnection(NetConnectionCommand::Connect { app }) = net_command else {
                    panic!("wrong command");
                };

                assert_eq!(app, "testapp");
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_audio_packet() {
        let chunk = Chunk::new(0, 0, MessageType::Audio, 0, vec![0x00, 0x00, 0x00, 0x00].into());

        let message = MessageData::parse(&chunk).expect("no errors").expect("message");
        match message {
            MessageData::AudioData { data } => {
                assert_eq!(data, vec![0x00, 0x00, 0x00, 0x00]);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_video_packet() {
        let chunk = Chunk::new(0, 0, MessageType::Video, 0, vec![0x00, 0x00, 0x00, 0x00].into());

        let message = MessageData::parse(&chunk).expect("no errors").expect("message");
        match message {
            MessageData::VideoData { data } => {
                assert_eq!(data, vec![0x00, 0x00, 0x00, 0x00]);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_set_chunk_size() {
        let chunk = Chunk::new(0, 0, MessageType::SetChunkSize, 0, vec![0x00, 0xFF, 0xFF, 0xFF].into());

        let message = MessageData::parse(&chunk).expect("no errors").expect("message");
        match message {
            MessageData::SetChunkSize(ProtocolControlMessageSetChunkSize { chunk_size }) => {
                assert_eq!(chunk_size, 0x00FFFFFF);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_metadata() {
        let mut amf0_writer = Vec::new();

        Amf0Encoder::encode_string(&mut amf0_writer, "onMetaData").unwrap();
        Amf0Encoder::encode_object(&mut amf0_writer, &[("duration".into(), Amf0Value::Number(0.0))]).unwrap();

        let amf_data = Bytes::from(amf0_writer);
        let chunk = Chunk::new(0, 0, MessageType::DataAMF0, 0, amf_data.clone());

        let message = MessageData::parse(&chunk).expect("no errors").expect("message");
        match message {
            MessageData::Amf0Data { data } => {
                assert_eq!(data, amf_data);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_unsupported_message_type() {
        let chunk = Chunk::new(0, 0, MessageType::Aggregate, 0, vec![0x00, 0x00, 0x00, 0x00].into());

        assert!(MessageData::parse(&chunk).expect("no errors").is_none())
    }
}
