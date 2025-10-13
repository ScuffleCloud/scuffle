use tokio::io::{AsyncRead, AsyncWrite};

/// A stream that can be either a TCP stream or a TLS stream.
///
/// Implements [`AsyncRead`] and [`AsyncWrite`] by delegating to the inner stream.
pub(crate) enum Stream {
    Tcp(tokio::net::TcpStream),
    #[cfg(feature = "tls-rustls")]
    Tls(Box<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>),
}

impl Stream {
    /// Try to upgrade the stream to a TLS stream by using a TLS acceptor.
    ///
    /// If the stream is already a TLS stream, this function will return the stream unchanged.
    #[cfg(feature = "tls-rustls")]
    pub(crate) async fn try_accept_tls(self, tls_acceptor: &tokio_rustls::TlsAcceptor) -> std::io::Result<Self> {
        match self {
            Stream::Tcp(stream) => {
                let stream = tls_acceptor.accept(stream).await?;
                Ok(Self::Tls(Box::new(stream)))
            }
            Stream::Tls(_) => Ok(self),
        }
    }

    /// Get the client certificates from the TLS stream.
    ///
    /// Returns `None` if the stream is not a TLS stream or if no client certificates are present.
    #[cfg(feature = "tls-rustls")]
    pub(crate) fn get_client_certs(&self) -> Option<&[tokio_rustls::rustls::pki_types::CertificateDer<'static>]> {
        match self {
            Stream::Tcp(_) => None,
            Stream::Tls(stream) => stream.get_ref().1.peer_certificates(),
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            #[cfg(feature = "tls-rustls")]
            Stream::Tls(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Stream::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            #[cfg(feature = "tls-rustls")]
            Stream::Tls(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            #[cfg(feature = "tls-rustls")]
            Stream::Tls(stream) => std::pin::Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Tcp(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            #[cfg(feature = "tls-rustls")]
            Stream::Tls(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
        }
    }

    fn poll_write_vectored(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Stream::Tcp(stream) => std::pin::Pin::new(stream).poll_write_vectored(cx, bufs),
            #[cfg(feature = "tls-rustls")]
            Stream::Tls(stream) => std::pin::Pin::new(stream).poll_write_vectored(cx, bufs),
        }
    }

    fn is_write_vectored(&self) -> bool {
        match self {
            Stream::Tcp(stream) => stream.is_write_vectored(),
            #[cfg(feature = "tls-rustls")]
            Stream::Tls(stream) => stream.is_write_vectored(),
        }
    }
}
