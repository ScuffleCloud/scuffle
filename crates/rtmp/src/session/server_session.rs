use std::time::Duration;

use bytes::BytesMut;
use scuffle_bytes_util::BytesCursorExt;
use scuffle_future_ext::FutureExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::SessionHandler;
use super::errors::SessionError;
use super::handler::SessionData;
use crate::chunk::{CHUNK_SIZE, ChunkDecoder, ChunkEncoder};
use crate::command_messages::netconnection::NetConnectionCommand;
use crate::command_messages::netstream::{NetStreamCommand, NetStreamCommandPublishPublishingType};
use crate::command_messages::{Command, CommandResultLevel, CommandType};
use crate::handshake;
use crate::handshake::HandshakeServer;
use crate::handshake::define::ServerHandshakeState;
use crate::messages::MessageData;
use crate::protocol_control_messages::{
    ProtocolControlMessageSetChunkSize, ProtocolControlMessageSetPeerBandwidth,
    ProtocolControlMessageSetPeerBandwidthLimitType, ProtocolControlMessageWindowAcknowledgementSize,
};
use crate::user_control_messages::EventMessageStreamBegin;

pub struct Session<S, H> {
    /// When you connect via rtmp, you specify the app name in the url
    /// For example: rtmp://localhost:1935/live/xyz
    /// The app name is "live"
    /// The next part of the url is the stream name (or the stream key) "xyz"
    /// However the stream key is not required to be the same for each stream
    /// you publish / play Traditionally we only publish a single stream per
    /// RTMP connection, However we can publish multiple streams per RTMP
    /// connection (using different stream keys) and or play multiple streams
    /// per RTMP connection (using different stream keys) as per the RTMP spec.
    app_name: Option<Box<str>>,

    /// Used to read and write data
    io: S,

    handler: H,

    /// Buffer to read data into
    read_buf: BytesMut,
    /// Buffer to write data to
    write_buf: Vec<u8>,

    /// Sometimes when doing the handshake we read too much data,
    /// this flag is used to indicate that we have data ready to parse and we
    /// should not read more data from the stream
    skip_read: bool,

    /// This is used to read the data from the stream and convert it into rtmp
    /// messages
    chunk_decoder: ChunkDecoder,
    /// This is used to convert rtmp messages into chunks
    chunk_encoder: ChunkEncoder,

    /// Is Publishing
    publishing_stream_ids: Vec<u32>,
}

impl<S, H> Session<S, H> {
    /// Create a new session.
    pub fn new(io: S, handler: H) -> Self {
        Self {
            app_name: None,
            io,
            handler,
            skip_read: false,
            chunk_decoder: ChunkDecoder::default(),
            chunk_encoder: ChunkEncoder::default(),
            read_buf: BytesMut::new(),
            write_buf: Vec::new(),
            publishing_stream_ids: Vec::new(),
        }
    }
}

impl<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, H: SessionHandler> Session<S, H> {
    /// Run the session to completion
    /// The result of the return value will be true if all publishers have
    /// disconnected If any publishers are still connected, the result will be
    /// false This can be used to detect non-graceful disconnects (ie. the
    /// client crashed)
    pub async fn run(&mut self) -> Result<bool, SessionError> {
        let mut handshaker = HandshakeServer::default();
        // Run the handshake to completion
        while !self.drive_handshake(&mut handshaker).await? {
            self.flush().await?;
        }

        // Drop the handshaker, we don't need it anymore
        // We can get rid of the memory that was allocated for it
        drop(handshaker);

        tracing::debug!("Handshake complete");

        // Drive the session to completion
        while match self.drive().await {
            Ok(v) => v,
            Err(err) if err.is_client_closed() => {
                // The client closed the connection
                // We are done with the session
                tracing::debug!("Client closed the connection");
                false
            }
            Err(e) => {
                return Err(e);
            }
        } {
            self.flush().await?;
        }

        // We should technically check the stream_map here
        // However most clients just disconnect without cleanly stopping the subscrition
        // streams (play streams) So we just check that all publishers have disconnected
        // cleanly
        Ok(self.publishing_stream_ids.is_empty())
    }

