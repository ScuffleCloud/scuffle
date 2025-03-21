//! Hyper backend.
use std::fmt::Debug;
use std::net::SocketAddr;

use scuffle_context::ContextFutExt;
#[cfg(feature = "tracing")]
use tracing::Instrument;

use crate::error::Error;
use crate::server::ConfigureSocketCallback;
use crate::service::{HttpService, HttpServiceFactory};

mod handler;
mod stream;
mod utils;

/// A backend that handles incoming HTTP connections using a hyper backend.
///
/// This is used internally by the [`HttpServer`](crate::server::HttpServer) but can be used directly if preferred.
///
/// Call [`run`](HyperBackend::run) to start the server.
#[derive(Debug, Clone, bon::Builder)]
pub struct HyperBackend<F> {
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
    /// Callback to configure socket
    configure_sock: Option<ConfigureSocketCallback>,
    /// rustls config.
    ///
    /// Use this field to set the server into TLS mode.
    /// It will only accept TLS connections when this is set.
    #[cfg(feature = "tls-rustls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls-rustls")))]
    rustls_config: Option<rustls::ServerConfig>,
    /// Enable HTTP/1.1.
    #[cfg(feature = "http1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http1")))]
    #[builder(default = true)]
    http1_enabled: bool,
    /// Enable HTTP/2.
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    #[builder(default = true)]
    http2_enabled: bool,
}

impl<F> HyperBackend<F>
where
    F: HttpServiceFactory + Clone + Send + 'static,
    F::Error: std::error::Error + Send,
    F::Service: Clone + Send + 'static,
    <F::Service as HttpService>::Error: std::error::Error + Send + Sync,
    <F::Service as HttpService>::ResBody: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Data: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
{
    /// Run the HTTP server
    ///
    /// This function will bind to the address specified in `bind`, listen for incoming connections and handle requests.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(bind = %self.bind)))]
    #[allow(unused_mut)] // allow the unused `mut self`
    pub async fn run(mut self) -> Result<(), Error<F>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("starting server");

        // reset to 0 because everything explodes if it's not
        // https://github.com/hyperium/hyper/issues/3841
        #[cfg(feature = "tls-rustls")]
        if let Some(rustls_config) = self.rustls_config.as_mut() {
            rustls_config.max_early_data_size = 0;
        }

        // We have to create an std listener first because the tokio listener isn't clonable
        let listener = {
            let mut sock = socket2::Socket::new(
                match self.bind {
                    SocketAddr::V4(_) => socket2::Domain::IPV4,
                    SocketAddr::V6(_) => socket2::Domain::IPV6,
                },
                socket2::Type::STREAM,
                Some(socket2::Protocol::TCP),
            )?;

            sock.set_nonblocking(true)?;
            sock.set_only_v6(false)?;
            if let Some(cfg_fn) = self.configure_sock.as_ref() {
                sock = cfg_fn.call(sock)?;
            }

            sock.bind(&socket2::SockAddr::from(self.bind))?;
            sock.listen(128)?;

            std::net::TcpListener::from(sock)
        };

        #[cfg(feature = "tls-rustls")]
        let tls_acceptor = self
            .rustls_config
            .map(|c| tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(c)));

        // Create a child context for the workers so we can shut them down if one of them fails without shutting down the main context
        let (worker_ctx, worker_handler) = self.ctx.new_child();

        let workers = (0..self.worker_tasks)
            .map(|_n| {
                let service_factory = self.service_factory.clone();
                let ctx = worker_ctx.clone();
                let std_listener = listener.try_clone()?;
                let listener = tokio::net::TcpListener::from_std(std_listener)?;
                #[cfg(feature = "tls-rustls")]
                let tls_acceptor = tls_acceptor.clone();

                let worker_fut = async move {
                    loop {
                        #[cfg(feature = "tracing")]
                        tracing::trace!("waiting for connections");

                        let (mut stream, addr) = match listener.accept().with_context(ctx.clone()).await {
                            Some(Ok((tcp_stream, addr))) => (stream::Stream::Tcp(tcp_stream), addr),
                            Some(Err(e)) if utils::is_fatal_tcp_error(&e) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(err = %e, "failed to accept tcp connection");
                                return Err(Error::<F>::from(e));
                            }
                            Some(Err(_)) => continue,
                            None => {
                                #[cfg(feature = "tracing")]
                                tracing::trace!("context done, stopping listener");
                                break;
                            }
                        };

                        #[cfg(feature = "tracing")]
                        tracing::trace!(addr = %addr, "accepted tcp connection");

                        let ctx = ctx.clone();
                        #[cfg(feature = "tls-rustls")]
                        let tls_acceptor = tls_acceptor.clone();
                        let mut service_factory = service_factory.clone();

                        let connection_fut = async move {
                            // Perform the TLS handshake if the acceptor is set
                            #[cfg(feature = "tls-rustls")]
                            if let Some(tls_acceptor) = tls_acceptor {
                                #[cfg(feature = "tracing")]
                                tracing::trace!("accepting tls connection");

                                stream = match stream.try_accept_tls(&tls_acceptor).with_context(&ctx).await {
                                    Some(Ok(stream)) => stream,
                                    Some(Err(_err)) => {
                                        #[cfg(feature = "tracing")]
                                        tracing::warn!(err = %_err, "failed to accept tls connection");
                                        return;
                                    }
                                    None => {
                                        #[cfg(feature = "tracing")]
                                        tracing::trace!("context done, stopping tls acceptor");
                                        return;
                                    }
                                };

                                #[cfg(feature = "tracing")]
                                tracing::trace!("accepted tls connection");
                            }

                            // make a new service
                            let http_service = match service_factory.new_service(addr).await {
                                Ok(service) => service,
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::warn!(err = %_e, "failed to create service");
                                    return;
                                }
                            };

                            #[cfg(feature = "tracing")]
                            tracing::trace!("handling connection");

                            #[cfg(feature = "http1")]
                            let http1 = self.http1_enabled;
                            #[cfg(not(feature = "http1"))]
                            let http1 = false;

                            #[cfg(feature = "http2")]
                            let http2 = self.http2_enabled;
                            #[cfg(not(feature = "http2"))]
                            let http2 = false;

                            let _res = handler::handle_connection::<F, _, _>(ctx, http_service, stream, http1, http2).await;

                            #[cfg(feature = "tracing")]
                            if let Err(e) = _res {
                                tracing::warn!(err = %e, "error handling connection");
                            }

                            #[cfg(feature = "tracing")]
                            tracing::trace!("connection closed");
                        };

                        #[cfg(feature = "tracing")]
                        let connection_fut = connection_fut.instrument(tracing::trace_span!("connection", addr = %addr));

                        tokio::spawn(connection_fut);
                    }

                    #[cfg(feature = "tracing")]
                    tracing::trace!("listener closed");

                    Ok(())
                };

                #[cfg(feature = "tracing")]
                let worker_fut = worker_fut.instrument(tracing::trace_span!("worker", n = _n));

                Ok(tokio::spawn(worker_fut))
            })
            .collect::<std::io::Result<Vec<_>>>()?;

        match futures::future::try_join_all(workers).await {
            Ok(res) => {
                for r in res {
                    if let Err(e) = r {
                        drop(worker_ctx);
                        worker_handler.shutdown().await;
                        return Err(e);
                    }
                }
            }
            Err(_e) => {
                #[cfg(feature = "tracing")]
                tracing::error!(err = %_e, "error running workers");
            }
        }

        drop(worker_ctx);
        worker_handler.shutdown().await;

        #[cfg(feature = "tracing")]
        tracing::debug!("all workers finished");

        Ok(())
    }
}
