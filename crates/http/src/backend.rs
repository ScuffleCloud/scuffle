//! This module contains the underlying backends for the server.
//!
//! You probably don't want to use this module directly and should instead use the [`HttpServer`](crate::HttpServer) struct.

#[cfg(feature = "http3")]
pub mod h3;
#[cfg(any(feature = "http1", feature = "http2"))]
pub mod hyper;
