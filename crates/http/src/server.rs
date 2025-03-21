use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::error::Error;
use crate::service::{HttpService, HttpServiceFactory};

/// The HTTP server.
///
/// This struct is the main entry point for creating and running an HTTP server.
///
/// Start creating a new server by calling [`HttpServer::builder`].
#[derive(Debug, Clone, bon::Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
#[allow(dead_code)]
pub struct HttpServer<F> {
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
    /// Enable HTTP/1.1.
    #[builder(default = true)]
    #[cfg(feature = "http1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http1")))]
    enable_http1: bool,
    /// Enable HTTP/2.
    #[builder(default = true)]
    #[cfg(feature = "http2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http2")))]
    enable_http2: bool,
    #[builder(default = false, setters(vis = "", name = enable_http3_internal))]
    #[cfg(feature = "http3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http3")))]
    enable_http3: bool,
    /// Callback to configure socket used for http1 and http2
    #[cfg(any(feature = "http1", feature = "http1"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "http1", feature = "http1"))))]
    configure_h12_sock: Option<ConfigureSocketCallback>,
    /// Callback to configure socket used for http3
    #[cfg(feature = "http3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http3")))]
    configure_h3_sock: Option<ConfigureSocketCallback>,
    /// rustls config.
    ///
    /// Use this field to set the server into TLS mode.
    /// It will only accept TLS connections when this is set.
    #[cfg(feature = "tls-rustls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls-rustls")))]
    rustls_config: Option<rustls::ServerConfig>,
}

#[cfg(feature = "http3")]
#[cfg_attr(docsrs, doc(cfg(feature = "http3")))]
impl<F, S> HttpServerBuilder<F, S>
where
    S: http_server_builder::State,
    S::EnableHttp3: http_server_builder::IsUnset,
    S::RustlsConfig: http_server_builder::IsSet,
{
    /// Enable HTTP/3 support.
    ///
    /// First enable TLS by calling [`rustls_config`](HttpServerBuilder::rustls_config) to enable HTTP/3.
    pub fn enable_http3(self, enable_http3: bool) -> HttpServerBuilder<F, http_server_builder::SetEnableHttp3<S>> {
        self.enable_http3_internal(enable_http3)
    }
}

#[cfg(feature = "tower")]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
impl<M, S> HttpServerBuilder<crate::service::TowerMakeServiceFactory<M, ()>, S>
where
    M: tower::MakeService<(), crate::IncomingRequest> + Send,
    M::Future: Send,
    M::Service: crate::service::HttpService,
    S: http_server_builder::State,
    S::ServiceFactory: http_server_builder::IsUnset,
{
    /// Same as calling `service_factory(tower_make_service_factory(tower_make_service))`.
    ///
    /// # See Also
    ///
    /// - [`service_factory`](HttpServerBuilder::service_factory)
    /// - [`tower_make_service_factory`](crate::service::tower_make_service_factory)
    pub fn tower_make_service_factory(
        self,
        tower_make_service: M,
    ) -> HttpServerBuilder<crate::service::TowerMakeServiceFactory<M, ()>, http_server_builder::SetServiceFactory<S>> {
        self.service_factory(crate::service::tower_make_service_factory(tower_make_service))
    }
}

#[cfg(feature = "tower")]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
impl<M, S> HttpServerBuilder<crate::service::TowerMakeServiceWithAddrFactory<M>, S>
where
    M: tower::MakeService<SocketAddr, crate::IncomingRequest> + Send,
    M::Future: Send,
    M::Service: crate::service::HttpService,
    S: http_server_builder::State,
    S::ServiceFactory: http_server_builder::IsUnset,
{
    /// Same as calling `service_factory(tower_make_service_with_addr_factory(tower_make_service))`.
    ///
    /// # See Also
    ///
    /// - [`service_factory`](HttpServerBuilder::service_factory)
    /// - [`tower_make_service_with_addr_factory`](crate::service::tower_make_service_with_addr_factory)
    pub fn tower_make_service_with_addr(
        self,
        tower_make_service: M,
    ) -> HttpServerBuilder<crate::service::TowerMakeServiceWithAddrFactory<M>, http_server_builder::SetServiceFactory<S>>
    {
        self.service_factory(crate::service::tower_make_service_with_addr_factory(tower_make_service))
    }
}

