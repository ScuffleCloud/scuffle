//! WebTransport session management for HTTP/3.
//!
//! This module provides types for handling WebTransport sessions over HTTP/3.
//! WebTransport allows bidirectional streams and datagrams to be established over QUIC.

use std::sync::Arc;
use std::{fmt, io};

use bytes::Bytes;
use h3::quic::StreamErrorIncoming;
use h3_webtransport::server::{AcceptedBi as H3AcceptedBi, WebTransportSession as H3WebTransportSession};
use h3_webtransport::stream::{BidiStream, RecvStream as WtRecvStream, SendStream as WtSendStream};

/// A WebTransport session handle.
///
/// This type provides access to bidirectional and unidirectional streams
/// for a WebTransport session established over HTTP/3.
///
/// The session can be retrieved from the request extensions when handling
/// a WebTransport CONNECT request.
///
/// # Example
///
/// ```rust,ignore
/// use scuffle_http::{IncomingRequest, Response};
/// use scuffle_http::backend::h3::webtransport::WebTransportSession;
///
/// async fn handle_webtransport(req: IncomingRequest) -> Result<Response<String>, std::convert::Infallible> {
///     if let Some(session) = req.extensions().get::<WebTransportSession>() {
///         // Handle WebTransport session
///         tokio::spawn({
///             let session = session.clone();
///             async move {
///                 while let Some(Ok(accepted)) = session.accept_bi().await {
///                     // Handle bidirectional streams
///                 }
///             }
///         });
///
///         return Ok(Response::builder()
///             .status(200)
///             .body(String::new())
///             .unwrap());
///     }
///
///     Ok(Response::builder()
///         .status(404)
///         .body(String::new())
///         .unwrap())
/// }
/// ```
#[derive(Clone)]
pub struct WebTransportSession {
    session: Arc<H3WebTransportSession<h3_quinn::Connection, Bytes>>,
}

impl WebTransportSession {
    /// Create a new WebTransport session from an h3-webtransport session.
    pub(crate) fn new(session: Arc<H3WebTransportSession<h3_quinn::Connection, Bytes>>) -> Self {
        Self { session }
    }