    /// This drives the first stage of the session.
    /// It is used to do the handshake with the client.
    /// The handshake is the first thing that happens when a client connects to a
    /// RTMP server.
    ///
    /// Returns true if the handshake is complete, false if the handshake is not complete yet.
    /// If the handshake is not complete yet, this function should be called again.
    async fn drive_handshake(&mut self, handshaker: &mut HandshakeServer) -> Result<bool, SessionError> {
        // Read the handshake data + 1 byte for the version
        const READ_SIZE: usize = handshake::define::RTMP_HANDSHAKE_SIZE + 1;
        self.read_buf.reserve(READ_SIZE);

        let mut bytes_read = 0;
        while bytes_read < READ_SIZE {
            let n = self
                .io
                .read_buf(&mut self.read_buf)
                .with_timeout(Duration::from_secs(2))
                .await??;
            bytes_read += n;
        }

        let mut cursor = std::io::Cursor::new(self.read_buf.split().freeze());

        handshaker.handshake(&mut cursor, &mut self.write_buf)?;

        if handshaker.state() == ServerHandshakeState::Finish {
            let over_read = cursor.extract_remaining();

            if !over_read.is_empty() {
                self.skip_read = true;
                self.read_buf.extend_from_slice(&over_read);
            }

            self.send_set_chunk_size().await?;

            // We are done with the handshake
            // This causes the loop to exit
            // And move onto the next stage of the session
            Ok(true)
        } else {
            // We are not done with the handshake yet
            // We need to read more data from the stream
            // This causes the loop to continue
            Ok(false)
        }
    }

    /// This drives the second and main stage of the session.
    /// It is used to read data from the stream and parse it into RTMP messages.
    /// We also send data to the client if they are playing a stream.
    ///
    /// Finish the handshake first by repeatedly calling [`drive_handshake`](Session::drive_handshake)
    /// until it returns true before calling this function.
    ///
    /// Returns true if the session is still active, false if the client has closed the connection.
    async fn drive(&mut self) -> Result<bool, SessionError> {
        // If we have data ready to parse, parse it
        if self.skip_read {
            self.skip_read = false;
        } else {
            self.read_buf.reserve(CHUNK_SIZE);

            let n = self
                .io
                .read_buf(&mut self.read_buf)
                .with_timeout(Duration::from_millis(2500))
                .await??;

            if n == 0 {
                return Ok(false);
            }
        }

        self.parse_chunks().await?;

        Ok(true)
    }

    /// Parse data from the client into RTMP messages and process them.
    async fn parse_chunks(&mut self) -> Result<(), SessionError> {
        while let Some(chunk) = self.chunk_decoder.read_chunk(&mut self.read_buf)? {
            let timestamp = chunk.message_header.timestamp;
            let msg_stream_id = chunk.message_header.msg_stream_id;

            if let Some(msg) = MessageData::parse(&chunk)? {
                self.process_message(msg, msg_stream_id, timestamp).await?;
            }
        }

        Ok(())
    }

    /// Process one RTMP message
    async fn process_message(&mut self, msg: MessageData, stream_id: u32, timestamp: u32) -> Result<(), SessionError> {
        match msg {
            MessageData::Amf0Command(command) => self.on_command_message(stream_id, command).await?,
            MessageData::SetChunkSize(ProtocolControlMessageSetChunkSize { chunk_size }) => {
                self.on_set_chunk_size(chunk_size as usize)?;
            }
            MessageData::AudioData { data } => {
                self.handler
                    .on_data(stream_id, SessionData::Audio { timestamp, data })
                    .await?;
            }
            MessageData::VideoData { data } => {
                self.handler
                    .on_data(stream_id, SessionData::Video { timestamp, data })
                    .await?;
            }
            MessageData::Amf0Data { data } => {
                self.handler.on_data(stream_id, SessionData::Amf0 { timestamp, data }).await?;
            }
        }

        Ok(())
    }

