//! HTTP3 backend.
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use body::QuicIncomingBody;
use scuffle_context::ContextFutExt;
#[cfg(feature = "tracing")]
use tracing::Instrument;
use utils::copy_response_body;
#[cfg(feature = "webtransport")]
use {h3::ext::Protocol, h3_webtransport as h3wt};

use crate::error::HttpError;
use crate::service::{HttpService, HttpServiceFactory};

pub mod body;
mod utils;

/// A backend that handles incoming HTTP3 connections.
///
/// This is used internally by the [`HttpServer`](crate::server::HttpServer) but can be used directly if preferred.
///
/// Call [`run`](Http3Backend::run) to start the server.
#[derive(bon::Builder, Debug, Clone)]
pub struct Http3Backend<F> {
    /// The [`scuffle_context::Context`] this server will live by.
    #[builder(default = scuffle_context::Context::global())]
    ctx: scuffle_context::Context,
    /// The number of worker tasks to spawn for each server backend.
    #[builder(default = 1)]
    worker_tasks: usize,
    /// The service factory that will be used to create new services.
    service_factory: F,
    /// The address to bind to.
    ///
    /// Use `[::]` for a dual-stack listener.
    /// For example, use `[::]:80` to bind to port 80 on both IPv4 and IPv6.
    bind: SocketAddr,
    /// rustls config.
    ///
    /// Use this field to set the server into TLS mode.
    /// It will only accept TLS connections when this is set.
    rustls_config: tokio_rustls::rustls::ServerConfig,
}

