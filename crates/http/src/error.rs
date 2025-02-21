//! Error types.
use std::fmt::Debug;

use crate::service::{HttpService, HttpServiceFactory};

/// An error that can occur when creating or running an HTTP server.
#[derive(Debug, thiserror::Error)]
pub enum Error<F>
where
    F: HttpServiceFactory,
    F::Error: std::error::Error,
    <F::Service as HttpService>::Error: std::error::Error,
    <<F::Service as HttpService>::ResBody as http_body::Body>::Error: std::error::Error,
{
    /// An I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// No initial cipher suite.
    ///
    /// Refer to [`h3_quinn::quinn::crypto::rustls::NoInitialCipherSuite`] for more information.
    #[error("{0}")]
    #[cfg(all(feature = "http3", feature = "tls-rustls"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "http3", feature = "tls-rustls"))))]
    NoInitialCipherSuite(#[from] h3_quinn::quinn::crypto::rustls::NoInitialCipherSuite),
    /// Any h3 error.
    ///
    /// Refer to [`h3::Error`] for more information.
    #[error("h3 error: {0}")]
    #[cfg(feature = "http3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http3")))]
    H3(#[from] h3::Error),
    /// An error that occurred while handling a hyper connection.
    #[error("hyper connection: {0}")]
    #[cfg(any(feature = "http1", feature = "http2"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "http1", feature = "http2"))))]
    HyperConnection(Box<dyn std::error::Error + Send + Sync>),
    /// An error that occurred while handling a quinn connection.
    #[error("quinn connection error: {0}")]
    #[cfg(feature = "http3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http3")))]
    QuinnConnection(#[from] h3_quinn::quinn::ConnectionError),
    /// An error that occurred while calling [`HttpServiceFactory::new_service`].
    #[error("make service error: {0}")]
    ServiceFactoryError(F::Error),
    /// An error that occurred while calling [`HttpService::call`].
    #[error("service error: {0}")]
    ServiceError(<F::Service as HttpService>::Error),
    /// An error that occurred while sending a response body.
    #[error("response body error: {0}")]
    ResBodyError(<<F::Service as HttpService>::ResBody as http_body::Body>::Error),
}
