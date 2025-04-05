//! RTMP server session.

use std::time::Duration;

use bytes::BytesMut;
use scuffle_bytes_util::{BytesCursorExt, StringCow};
use scuffle_context::ContextFutExt;
use scuffle_future_ext::FutureExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::chunk::CHUNK_SIZE;
use crate::chunk::reader::ChunkReader;
use crate::chunk::writer::ChunkWriter;
use crate::command_messages::netconnection::{
    CapsExMask, NetConnectionCommand, NetConnectionCommandConnect, NetConnectionCommandConnectResult,
};
use crate::command_messages::netstream::{NetStreamCommand, NetStreamCommandPublishPublishingType};
use crate::command_messages::on_status::{OnStatus, OnStatusCode};
use crate::command_messages::{Command, CommandResultLevel, CommandType};
use crate::handshake;
use crate::handshake::HandshakeServer;
use crate::messages::MessageData;
use crate::protocol_control_messages::{
    ProtocolControlMessageAcknowledgement, ProtocolControlMessageSetChunkSize, ProtocolControlMessageSetPeerBandwidth,
    ProtocolControlMessageSetPeerBandwidthLimitType, ProtocolControlMessageWindowAcknowledgementSize,
};
use crate::user_control_messages::EventMessageStreamBegin;

mod error;
mod handler;

pub use error::ServerSessionError;
pub use handler::{SessionData, SessionHandler};

// The default acknowledgement window size that is used until the client sends a
// new acknowledgement window size.
// This is a common value used by other media servers as well.
// - https://github.com/FFmpeg/FFmpeg/blob/154c00514d889d27ae84a1001e00f9032fdc1c54/libavformat/rtmpproto.c#L2850
const DEFAULT_ACKNOWLEDGEMENT_WINDOW_SIZE: u32 = 2_500_000; // 2.5 MB

/// A RTMP server session that is used to communicate with a client.
///
/// This provides a high-level API to drive a RTMP session.
pub struct ServerSession<S, H> {
    /// The context of the session
    /// A reconnect request will be sent if this context gets cancelled.
    ctx: Option<scuffle_context::Context>,
    /// Keep track of whether a reconnect request has already been sent.
    reconnect_request_sent: bool,
    /// When you connect via rtmp, you specify the app name in the url
    /// For example: rtmp://localhost:1935/live/xyz
    /// The app name is "live"
    /// The next part of the url is the stream name (or the stream key) "xyz"
    /// However the stream key is not required to be the same for each stream
    /// you publish / play Traditionally we only publish a single stream per
    /// RTMP connection, However we can publish multiple streams per RTMP
    /// connection (using different stream keys) and or play multiple streams
    /// per RTMP connection (using different stream keys) as per the RTMP spec.
    app_name: Option<StringCow<'static>>,
    caps_ex: Option<CapsExMask>,
    /// Used to read and write data
    io: S,
    handler: H,
    /// The size of the acknowledgement window
    acknowledgement_window_size: u32,
    /// The number of bytes read from the stream. Value wraps when reaching u32::MAX.
    /// This is used to know when to send acknoledgements.
    sequence_number: u32,
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
    chunk_reader: ChunkReader,
    /// This is used to convert rtmp messages into chunks
    chunk_writer: ChunkWriter,
    /// Is Publishing
    publishing_stream_ids: Vec<u32>,
}

impl<S, H> ServerSession<S, H> {
    /// Create a new session.
    pub fn new(io: S, handler: H) -> Self {
        Self {
            ctx: None,
            reconnect_request_sent: false,
            app_name: None,
            caps_ex: None,
            io,
            handler,
            acknowledgement_window_size: DEFAULT_ACKNOWLEDGEMENT_WINDOW_SIZE,
            sequence_number: 0,
            skip_read: false,
            chunk_reader: ChunkReader::default(),
            chunk_writer: ChunkWriter::default(),
            read_buf: BytesMut::new(),
            write_buf: Vec::new(),
            publishing_stream_ids: Vec::new(),
        }
    }

    /// Set the context of the session.
    pub fn with_context(mut self, ctx: scuffle_context::Context) -> Self {
        self.ctx = Some(ctx);
        self
    }
}