impl<F> Http3Backend<F>
where
    F: HttpServiceFactory + Clone + Send + 'static,
    F::Error: std::error::Error + Send,
    F::Service: Clone + Send + 'static,
    <F::Service as HttpService>::Error: std::error::Error + Send + Sync,
    <F::Service as HttpService>::ResBody: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Data: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
{
    /// Run the HTTP3 server
    ///
    /// This function will bind to the address specified in `bind`, listen for incoming connections and handle requests.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(bind = %self.bind)))]
    pub async fn run(mut self) -> Result<(), HttpError<F>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("starting server");

        // not quite sure why this is necessary but it is
        self.rustls_config.max_early_data_size = u32::MAX;
        let crypto = h3_quinn::quinn::crypto::rustls::QuicServerConfig::try_from(self.rustls_config)?;
        let server_config = h3_quinn::quinn::ServerConfig::with_crypto(Arc::new(crypto));

        // Bind the UDP socket
        let socket = std::net::UdpSocket::bind(self.bind)?;

        // Runtime for the quinn endpoint
        let runtime = h3_quinn::quinn::default_runtime().ok_or_else(|| io::Error::other("no async runtime found"))?;

        // Create a child context for the workers so we can shut them down if one of them fails without shutting down the main context
        let (worker_ctx, worker_handler) = self.ctx.new_child();

        let workers = (0..self.worker_tasks).map(|_n| {
            let ctx = worker_ctx.clone();
            let service_factory = self.service_factory.clone();
            let server_config = server_config.clone();
            let socket = socket.try_clone().expect("failed to clone socket");
            let runtime = Arc::clone(&runtime);

            let worker_fut =
                async move {
                    let endpoint = h3_quinn::quinn::Endpoint::new(
                        h3_quinn::quinn::EndpointConfig::default(),
                        Some(server_config),
                        socket,
                        runtime,
                    )?;

                    #[cfg(feature = "tracing")]
                    tracing::trace!("waiting for connections");

                    while let Some(Some(new_conn)) = endpoint.accept().with_context(&ctx).await {
                        let mut service_factory = service_factory.clone();
                        let ctx = ctx.clone();

                        tokio::spawn(async move {
                            let _res: Result<_, HttpError<F>> = async move {
                            let Some(conn) = new_conn.with_context(&ctx).await.transpose()? else {
                                #[cfg(feature = "tracing")]
                                tracing::trace!("context done while accepting connection");
                                return Ok(());
                            };
                            let addr = conn.remote_address();
                            let client_certs = conn
                                .peer_identity()
                                .and_then(|any| any.downcast::<Vec<tokio_rustls::rustls::pki_types::CertificateDer>>().ok());

                            #[cfg(feature = "tracing")]
                            tracing::debug!(addr = %addr, "accepted quic connection");

                            let connection_fut = async move {
                                let Some(mut h3_conn) = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                                    .with_context(&ctx)
                                    .await
                                    .transpose()?
                                else {
                                    #[cfg(feature = "tracing")]
                                    tracing::trace!("context done while establishing connection");
                                    return Ok(());
                                };

                                let mut extra_extensions = http::Extensions::new();
                                extra_extensions.insert(crate::extensions::ClientAddr(addr));
                                if let Some(certs) = client_certs {
                                    extra_extensions.insert(crate::extensions::ClientIdentity(Arc::new(*certs)));
                                }

                                // make a new service for this connection
                                let mut http_service = service_factory
                                    .new_service(addr)
                                    .await
                                    .map_err(|e| HttpError::ServiceFactoryError(e))?;

                                loop {
                                    match h3_conn.accept().with_context(&ctx).await {
                                        Some(Ok(Some(resolver))) => {
                                            // Resolve the request
                                            let (req, stream) = match resolver.resolve_request().await {
                                                Ok(r) => r,
                                                Err(_err) => {
                                                    #[cfg(feature = "tracing")]
                                                    tracing::warn!("error on accept: {}", _err);
                                                    continue;
                                                }
                                            };

                                            #[cfg(feature = "tracing")]
                                            tracing::debug!(method = %req.method(), uri = %req.uri(), "received request");

                                            // Check if this is a WebTransport CONNECT request
                                            #[cfg(feature = "webtransport")]
                                            if req.extensions().get::<Protocol>() == Some(&Protocol::WEB_TRANSPORT)
                                                && req.method() == http::Method::CONNECT
                                            {
                                                #[cfg(feature = "tracing")]
                                                tracing::debug!("starting WebTransport session");

                                                match drive_webtransport_session::<F>(
                                                    req, stream, h3_conn, ctx, &mut http_service
                                                ).await {
                                                    Ok(_) => {
                                                        #[cfg(feature = "tracing")]
                                                        tracing::debug!("WebTransport session ended");
                                                    }
                                                    Err(err) => {
                                                        #[cfg(feature = "tracing")]
                                                        tracing::warn!(err = %err, "WebTransport session error");
                                                    }
                                                }
                                                break;
                                            }

                                            let (mut send, recv) = stream.split();

                                            let size_hint = req
                                                .headers()
                                                .get(http::header::CONTENT_LENGTH)
                                                .and_then(|len| len.to_str().ok().and_then(|x| x.parse().ok()));
                                            let body = QuicIncomingBody::new(recv, size_hint);
                                            let mut req = req.map(|_| crate::body::IncomingBody::from(body));

                                            req.extensions_mut().extend(extra_extensions.clone());

                                            tokio::spawn({
                                                let ctx = ctx.clone();
                                                let mut http_service = http_service.clone();
                                                async move {
                                                    let _res: Result<_, HttpError<F>> = async move {
                                                        let resp = http_service
                                                            .call(req)
                                                            .await
                                                            .map_err(|e| HttpError::ServiceError(e))?;
                                                        let (parts, body) = resp.into_parts();

                                                        send.send_response(http::Response::from_parts(parts, ())).await?;
                                                        copy_response_body(send, body).await?;

                                                        Ok(())
                                                    }
                                                    .await;

                                                    #[cfg(feature = "tracing")]
                                                    if let Err(e) = _res {
                                                        tracing::warn!(err = %e, "error handling request");
                                                    }

                                                    // This moves the context into the async block because it is dropped here
                                                    drop(ctx);
                                                }
                                            });
                                        }
                                        // indicating no more streams to be received
                                        Some(Ok(None)) => {
                                            break;
                                        }
                                        Some(Err(err)) => return Err(err.into()),
                                        // context is done
                                        None => {
                                            #[cfg(feature = "tracing")]
                                            tracing::trace!("context done, stopping connection loop");
                                            break;
                                        }
                                    }
                                }

                                #[cfg(feature = "tracing")]
                                tracing::trace!("connection closed");

                                Ok(())
                            };

                            #[cfg(feature = "tracing")]
                            let connection_fut = connection_fut.instrument(tracing::trace_span!("connection", addr = %addr));

                            connection_fut.await
                        }
                        .await;

                            #[cfg(feature = "tracing")]
                            if let Err(err) = _res {
                                tracing::warn!(err = %err, "error handling connection");
                            }
                        });
                    }

                    // shut down gracefully
                    // wait for connections to be closed before exiting
                    endpoint.wait_idle().await;

                    Ok::<_, crate::error::HttpError<F>>(())
                };

            #[cfg(feature = "tracing")]
            let worker_fut = worker_fut.instrument(tracing::trace_span!("worker", n = _n));

            tokio::spawn(worker_fut)
        });

        if let Err(_e) = futures::future::try_join_all(workers).await {
            #[cfg(feature = "tracing")]
            tracing::error!(err = %_e, "error running workers");
        }

        drop(worker_ctx);
        worker_handler.shutdown().await;

        #[cfg(feature = "tracing")]
        tracing::debug!("all workers finished");

        Ok(())
    }
}

