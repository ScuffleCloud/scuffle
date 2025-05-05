use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto;
use scuffle_context::ContextFutExt;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::error::HttpError;
use crate::service::{HttpService, HttpServiceFactory};

/// Helper function used by hyper server to handle incoming connections.
pub(crate) async fn handle_connection<F, S, I>(
    ctx: scuffle_context::Context,
    service: S,
    io: I,
    http1: bool,
    http2: bool,
) -> Result<(), HttpError<F>>
where
    F: HttpServiceFactory<Service = S>,
    F::Error: std::error::Error,
    S: HttpService + Clone + Send + 'static,
    S::Error: std::error::Error + Send + Sync,
    S::ResBody: Send,
    <S::ResBody as http_body::Body>::Data: Send,
    <S::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let io = TokioIo::new(io);

    let hyper_proxy_service = hyper::service::service_fn(move |req: http::Request<hyper::body::Incoming>| {
        let mut service = service.clone();
        async move {
            let (parts, body) = req.into_parts();
            let body = crate::body::IncomingBody::from(body);
            let req = http::Request::from_parts(parts, body);
            service.call(req).await
        }
    });

    let mut builder = auto::Builder::new(TokioExecutor::new());

    if http1 && http2 {
        #[cfg(feature = "http1")]
        builder.http1().timer(TokioTimer::new());

        #[cfg(feature = "http2")]
        builder.http2().timer(TokioTimer::new());

        builder
            .serve_connection_with_upgrades(io, hyper_proxy_service)
            .with_context(ctx)
            .await
            .transpose()
            .map_err(HttpError::HyperConnection)?;
    } else if http1 {
        #[cfg(not(feature = "http1"))]
        unreachable!("http1 enabled but http1 feature disabled");

        #[cfg(feature = "http1")]
        builder
            .http1_only()
            .serve_connection_with_upgrades(io, hyper_proxy_service)
            .with_context(ctx)
            .await
            .transpose()
            .map_err(HttpError::HyperConnection)?;
    } else if http2 {
        #[cfg(not(feature = "http2"))]
        unreachable!("http2 enabled but http2 feature disabled");

        #[cfg(feature = "http2")]
        builder
            .http2_only()
            .serve_connection_with_upgrades(io, hyper_proxy_service)
            .with_context(ctx)
            .await
            .transpose()
            .map_err(HttpError::HyperConnection)?;
    } else {
        #[cfg(feature = "tracing")]
        tracing::warn!("both http1 and http2 are disabled, closing connection");
    }

    Ok(())
}