    /// Accept the next incoming bidirectional stream or request.
    ///
    /// Returns `None` when the session is closed or no more streams are available.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use scuffle_http::backend::h3::webtransport::{WebTransportSession, AcceptedBi};
    ///
    /// async fn handle_session(session: WebTransportSession) {
    ///     while let Some(Ok(accepted)) = session.accept_bi().await {
    ///         match accepted {
    ///             AcceptedBi::BidiStream(stream) => {
    ///                 // Handle raw bidirectional stream
    ///             }
    ///             AcceptedBi::Request(req, stream) => {
    ///                 // Handle HTTP request over WebTransport
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn accept_bi(&self) -> Option<Result<AcceptedBi, h3::error::StreamError>> {
        match self.session.accept_bi().await {
            Ok(Some(H3AcceptedBi::BidiStream(id, stream))) => {
                Some(Ok(AcceptedBi::BidiStream(WebTransportBidiStream { stream, _id: id })))
            }
            Ok(Some(H3AcceptedBi::Request(req, stream))) => {
                Some(Ok(AcceptedBi::Request(req, WebTransportRequestStream { stream })))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    /// Accept the next incoming unidirectional stream.
    ///
    /// Returns `None` when the session is closed or no more streams are available.
    pub async fn accept_uni(
        &self,
    ) -> Option<Result<(WebTransportStreamId, WebTransportRecvStream), h3::error::ConnectionError>> {
        match self.session.accept_uni().await {
            Ok(Some((id, stream))) => Some(Ok((WebTransportStreamId(id), WebTransportRecvStream { stream }))),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    /// Open a new bidirectional stream.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (mut send, mut recv) = session.open_bi().await?;
    /// send.write(Bytes::from("Hello")).await?;
    /// send.finish().await?;
    /// ```
    pub async fn open_bi(&self) -> Result<(WebTransportSendStream, WebTransportRecvStream), h3::error::StreamError> {
        let stream = self.session.open_bi(WebTransportStreamId::next_session_id()).await?;
        use h3::quic::BidiStream;
        let (send, recv) = stream.split();
        Ok((
            WebTransportSendStream { stream: send },
            WebTransportRecvStream { stream: recv },
        ))
    }

    /// Open a new unidirectional stream.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut send = session.open_uni().await?;
    /// send.write(Bytes::from("Hello")).await?;
    /// send.finish().await?;
    /// ```
    pub async fn open_uni(&self) -> Result<WebTransportSendStream, h3::error::StreamError> {
        let send = self.session.open_uni(WebTransportStreamId::next_session_id()).await?;
        Ok(WebTransportSendStream { stream: send })
    }

    /// Get the session ID for this WebTransport session.
    pub fn session_id(&self) -> h3_webtransport::SessionId {
        self.session.session_id()
    }

    /// Get a datagram sender for sending datagrams over this session.
    ///
    /// Datagrams are unreliable and unordered messages.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut sender = session.datagram_sender();
    /// sender.send_datagram(Bytes::from("Hello"))?;
    /// ```
    pub fn datagram_sender(
        &self,
    ) -> h3_datagram::datagram_handler::DatagramSender<
        <h3_quinn::Connection as h3_datagram::quic_traits::DatagramConnectionExt<Bytes>>::SendDatagramHandler,
        Bytes,
    > {
        self.session.datagram_sender()
    }

    /// Get a datagram reader for receiving datagrams over this session.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut reader = session.datagram_reader();
    /// while let Ok(datagram) = reader.read_datagram().await {
    ///     println!("Received: {} bytes", datagram.payload().len());
    /// }
    /// ```
    pub fn datagram_reader(
        &self,
    ) -> h3_datagram::datagram_handler::DatagramReader<
        <h3_quinn::Connection as h3_datagram::quic_traits::DatagramConnectionExt<Bytes>>::RecvDatagramHandler,
    > {
        self.session.datagram_reader()
    }
}

impl fmt::Debug for WebTransportSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebTransportSession").finish_non_exhaustive()
    }
}

/// An accepted bidirectional stream or request.
#[derive(Debug)]
pub enum AcceptedBi {
    /// A raw bidirectional stream.
    BidiStream(WebTransportBidiStream),
    /// An HTTP request over WebTransport.
    Request(http::Request<()>, WebTransportRequestStream),
}

/// A bidirectional WebTransport stream.
pub struct WebTransportBidiStream {
    stream: BidiStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    _id: h3_webtransport::SessionId,
}

impl WebTransportBidiStream {
    /// Split this stream into separate send and receive halves.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (mut send, mut recv) = bidi_stream.split();
    /// tokio::spawn(async move {
    ///     while let Ok(Some(data)) = recv.read().await {
    ///         println!("Received: {:?}", data);
    ///     }
    /// });
    /// send.write(Bytes::from("Hello")).await?;
    /// ```
    pub fn split(self) -> (WebTransportSendStream, WebTransportRecvStream) {
        use h3::quic::BidiStream;
        let (send, recv) = self.stream.split();
        (
            WebTransportSendStream { stream: send },
            WebTransportRecvStream { stream: recv },
        )
    }

    /// Read data from the receive side of the stream.
    pub async fn read(&mut self) -> Result<Option<Bytes>, StreamErrorIncoming> {
        use h3::quic::RecvStream;
        std::future::poll_fn(|cx| self.stream.poll_data(cx)).await
    }

    /// Read all remaining data from the receive side until the stream is finished.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let data = bidi_stream.read_to_end(1024 * 1024).await?; // max 1MB
    /// ```
    pub async fn read_to_end(&mut self, max_size: usize) -> Result<Bytes, io::Error> {
        let mut chunks = Vec::new();
        let mut total_size = 0;

        while let Some(chunk) = self
            .read()
            .await
            .map_err(|e| io::Error::other(format!("stream read error: {}", e)))?
        {
            total_size += chunk.len();
            if total_size > max_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "stream data too large: {} bytes exceeds maximum of {} bytes",
                        total_size, max_size
                    ),
                ));
            }
            chunks.push(chunk);
        }

        if chunks.is_empty() {
            Ok(Bytes::new())
        } else if chunks.len() == 1 {
            Ok(chunks.into_iter().next().unwrap())
        } else {
            let mut combined = bytes::BytesMut::with_capacity(total_size);
            for chunk in chunks {
                combined.extend_from_slice(&chunk);
            }
            Ok(combined.freeze())
        }
    }

    /// Write data to the send side of the stream.
    pub async fn write(&mut self, data: Bytes) -> Result<(), StreamErrorIncoming> {
        use bytes::Buf;
        use h3::quic::{SendStream, SendStreamUnframed};

        std::future::poll_fn(|cx| self.stream.poll_ready(cx)).await?;
        let mut buf = data;
        while buf.has_remaining() {
            let written = std::future::poll_fn(|cx| self.stream.poll_send(cx, &mut buf)).await?;
            if written == 0 {
                break;
            }
        }
        Ok(())
    }

    /// Finish writing to the stream.
    pub async fn finish(&mut self) -> Result<(), StreamErrorIncoming> {
        use h3::quic::SendStream;
        std::future::poll_fn(|cx| self.stream.poll_finish(cx)).await
    }
}

impl fmt::Debug for WebTransportBidiStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebTransportBidiStream").finish()
    }
}

/// A receive-only WebTransport stream.
pub struct WebTransportRecvStream {
    stream: WtRecvStream<h3_quinn::RecvStream, Bytes>,
}

impl WebTransportRecvStream {
    /// Read data from the stream.
    ///
    /// Returns `Ok(None)` when the stream is finished.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// while let Ok(Some(data)) = recv_stream.read().await {
    ///     println!("Received {} bytes", data.len());
    /// }
    /// ```
    pub async fn read(&mut self) -> Result<Option<Bytes>, StreamErrorIncoming> {
        use h3::quic::RecvStream;
        std::future::poll_fn(|cx| self.stream.poll_data(cx)).await
    }