#[cfg(feature = "webtransport")]
async fn drive_webtransport_session<F>(
    req: http::Request<()>,
    stream: h3::server::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,
    h3_conn: h3::server::Connection<h3_quinn::Connection, bytes::Bytes>,
    ctx: scuffle_context::Context,
    http_service: &mut F::Service,
) -> Result<(), crate::error::HttpError<F>>
where
    F: HttpServiceFactory + Clone + Send + 'static,
    F::Error: std::error::Error + Send,
    F::Service: Clone + Send + 'static,
    <F::Service as HttpService>::Error: std::error::Error + Send + Sync,
    <F::Service as HttpService>::ResBody: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Data: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
{
    // Accept the WebTransport session
    let session = std::sync::Arc::new(h3wt::server::WebTransportSession::accept(req, stream, h3_conn).await?);

    // Spawn task to handle unidirectional streams
    let uni_handle = tokio::spawn({
        let ctx = ctx.clone();
        let session = session.clone();
        async move {
            loop {
                match session.accept_uni().with_context(&ctx).await {
                    Some(Ok(Some((sid, stream)))) => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!(session_id = ?sid, "received WebTransport uni stream");

                        tokio::spawn({
                            let ctx = ctx.clone();
                            async move {
                                if let Err(err) = handle_webtransport_uni_stream(stream, ctx).await {
                                    #[cfg(feature = "tracing")]
                                    tracing::warn!(err = %err, "error handling WebTransport uni stream");
                                }
                            }
                        });
                    }
                    Some(Ok(None)) => break,
                    Some(Err(err)) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(err = %err, "WebTransport uni stream accept error");
                        break;
                    }
                    None => break,
                }
            }
        }
    });

    // Handle bidirectional streams and requests
    loop {
        match session.accept_bi().with_context(&ctx).await {
            Some(Ok(Some(h3wt::server::AcceptedBi::Request(req, stream)))) => {
                let (mut send, recv) = stream.split();
                let size_hint = req
                    .headers()
                    .get(http::header::CONTENT_LENGTH)
                    .and_then(|len| len.to_str().ok().and_then(|x| x.parse().ok()));
                let body = QuicIncomingBody::new(recv, size_hint);
                let req = req.map(|_| crate::body::IncomingBody::from(body));

                let resp = match http_service.call(req).await {
                    Ok(r) => r,
                    Err(_e) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(err = %_e, "service error in WebTransport request");
                        let _ = send
                            .send_response(
                                http::Response::builder()
                                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(())
                                    .unwrap(),
                            )
                            .await;
                        continue;
                    }
                };

                let (parts, body) = resp.into_parts();
                if let Err(err) = send.send_response(http::Response::from_parts(parts, ())).await {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(err = %err, "failed to send WebTransport response headers");
                    continue;
                }
                if let Err(err) = copy_response_body::<_, F>(send, body).await {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(err = %err, "failed to send WebTransport response body");
                }
            }
            Some(Ok(Some(h3wt::server::AcceptedBi::BidiStream(sid, bidi)))) => {
                // Handle raw bidirectional WebTransport stream
                #[cfg(feature = "tracing")]
                tracing::debug!(session_id = ?sid, "received WebTransport bidi stream");

                tokio::spawn({
                    let ctx = ctx.clone();
                    async move {
                        if let Err(err) = handle_webtransport_bidi_stream(bidi, ctx).await {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(err = %err, "error handling WebTransport bidi stream");
                        }
                    }
                });
            }
            Some(Ok(None)) => break,
            Some(Err(err)) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(err = %err, "WebTransport session error");
                break;
            }
            None => break,
        }
    }

    // Wait for uni stream handler to complete
    let _ = uni_handle.await;

    Ok(())
}

