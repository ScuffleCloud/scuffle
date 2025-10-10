use std::sync::Arc;

use anyhow::Context;
use ingest_traits::RtmpConfigInterface;
use scuffle_context::ContextFutExt;

mod rtmp;

#[derive(Debug)]
pub struct IngestSvc<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G> Default for IngestSvc<G> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<G: ingest_traits::Global> scuffle_bootstrap::Service<G> for IngestSvc<G> {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let rtmp = async {
            let addr = global.rtmp_bind();
            tracing::info!(addr = %addr, "starting RTMP server");

            let tcp_listener = tokio::net::TcpListener::bind(addr).await.context("bind TCP listener")?;

            while let Some(connection) = tcp_listener.accept().with_context(&ctx).await {
                match connection {
                    Ok((stream, _addr)) => {
                        let session = scuffle_rtmp::ServerSession::new(stream, rtmp::Handler).with_context(ctx.clone());

                        // This is bound by the context because we pass it to the session.
                        tokio::spawn(async move {
                            if let Err(err) = session.run().await {
                                tracing::error!(err = %err, "RTMP session error");
                                // TODO: what do we do here?
                            }
                        });
                    }
                    Err(err) => {
                        tracing::error!(err = %err, "failed to accept connection");
                        // TODO: what do we do here? can this be fatal?
                    }
                }
            }

            anyhow::Result::Ok(())
        };

        let rtmps = async {
            if let Some(rtmps) = global.rtmps_config() {
                let addr = rtmps.rtmps_bind();
                tracing::info!(addr = %addr, "starting RTMPS server");

                let tcp_listener = tokio::net::TcpListener::bind(addr).await.context("bind TCP listener")?;
                let tls_acceptor = tokio_rustls::TlsAcceptor::from(rtmps.rtmps_rustls_server_config());

                while let Some(connection) = tcp_listener.accept().with_context(&ctx).await {
                    match connection {
                        Ok((stream, _addr)) => {
                            let ctx = ctx.clone();
                            let tls_acceptor = tls_acceptor.clone();

                            tokio::spawn(async move {
                                match tls_acceptor.accept(stream).with_context(&ctx).await {
                                    Some(Ok(stream)) => {
                                        let session =
                                            scuffle_rtmp::ServerSession::new(stream, rtmp::Handler).with_context(ctx);

                                        // run is bound by the context because we pass it to the session.
                                        if let Err(err) = session.run().await {
                                            tracing::error!(err = %err, "RTMP session error");
                                            // TODO: what do we do here?
                                        }
                                    }
                                    Some(Err(err)) => {
                                        tracing::error!(err = %err, "failed to accept TLS connection");
                                    }
                                    None => {} // context cancelled
                                }
                            });
                        }
                        Err(err) => {
                            tracing::error!(err = %err, "failed to accept connection");
                            // TODO: what do we do here? can this be fatal?
                        }
                    }
                }
            }

            anyhow::Result::Ok(())
        };

        let (rtmp, rtmps): (anyhow::Result<()>, anyhow::Result<()>) = tokio::join!(rtmp, rtmps);
        rtmp?;
        rtmps?;

        Ok(())
    }
}