impl<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, H: SessionHandler> ServerSession<S, H> {
    /// Run the session to completion
    /// The result of the return value will be true if all publishers have
    /// disconnected If any publishers are still connected, the result will be
    /// false This can be used to detect non-graceful disconnects (ie. the
    /// client crashed)
    pub async fn run(mut self) -> Result<bool, crate::error::RtmpError> {
        let ctx = self.ctx.clone().unwrap_or_else(scuffle_context::Context::global);

        let mut handshaker = HandshakeServer::default();
        // Run the handshake to completion
        loop {
            match self.drive_handshake(&mut handshaker).with_context(&ctx).await {
                Some(Ok(false)) => self.flush().await?, // Continue driving
                Some(Ok(true)) => break,                // Handshake is complete
                Some(Err(e)) => return Err(e),
                None => return Ok(false), // Context was cancelled
            }
        }

        // Drop the handshaker, we don't need it anymore
        // We can get rid of the memory that was allocated for it
        drop(handshaker);

        tracing::debug!("handshake complete");

        // Drive the session to completion
        loop {
            match self.drive().await {
                Ok(true) => self.flush().await?, // Continue driving
                Ok(false) => break,              // Client has closed the connection
                Err(err) if err.is_client_closed() => {
                    // The client closed the connection
                    // We are done with the session
                    tracing::debug!("client closed the connection");
                    break;
                }
                Err(e) => return Err(e),
            }
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
    async fn drive_handshake(&mut self, handshaker: &mut HandshakeServer) -> Result<bool, crate::error::RtmpError> {
        // Read the handshake data + 1 byte for the version
        const READ_SIZE: usize = handshake::RTMP_HANDSHAKE_SIZE + 1;
        self.read_buf.reserve(READ_SIZE);

        let mut bytes_read = 0;
        while bytes_read < READ_SIZE {
            let n = self
                .io
                .read_buf(&mut self.read_buf)
                .with_timeout(Duration::from_secs(2))
                .await
                .map_err(ServerSessionError::Timeout)??;
            bytes_read += n;

            self.sequence_number = self.sequence_number.wrapping_add(n.try_into().unwrap_or(u32::MAX));
        }

        let mut cursor = std::io::Cursor::new(self.read_buf.split().freeze());

        handshaker.handshake(&mut cursor, &mut self.write_buf)?;

        if handshaker.is_finished() {
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
    async fn drive(&mut self) -> Result<bool, crate::error::RtmpError> {
        // Send a reconnect request if we haven't yet, the client supports it and the context is cancelled
        if !self.reconnect_request_sent
            && self.caps_ex.is_some_and(|c| c.intersects(CapsExMask::Reconnect))
            && self.ctx.as_ref().is_some_and(|ctx| ctx.is_done())
        {
            tracing::debug!("sending reconnect request");

            OnStatus {
                code: OnStatusCode::NET_CONNECTION_CONNECT_RECONNECT_REQUEST,
                level: CommandResultLevel::Status,
                description: None,
                others: None,
            }
            .write(&mut self.write_buf, 0.0)?;

            self.reconnect_request_sent = true;
        }

        // If we have data ready to parse, parse it
        if self.skip_read {
            self.skip_read = false;
        } else {
            self.read_buf.reserve(CHUNK_SIZE);

            let n = self
                .io
                .read_buf(&mut self.read_buf)
                .with_timeout(Duration::from_millis(2500))
                .await
                .map_err(ServerSessionError::Timeout)?? as u32;

            if n == 0 {
                return Ok(false);
            }

            // We have to send an acknowledgement every `self.acknowledgement_window_size` bytes.
            // We also have to keep track of the total number of bytes read from the stream in `self.sequence_number`
            // because it has to be sent as part of an acknowledgement message.

            // This condition checks if we have read enough bytes to send the next acknowledgement.
            // - `self.sequence_number % self.acknowledgement_window_size` calculates the number of bytes read since
            //   the last acknowledgement.
            // - `n` is the number of bytes read in this read operation.
            // If the sum of the two is greater than or equal to the window size, we know that
            // we just exceeded the window size and we need to send an acknowledgement again.
            if (self.sequence_number % self.acknowledgement_window_size) + n >= self.acknowledgement_window_size {
                tracing::debug!(sequence_number = %self.sequence_number, "sending acknowledgement");

                // Send acknowledgement
                ProtocolControlMessageAcknowledgement {
                    sequence_number: self.sequence_number,
                }
                .write(&mut self.write_buf, &self.chunk_writer)?;
            }

            // Wrap back to 0 when we reach u32::MAX
            self.sequence_number = self.sequence_number.wrapping_add(n);
        }

        self.process_chunks().await?;

        Ok(true)
    }

    /// Parse data from the client into RTMP messages and process them.
    async fn process_chunks(&mut self) -> Result<(), crate::error::RtmpError> {
        while let Some(chunk) = self.chunk_reader.read_chunk(&mut self.read_buf)? {
            let timestamp = chunk.message_header.timestamp;
            let msg_stream_id = chunk.message_header.msg_stream_id;

            let msg = MessageData::read(&chunk)?;
            self.process_message(msg, msg_stream_id, timestamp).await?;
        }

        Ok(())
    }

    /// Process one RTMP message
    async fn process_message(
        &mut self,
        msg: MessageData<'_>,
        stream_id: u32,
        timestamp: u32,
    ) -> Result<(), crate::error::RtmpError> {
        match msg {
            MessageData::Amf0Command(command) => self.on_command_message(stream_id, command).await?,
            MessageData::SetChunkSize(ProtocolControlMessageSetChunkSize { chunk_size }) => {
                self.on_set_chunk_size(chunk_size as usize)?;
            }
            MessageData::SetAcknowledgementWindowSize(ProtocolControlMessageWindowAcknowledgementSize {
                acknowledgement_window_size,
            }) => {
                self.on_acknowledgement_window_size(acknowledgement_window_size)?;
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
            MessageData::DataAmf0 { data } => {
                self.handler.on_data(stream_id, SessionData::Amf0 { timestamp, data }).await?;
            }
            MessageData::Unknown(unknown_message) => {
                self.handler.on_unknown_message(stream_id, unknown_message).await?;
            }
            // ignore everything else
            _ => {}
        }

        Ok(())
    }

    /// Set the server chunk size to the client
    async fn send_set_chunk_size(&mut self) -> Result<(), crate::error::RtmpError> {
        ProtocolControlMessageSetChunkSize {
            chunk_size: CHUNK_SIZE as u32,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;
        self.chunk_writer.set_chunk_size(CHUNK_SIZE);

        Ok(())
    }

    /// on_amf0_command_message is called when we receive an AMF0 command
    /// message from the client We then handle the command message
    async fn on_command_message(&mut self, stream_id: u32, command: Command<'_>) -> Result<(), crate::error::RtmpError> {
        match command.command_type {
            CommandType::NetConnection(NetConnectionCommand::Connect(connect)) => {
                self.on_command_connect(stream_id, command.transaction_id, connect).await?;
            }
            CommandType::NetConnection(NetConnectionCommand::CreateStream) => {
                self.on_command_create_stream(stream_id, command.transaction_id).await?;
            }
            CommandType::NetStream(NetStreamCommand::Play { .. })
            | CommandType::NetStream(NetStreamCommand::Play2 { .. }) => {
                return Err(crate::error::RtmpError::Session(ServerSessionError::PlayNotSupported));
            }
            CommandType::NetStream(NetStreamCommand::DeleteStream {
                stream_id: delete_stream_id,
            }) => {
                self.on_command_delete_stream(stream_id, command.transaction_id, delete_stream_id)
                    .await?;
            }
            CommandType::NetStream(NetStreamCommand::CloseStream) => {
                // Not sure what this does, might be important
            }
            CommandType::NetStream(NetStreamCommand::Publish {
                publishing_name,
                publishing_type,
            }) => {
                self.on_command_publish(stream_id, command.transaction_id, publishing_name.as_str(), publishing_type)
                    .await?;
            }
            CommandType::Unknown(unknown_command) => {
                self.handler.on_unknown_command(stream_id, unknown_command).await?;
            }
            // ignore everything else
            _ => {}
        }

        Ok(())
    }

    /// on_set_chunk_size is called when we receive a set chunk size message
    /// from the client We then update the chunk size of the unpacketizer
    fn on_set_chunk_size(&mut self, chunk_size: usize) -> Result<(), crate::error::RtmpError> {
        if self.chunk_reader.update_max_chunk_size(chunk_size) {
            Ok(())
        } else {
            Err(crate::error::RtmpError::Session(ServerSessionError::InvalidChunkSize(
                chunk_size,
            )))
        }
    }

    /// on_acknowledgement_window_size is called when we receive a new acknowledgement window size
    /// from the client.
    fn on_acknowledgement_window_size(&mut self, acknowledgement_window_size: u32) -> Result<(), crate::error::RtmpError> {
        tracing::debug!(acknowledgement_window_size = %acknowledgement_window_size, "received new acknowledgement window size");
        self.acknowledgement_window_size = acknowledgement_window_size;
        Ok(())
    }

    /// on_command_connect is called when we receive a amf0 command message with
    /// the name "connect" We then handle the connect message
    /// This is called when the client first connects to the server
    async fn on_command_connect(
        &mut self,
        _stream_id: u32,
        transaction_id: f64,
        connect: NetConnectionCommandConnect<'_>,
    ) -> Result<(), crate::error::RtmpError> {
        ProtocolControlMessageWindowAcknowledgementSize {
            acknowledgement_window_size: CHUNK_SIZE as u32,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;

        ProtocolControlMessageSetPeerBandwidth {
            acknowledgement_window_size: CHUNK_SIZE as u32,
            limit_type: ProtocolControlMessageSetPeerBandwidthLimitType::Dynamic,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;

        self.app_name = Some(connect.app.into_owned());
        self.caps_ex = connect.caps_ex;

        let result = NetConnectionCommand::ConnectResult(NetConnectionCommandConnectResult::default());

        Command {
            command_type: CommandType::NetConnection(result),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;

        Ok(())
    }

    /// on_command_create_stream is called when we receive a amf0 command
    /// message with the name "createStream" We then handle the createStream
    /// message This is called when the client wants to create a stream
    /// A NetStream is used to start publishing or playing a stream
    async fn on_command_create_stream(
        &mut self,
        _stream_id: u32,
        transaction_id: f64,
    ) -> Result<(), crate::error::RtmpError> {
        // 1.0 is the Stream ID of the stream we are creating
        Command {
            command_type: CommandType::NetConnection(NetConnectionCommand::CreateStreamResult { stream_id: 1.0 }),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;

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
    ) -> Result<(), crate::error::RtmpError> {
        let stream_id = delete_stream_id as u32;

        self.handler.on_unpublish(stream_id).await?;

        // Remove the stream id from the list of publishing stream ids
        self.publishing_stream_ids.retain(|id| *id != stream_id);

        Command {
            command_type: CommandType::OnStatus(OnStatus {
                level: CommandResultLevel::Status,
                code: OnStatusCode::NET_STREAM_DELETE_STREAM_SUCCESS,
                description: None,
                others: None,
            }),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;

        Ok(())
    }

    /// on_command_publish is called when we receive a amf0 command message with
    /// the name "publish" publish commands are used to publish a stream to the
    /// server ie. the user wants to start streaming to the server
    async fn on_command_publish(
        &mut self,
        stream_id: u32,
        transaction_id: f64,
        publishing_name: &str,
        _publishing_type: NetStreamCommandPublishPublishingType<'_>,
    ) -> Result<(), crate::error::RtmpError> {
        let Some(app_name) = &self.app_name else {
            // The app name is not set yet
            return Err(crate::error::RtmpError::Session(ServerSessionError::PublishBeforeConnect));
        };

        self.handler.on_publish(stream_id, app_name.as_ref(), publishing_name).await?;

        self.publishing_stream_ids.push(stream_id);

        EventMessageStreamBegin { stream_id }.write(&self.chunk_writer, &mut self.write_buf)?;

        Command {
            command_type: CommandType::OnStatus(OnStatus {
                level: CommandResultLevel::Status,
                code: OnStatusCode::NET_STREAM_PUBLISH_START,
                description: None,
                others: None,
            }),
            transaction_id,
        }
        .write(&mut self.write_buf, &self.chunk_writer)?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<(), crate::error::RtmpError> {
        if !self.write_buf.is_empty() {
            self.io
                .write_all(self.write_buf.as_ref())
                .with_timeout(Duration::from_secs(2))
                .await
                .map_err(ServerSessionError::Timeout)??;
            self.write_buf.clear();
        }

        Ok(())
    }
}