    /// Set the server chunk size to the client
    async fn send_set_chunk_size(&mut self) -> Result<(), SessionError> {
        ProtocolControlMessageSetChunkSize {
            chunk_size: CHUNK_SIZE as u32,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;
        self.chunk_encoder.set_chunk_size(CHUNK_SIZE);

        Ok(())
    }

    /// on_amf0_command_message is called when we receive an AMF0 command
    /// message from the client We then handle the command message
    async fn on_command_message(&mut self, stream_id: u32, command: Command) -> Result<(), SessionError> {
        match command.net_command {
            CommandType::NetConnection(NetConnectionCommand::Connect { app }) => {
                self.on_command_connect(stream_id, command.transaction_id, app).await?;
            }
            CommandType::NetConnection(NetConnectionCommand::CreateStream) => {
                self.on_command_create_stream(stream_id, command.transaction_id).await?;
            }
            CommandType::NetStream(NetStreamCommand::DeleteStream {
                stream_id: delete_stream_id,
            }) => {
                self.on_command_delete_stream(stream_id, command.transaction_id, delete_stream_id)
                    .await?;
            }
            CommandType::NetStream(NetStreamCommand::Play) | CommandType::NetStream(NetStreamCommand::Play2) => {
                return Err(SessionError::PlayNotSupported);
            }
            CommandType::NetStream(NetStreamCommand::Publish {
                publishing_name,
                publishing_type,
            }) => {
                self.on_command_publish(stream_id, command.transaction_id, publishing_name, publishing_type)
                    .await?;
            }
            CommandType::NetStream(NetStreamCommand::CloseStream) => {
                // Not sure what this is for
            }
            // ignore everything else
            _ => {}
        }

        Ok(())
    }

    /// on_set_chunk_size is called when we receive a set chunk size message
    /// from the client We then update the chunk size of the unpacketizer
    fn on_set_chunk_size(&mut self, chunk_size: usize) -> Result<(), SessionError> {
        if self.chunk_decoder.update_max_chunk_size(chunk_size) {
            Ok(())
        } else {
            Err(SessionError::InvalidChunkSize(chunk_size))
        }
    }

    /// on_command_connect is called when we receive a amf0 command message with
    /// the name "connect" We then handle the connect message
    /// This is called when the client first connects to the server
    async fn on_command_connect(&mut self, _stream_id: u32, transaction_id: f64, app: String) -> Result<(), SessionError> {
        ProtocolControlMessageWindowAcknowledgementSize {
            acknowledgement_window_size: CHUNK_SIZE as u32,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;

        ProtocolControlMessageSetPeerBandwidth {
            acknowledgement_window_size: CHUNK_SIZE as u32,
            limit_type: ProtocolControlMessageSetPeerBandwidthLimitType::Dynamic,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;

        self.app_name = Some(Box::from(app));

        let result = NetConnectionCommand::ConnectResult {
            fmsver: "FMS/3,0,1,123".to_string(), // flash version (this value is used by other media servers as well)
            capabilities: 31.0,                  // No idea what this means, but it is used by other media servers as well
            level: CommandResultLevel::Status,
            code: "NetConnection.Connect.Success".to_string(),
            description: "Connection Succeeded.".to_string(),
            encoding: 0.0,
        };

        Command {
            net_command: CommandType::NetConnection(result),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;

        Ok(())
    }

    /// on_command_create_stream is called when we receive a amf0 command
    /// message with the name "createStream" We then handle the createStream
    /// message This is called when the client wants to create a stream
    /// A NetStream is used to start publishing or playing a stream
    async fn on_command_create_stream(&mut self, _stream_id: u32, transaction_id: f64) -> Result<(), SessionError> {
        // 1.0 is the Stream ID of the stream we are creating
        Command {
            net_command: CommandType::NetConnection(NetConnectionCommand::CreateStreamResult { stream_id: 1.0 }),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;

        Ok(())
    }

    /// A delete stream message is unrelated to the NetConnection close method.
    /// Delete stream is basically a way to tell the server that you are done
    /// publishing or playing a stream. The server will then remove the stream
    /// from its list of streams.
    async fn on_command_delete_stream(
        &mut self,
        _stream_id: u32,
        transaction_id: f64,
        delete_stream_id: f64,
    ) -> Result<(), SessionError> {
        let stream_id = delete_stream_id as u32;

        self.handler.on_unpublish(stream_id).await?;

        // Remove the stream id from the list of publishing stream ids
        self.publishing_stream_ids.retain(|id| *id != stream_id);

        Command {
            net_command: CommandType::NetStream(NetStreamCommand::OnStatus {
                level: CommandResultLevel::Status,
                code: "NetStream.DeleteStream.Suceess".to_string(),
                description: "".to_string(),
            }),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;

        Ok(())
    }

    /// on_command_publish is called when we receive a amf0 command message with
    /// the name "publish" publish commands are used to publish a stream to the
    /// server ie. the user wants to start streaming to the server
    async fn on_command_publish(
        &mut self,
        stream_id: u32,
        transaction_id: f64,
        publishing_name: String,
        _publishing_type: NetStreamCommandPublishPublishingType,
    ) -> Result<(), SessionError> {
        let Some(app_name) = &self.app_name else {
            return Err(SessionError::NoAppName);
        };

        self.handler
            .on_publish(stream_id, app_name.as_ref(), publishing_name.as_ref())
            .await?;

        self.publishing_stream_ids.push(stream_id);

        EventMessageStreamBegin { stream_id }.write(&self.chunk_encoder, &mut self.write_buf)?;

        Command {
            net_command: CommandType::NetStream(NetStreamCommand::OnStatus {
                level: CommandResultLevel::Status,
                code: "NetStream.Publish.Start".to_string(),
                description: "".to_string(),
            }),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_encoder)?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<(), SessionError> {
        if !self.write_buf.is_empty() {
            self.io
                .write_all(self.write_buf.as_ref())
                .with_timeout(Duration::from_secs(2))
                .await??;
            self.write_buf.clear();
        }

        Ok(())
    }
}