#[cfg(feature = "tower")]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
impl<M, T, S> HttpServerBuilder<crate::service::TowerMakeServiceFactory<M, T>, S>
where
    M: tower::MakeService<T, crate::IncomingRequest> + Send,
    M::Future: Send,
    M::Service: crate::service::HttpService,
    T: Clone + Send,
    S: http_server_builder::State,
    S::ServiceFactory: http_server_builder::IsUnset,
{
    /// Same as calling `service_factory(custom_tower_make_service_factory(tower_make_service, target))`.
    ///
    /// # See Also
    ///
    /// - [`service_factory`](HttpServerBuilder::service_factory)
    /// - [`custom_tower_make_service_factory`](crate::service::custom_tower_make_service_factory)
    pub fn custom_tower_make_service_factory(
        self,
        tower_make_service: M,
        target: T,
    ) -> HttpServerBuilder<crate::service::TowerMakeServiceFactory<M, T>, http_server_builder::SetServiceFactory<S>> {
        self.service_factory(crate::service::custom_tower_make_service_factory(tower_make_service, target))
    }
}

impl<F> HttpServer<F>
where
    F: HttpServiceFactory + Clone + Send + 'static,
    F::Error: std::error::Error + Send,
    F::Service: Clone + Send + 'static,
    <F::Service as HttpService>::Error: std::error::Error + Send + Sync,
    <F::Service as HttpService>::ResBody: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Data: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
{
    #[cfg(feature = "tls-rustls")]
    fn set_alpn_protocols(&mut self) {
        let Some(rustls_config) = &mut self.rustls_config else {
            return;
        };

        // https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml#alpn-protocol-ids
        if rustls_config.alpn_protocols.is_empty() {
            #[cfg(feature = "http1")]
            if self.enable_http1 {
                rustls_config.alpn_protocols.push(b"http/1.0".to_vec());
                rustls_config.alpn_protocols.push(b"http/1.1".to_vec());
            }

            #[cfg(feature = "http2")]
            if self.enable_http2 {
                rustls_config.alpn_protocols.push(b"h2".to_vec());
                rustls_config.alpn_protocols.push(b"h2c".to_vec());
            }

            #[cfg(feature = "http3")]
            if self.enable_http3 {
                rustls_config.alpn_protocols.push(b"h3".to_vec());
            }
        }
    }

    /// Run the server.
    ///
    /// This will:
    ///
    /// - Start listening on all configured interfaces for incoming connections.
    /// - Accept all incoming connections.
    /// - Handle incoming requests by passing them to the configured service factory.
    pub async fn run(#[allow(unused_mut)] mut self) -> Result<(), Error<F>> {
        #[cfg(feature = "tls-rustls")]
        self.set_alpn_protocols();

        #[cfg(all(not(any(feature = "http1", feature = "http2")), feature = "tls-rustls"))]
        let start_tcp_backend = false;
        #[cfg(all(feature = "http1", not(feature = "http2")))]
        let start_tcp_backend = self.enable_http1;
        #[cfg(all(not(feature = "http1"), feature = "http2"))]
        let start_tcp_backend = self.enable_http2;
        #[cfg(all(feature = "http1", feature = "http2"))]
        let start_tcp_backend = self.enable_http1 || self.enable_http2;

        #[cfg(feature = "tls-rustls")]
        if let Some(_rustls_config) = self.rustls_config {
            #[cfg(not(feature = "http3"))]
            let enable_http3 = false;
            #[cfg(feature = "http3")]
            let enable_http3 = self.enable_http3;

            match (start_tcp_backend, enable_http3) {
                #[cfg(feature = "http3")]
                (false, true) => {
                    let backend = crate::backend::h3::Http3Backend::builder()
                        .ctx(self.ctx)
                        .worker_tasks(self.worker_tasks)
                        .service_factory(self.service_factory)
                        .bind(self.bind)
                        .rustls_config(_rustls_config)
                        .maybe_configure_sock(self.configure_h3_sock.clone())
                        .build();

                    return backend.run().await;
                }
                #[cfg(any(feature = "http1", feature = "http2"))]
                (true, false) => {
                    let builder = crate::backend::hyper::HyperBackend::builder()
                        .ctx(self.ctx)
                        .worker_tasks(self.worker_tasks)
                        .service_factory(self.service_factory)
                        .bind(self.bind)
                        .maybe_configure_sock(self.configure_h12_sock.clone())
                        .rustls_config(_rustls_config);

                    #[cfg(feature = "http1")]
                    let builder = builder.http1_enabled(self.enable_http1);

                    #[cfg(feature = "http2")]
                    let builder = builder.http2_enabled(self.enable_http2);

                    return builder.build().run().await;
                }
                #[cfg(all(any(feature = "http1", feature = "http2"), feature = "http3"))]
                (true, true) => {
                    let builder = crate::backend::hyper::HyperBackend::builder()
                        .ctx(self.ctx.clone())
                        .worker_tasks(self.worker_tasks)
                        .service_factory(self.service_factory.clone())
                        .bind(self.bind)
                        .maybe_configure_sock(self.configure_h12_sock.clone())
                        .rustls_config(_rustls_config.clone());

                    #[cfg(feature = "http1")]
                    let builder = builder.http1_enabled(self.enable_http1);

                    #[cfg(feature = "http2")]
                    let builder = builder.http2_enabled(self.enable_http2);

                    let hyper = std::pin::pin!(builder.build().run());

                    let mut http3 = crate::backend::h3::Http3Backend::builder()
                        .ctx(self.ctx)
                        .worker_tasks(self.worker_tasks)
                        .service_factory(self.service_factory)
                        .bind(self.bind)
                        .maybe_configure_sock(self.configure_h3_sock.clone())
                        .rustls_config(_rustls_config)
                        .build()
                        .run();
                    let http3 = std::pin::pin!(http3);

                    let res = futures::future::select(hyper, http3).await;
                    match res {
                        futures::future::Either::Left((res, _)) => return res,
                        futures::future::Either::Right((res, _)) => return res,
                    }
                }
                _ => return Ok(()),
            }

            // This line must be unreachable
        }

        // At this point we know that we are not using TLS either
        // - because the feature is disabled
        // - or because it's enabled but the config is None.

        #[cfg(any(feature = "http1", feature = "http2"))]
        if start_tcp_backend {
            let builder = crate::backend::hyper::HyperBackend::builder()
                .ctx(self.ctx)
                .worker_tasks(self.worker_tasks)
                .service_factory(self.service_factory)
                .maybe_configure_sock(self.configure_h12_sock.clone())
                .bind(self.bind);

            #[cfg(feature = "http1")]
            let builder = builder.http1_enabled(self.enable_http1);

            #[cfg(feature = "http2")]
            let builder = builder.http2_enabled(self.enable_http2);

            return builder.build().run().await;
        }

        Ok(())
    }
}

/// A callback used to configure a socket2 instance.
///
/// This can be used to tweak options on the TCP/UDP layer
#[derive(Clone)]
pub struct ConfigureSocketCallback(Arc<dyn Fn(socket2::Socket) -> std::io::Result<socket2::Socket> + Send + Sync>);

impl ConfigureSocketCallback {
    /// Create a new `ConfigureSocketCallback` from the given callback function.
    pub fn new<F: Fn(socket2::Socket) -> std::io::Result<socket2::Socket> + 'static + Send + Sync>(f: F) -> Self {
        Self(Arc::new(f))
    }

    /// Create a new `ConfigureSocketCallback` from the given callback function.
    pub fn call(&self, sock: socket2::Socket) -> std::io::Result<socket2::Socket> {
        (self.0)(sock)
    }
}

impl std::fmt::Debug for ConfigureSocketCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ConfigureSocketCallback ")
    }
}
