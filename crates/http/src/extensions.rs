//! HTTP extensions that this library provides.

use std::ops::Deref;

/// This extension is always present on the request and contains the remote address of the client.
#[derive(Clone, Debug)]
pub struct ClientAddr(pub std::net::SocketAddr);

impl Deref for ClientAddr {
    type Target = std::net::SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This extension is present on the request when the client has provided one or multiple TLS client
/// certificates.
#[derive(Clone, Debug)]
#[cfg(feature = "tls-rustls")]
pub struct ClientIdentity(pub std::sync::Arc<Vec<tokio_rustls::rustls::pki_types::CertificateDer<'static>>>);

#[cfg(feature = "tls-rustls")]
impl Deref for ClientIdentity {
    type Target = std::sync::Arc<Vec<tokio_rustls::rustls::pki_types::CertificateDer<'static>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