#[cfg(feature = "webtransport")]
async fn handle_webtransport_uni_stream(
    mut stream: h3wt::stream::RecvStream<h3_quinn::RecvStream, bytes::Bytes>,
    ctx: scuffle_context::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use bytes::Buf;
    use h3::quic::RecvStream;

    let mut buffer = Vec::new();
    loop {
        match std::future::poll_fn(|cx| stream.poll_data(cx)).with_context(&ctx).await {
            Some(Ok(Some(mut chunk))) => {
                let chunk_size = chunk.remaining();
                let chunk_bytes = chunk.copy_to_bytes(chunk_size);
                buffer.extend_from_slice(&chunk_bytes);

                #[cfg(feature = "tracing")]
                tracing::debug!(
                    bytes = chunk_size,
                    total = buffer.len(),
                    "received data on WebTransport uni stream"
                );
            }
            Some(Ok(None)) => {
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    total_bytes = buffer.len(),
                    data = ?String::from_utf8_lossy(&buffer),
                    "WebTransport uni stream finished"
                );
                break;
            }
            Some(Err(err)) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(err = %err, "error reading from WebTransport uni stream");
                return Err(err.into());
            }
            None => {
                #[cfg(feature = "tracing")]
                tracing::trace!("context done while reading WebTransport uni stream");
                return Ok(());
            }
        }
    }

    Ok(())
}

#[cfg(feature = "webtransport")]
async fn handle_webtransport_bidi_stream(
    mut stream: h3wt::stream::BidiStream<h3_quinn::BidiStream<bytes::Bytes>, bytes::Bytes>,
    ctx: scuffle_context::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use bytes::Buf;
    use h3::quic::RecvStream;

    // Read data from the receive side
    let mut buffer = Vec::new();
    loop {
        match std::future::poll_fn(|cx| stream.poll_data(cx)).with_context(&ctx).await {
            Some(Ok(Some(mut chunk))) => {
                let chunk_size = chunk.remaining();
                let chunk_bytes = chunk.copy_to_bytes(chunk_size);
                buffer.extend_from_slice(&chunk_bytes);

                #[cfg(feature = "tracing")]
                tracing::debug!(
                    bytes = chunk_size,
                    total = buffer.len(),
                    "received data on WebTransport bidi stream"
                );
            }
            Some(Ok(None)) => {
                // Stream finished
                #[cfg(feature = "tracing")]
                tracing::debug!(total_bytes = buffer.len(), "WebTransport bidi stream finished reading");
                break;
            }
            Some(Err(err)) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(err = %err, "error reading from WebTransport bidi stream");
                return Err(err.into());
            }
            None => {
                // Context cancelled
                #[cfg(feature = "tracing")]
                tracing::trace!("context done while reading WebTransport bidi stream");
                return Ok(());
            }
        }
    }

    // Echo back the received data
    if !buffer.is_empty() {
        #[cfg(feature = "tracing")]
        tracing::debug!(bytes = buffer.len(), "echoing data back on WebTransport bidi stream");

        let response_msg = format!("Echo: received {} bytes", buffer.len());
        let response_bytes = bytes::Bytes::from(response_msg);

        // Send the response using SendStreamUnframed
        use h3::quic::{SendStream, SendStreamUnframed};
        let mut bytes_buf = response_bytes.clone();
        while bytes_buf.has_remaining() {
            let written = std::future::poll_fn(|cx| stream.poll_send(cx, &mut bytes_buf)).await?;
            if written == 0 {
                break;
            }
        }
        std::future::poll_fn(|cx| stream.poll_finish(cx)).await?;
        #[cfg(feature = "tracing")]
        tracing::debug!("successfully echoed data on WebTransport bidi stream");
    }

    Ok(())
}