    /// Read all remaining data from the stream until it's finished.
    ///
    /// This collects all chunks into a single `Bytes` object.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let data = recv_stream.read_to_end(1024 * 1024).await?; // max 1MB
    /// println!("Received complete message: {} bytes", data.len());
    /// ```
    pub async fn read_to_end(&mut self, max_size: usize) -> Result<Bytes, io::Error> {
        let mut chunks = Vec::new();
        let mut total_size = 0;

        while let Some(chunk) = self
            .read()
            .await
            .map_err(|e| io::Error::other(format!("stream read error: {}", e)))?
        {
            total_size += chunk.len();
            if total_size > max_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "stream data too large: {} bytes exceeds maximum of {} bytes",
                        total_size, max_size
                    ),
                ));
            }
            chunks.push(chunk);
        }

        if chunks.is_empty() {
            Ok(Bytes::new())
        } else if chunks.len() == 1 {
            Ok(chunks.into_iter().next().unwrap())
        } else {
            // Combine all chunks into a single buffer
            let mut combined = bytes::BytesMut::with_capacity(total_size);
            for chunk in chunks {
                combined.extend_from_slice(&chunk);
            }
            Ok(combined.freeze())
        }
    }

    /// Stop receiving data on this stream with an error code.
    pub fn stop_sending(&mut self, error_code: u64) {
        use h3::quic::RecvStream;
        self.stream.stop_sending(error_code)
    }
}

impl fmt::Debug for WebTransportRecvStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebTransportRecvStream").finish_non_exhaustive()
    }
}

/// A send-only WebTransport stream.
pub struct WebTransportSendStream {
    stream: WtSendStream<h3_quinn::SendStream<Bytes>, Bytes>,
}

impl WebTransportSendStream {
    /// Write data to the stream.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// send_stream.write(Bytes::from("Hello, world!")).await?;
    /// send_stream.finish().await?;
    /// ```
    pub async fn write(&mut self, data: Bytes) -> Result<(), StreamErrorIncoming> {
        use bytes::Buf;
        use h3::quic::{SendStream, SendStreamUnframed};

        std::future::poll_fn(|cx| self.stream.poll_ready(cx)).await?;
        let mut buf = data;
        while buf.has_remaining() {
            let written = std::future::poll_fn(|cx| self.stream.poll_send(cx, &mut buf)).await?;
            if written == 0 {
                break;
            }
        }
        Ok(())
    }

    /// Write all data and finish the stream in one operation.
    ///
    /// This is a convenience method that writes the data and then finishes the stream.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// send_stream.write_all(Bytes::from("Complete message")).await?;
    /// ```
    pub async fn write_all(&mut self, data: Bytes) -> Result<(), StreamErrorIncoming> {
        self.write(data).await?;
        self.finish().await
    }

    /// Finish writing to the stream.
    ///
    /// This signals that no more data will be sent on this stream.
    pub async fn finish(&mut self) -> Result<(), StreamErrorIncoming> {
        use h3::quic::SendStream;
        std::future::poll_fn(|cx| self.stream.poll_finish(cx)).await
    }

    /// Reset the stream with an error code.
    pub fn reset(&mut self, reset_code: u64) {
        use h3::quic::SendStream;
        self.stream.reset(reset_code)
    }
}
impl fmt::Debug for WebTransportSendStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebTransportSendStream").finish_non_exhaustive()
    }
}

/// A stream for handling HTTP requests over WebTransport.
pub struct WebTransportRequestStream {
    stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
}

impl WebTransportRequestStream {
    /// Split this stream into separate send and receive halves.
    pub fn split(
        self,
    ) -> (
        h3::server::RequestStream<h3_quinn::SendStream<Bytes>, Bytes>,
        h3::server::RequestStream<h3_quinn::RecvStream, Bytes>,
    ) {
        self.stream.split()
    }
}

impl fmt::Debug for WebTransportRequestStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebTransportRequestStream").finish_non_exhaustive()
    }
}

/// A WebTransport stream identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebTransportStreamId(h3_webtransport::SessionId);

impl WebTransportStreamId {
    fn next_session_id() -> h3_webtransport::SessionId {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        // SessionId is created from a VarInt-encoded StreamId
        let varint = h3::proto::varint::VarInt::from_u64(id).expect("valid varint");
        let stream_id = h3::quic::StreamId::from(varint);
        h3_webtransport::SessionId::from(stream_id)
    }

    /// Get the inner session ID.
    pub fn inner(&self) -> h3_webtransport::SessionId {
        self.0
    }
}

impl From<h3_webtransport::SessionId> for WebTransportStreamId {
    fn from(id: h3_webtransport::SessionId) -> Self {
        Self(id)
    }
}

impl fmt::Display for WebTransportStreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
