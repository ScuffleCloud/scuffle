use std::net::{IpAddr, SocketAddr};

use arti_client::{TorClient, TorClientConfig};
use futures::StreamExt;
use tor_cell::relaycell::msg::Connected;
pub use tor_hsservice::config::{OnionServiceConfig, OnionServiceConfigBuilder};
use tor_proto::stream::IncomingStreamRequest;
#[cfg(feature = "tracing")]
use tracing::Instrument;

use crate::service::{HttpService, HttpServiceFactory};

#[derive(Debug, Clone, bon::Builder)]
pub struct ArtiBackend<F> {
    /// The [`scuffle_context::Context`] this server will live by.
    #[builder(default = scuffle_context::Context::global())]
    ctx: scuffle_context::Context,
    /// The service factory that will be used to create new services.
    service_factory: F,
    #[builder(default = TorClientConfig::default())]
    tor_client_config: TorClientConfig,
    onion_service_config: OnionServiceConfig,
    bind_port: u16,
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

impl<F> ArtiBackend<F>
where
    F: HttpServiceFactory + Clone + Send + 'static,
    F::Error: std::error::Error + Send,
    F::Service: Clone + Send + 'static,
    <F::Service as HttpService>::Error: std::error::Error + Send + Sync,
    <F::Service as HttpService>::ResBody: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Data: Send,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
{
    pub async fn run(self) -> Result<(), crate::error::Error<F>> {
        let client = TorClient::create_bootstrapped(self.tor_client_config).await?;
        let (service, request_stream) = client.launch_onion_service(self.onion_service_config)?;

        #[cfg(feature = "tracing")]
        if let Some(id) = service.onion_name() {
            tracing::info!(onion_address = %id, "onion service started");
        }

        let stream_requests = tor_hsservice::handle_rend_requests(request_stream);
        tokio::pin!(stream_requests);

        #[cfg(feature = "tracing")]
        tracing::debug!("listening for incoming connections");

        while let Some(stream_request) = stream_requests.next().await {
            let ctx = self.ctx.clone();
            let mut service_factory = self.service_factory.clone();

            let connection_fut = async move {
                match stream_request.request() {
                    IncomingStreamRequest::Begin(begin) => {
                        if begin.port() != self.bind_port {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(incoming_port = %begin.port(), bind_port = %self.bind_port, "port mismatch");
                            return;
                        }

                        #[cfg(feature = "tracing")]
                        tracing::trace!("accepting new connection");

                        // workaround
                        let null_addr = SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0);

                        // make a new service
                        let http_service = match service_factory.new_service(null_addr).await {
                            Ok(service) => service,
                            Err(_e) => {
                                #[cfg(feature = "tracing")]
                                tracing::warn!(err = %_e, "failed to create service");
                                return;
                            }
                        };

                        let onion_service_stream = match stream_request.accept(Connected::new_empty()).await {
                            Ok(stream) => stream,
                            Err(_e) => {
                                #[cfg(feature = "tracing")]
                                tracing::warn!(err = %_e, "failed to accept stream");
                                return;
                            }
                        };

                        #[cfg(feature = "http1")]
                        let http1 = self.http1_enabled;
                        #[cfg(not(feature = "http1"))]
                        let http1 = false;

                        #[cfg(feature = "http2")]
                        let http2 = self.http2_enabled;
                        #[cfg(not(feature = "http2"))]
                        let http2 = false;

                        let _res = crate::backend::hyper::handler::handle_connection::<F, _, _>(
                            ctx,
                            http_service,
                            onion_service_stream,
                            http1,
                            http2,
                        )
                        .await;

                        #[cfg(feature = "tracing")]
                        if let Err(e) = _res {
                            tracing::warn!(err = %e, "error handling connection");
                        }

                        #[cfg(feature = "tracing")]
                        tracing::trace!("connection closed");
                    }
                    _ => {
                        #[cfg(feature = "tracing")]
                        tracing::info!("closing circuit");

                        if let Err(_e) = stream_request.shutdown_circuit() {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(err = %_e, "failed to shutdown circuit");
                        }
                    }
                }
            };

            #[cfg(feature = "tracing")]
            let connection_fut = connection_fut.instrument(tracing::info_span!("connection"));

            tokio::spawn(connection_fut);
        }

        Ok(())
    }
}
