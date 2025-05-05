//! An HTTP server with support for HTTP/1, HTTP/2 and HTTP/3.
//!
//! It abstracts away [`hyper`](https://crates.io/crates/hyper) and [`h3`](https://crates.io/crates/h3) to provide a rather simple interface for creating and running a server that can handle all three protocols.
//!
//! See the [examples](./examples) directory for usage examples.
//!
//! ## Why do we need this?
//!
//! This crate is designed to be a simple and easy to use HTTP server that supports HTTP/1, HTTP/2 and HTTP/3.
//!
//! Currently, there are simply no other crates that provide support for all three protocols with a unified API.
//! This crate aims to fill that gap.
//!
//! ## Feature Flags
//!
//! - `tower`: Enables support for [`tower`](https://crates.io/crates/tower) services. Enabled by default.
//! - `http1`: Enables support for HTTP/1. Enabled by default.
//! - `http2`: Enables support for HTTP/2. Enabled by default.
//! - `http3`: Enables support for HTTP/3. Disabled by default.
//! - `tracing`: Enables logging with [`tracing`](https://crates.io/crates/tracing). Disabled by default.
//! - `tls-rustls`: Enables support for TLS with [`rustls`](https://crates.io/crates/rustls). Disabled by default.
//! - `http3-tls-rustls`: Enables both `http3` and `tls-rustls` features. Disabled by default.
//!
//! ## Example
//!
//! The following example demonstrates how to create a simple HTTP server (without TLS) that responds with "Hello, world!" to all requests on port 3000.
//!
//! ```rust
//! # use scuffle_future_ext::FutureExt;
//! # tokio_test::block_on(async {
//! # let run = async {
//! let service = scuffle_http::service::fn_http_service(|req| async move {
//!     scuffle_http::Response::builder()
//!         .status(scuffle_http::http::StatusCode::OK)
//!         .header(scuffle_http::http::header::CONTENT_TYPE, "text/plain")
//!         .body("Hello, world!".to_string())
//! });
//! let service_factory = scuffle_http::service::service_clone_factory(service);
//!
//! scuffle_http::HttpServer::builder()
//!     .service_factory(service_factory)
//!     .bind("[::]:3000".parse().unwrap())
//!     .build()
//!     .run()
//!     .await
//!     .expect("server failed");
//! # };
//! # run.with_timeout(std::time::Duration::from_secs(1)).await.expect_err("test should have timed out");
//! # });
//! ```
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! ### Missing Features
//!
//! - HTTP/3 webtransport support
//! - Upgrading to websocket connections from HTTP/3 connections (this is usually done via HTTP/1.1 anyway)
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

#[cfg(all(feature = "http3", not(feature = "tls-rustls")))]
compile_error!("feature \"tls-rustls\" must be enabled when \"http3\" is enabled.");

#[cfg(any(feature = "http1", feature = "http2", feature = "http3"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "http1", feature = "http2", feature = "http3"))))]
pub mod backend;
pub mod body;
pub mod error;
mod server;
pub mod service;

pub use http;
pub use http::Response;
pub use server::{HttpServer, HttpServerBuilder};

/// An incoming request.
pub type IncomingRequest = http::Request<body::IncomingBody>;

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::convert::Infallible;
    use std::time::Duration;

    use scuffle_future_ext::FutureExt;

    use crate::HttpServer;
    use crate::service::{fn_http_service, service_clone_factory};

    fn get_available_addr() -> std::io::Result<std::net::SocketAddr> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        listener.local_addr()
    }

    const RESPONSE_TEXT: &str = "Hello, world!";

    #[allow(dead_code)]
    async fn test_server<F, S>(builder: crate::HttpServerBuilder<F, S>, versions: &[reqwest::Version])
    where
        F: crate::service::HttpServiceFactory + std::fmt::Debug + Clone + Send + 'static,
        F::Error: std::error::Error + Send,
        F::Service: Clone + std::fmt::Debug + Send + 'static,
        <F::Service as crate::service::HttpService>::Error: std::error::Error + Send + Sync,
        <F::Service as crate::service::HttpService>::ResBody: Send,
        <<F::Service as crate::service::HttpService>::ResBody as http_body::Body>::Data: Send,
        <<F::Service as crate::service::HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
        S: crate::server::http_server_builder::State,
        S::ServiceFactory: crate::server::http_server_builder::IsSet,
        S::Bind: crate::server::http_server_builder::IsUnset,
        S::Ctx: crate::server::http_server_builder::IsUnset,
    {
        let addr = get_available_addr().expect("failed to get available address");
        let (ctx, handler) = scuffle_context::Context::new();

        let server = builder.bind(addr).ctx(ctx).build();

        let handle = tokio::spawn(async move {
            server.run().await.expect("server run failed");
        });

        // Wait for the server to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let url = format!("http://{addr}/");

        for version in versions {
            let mut builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

            if *version == reqwest::Version::HTTP_3 {
                builder = builder.http3_prior_knowledge();
            } else if *version == reqwest::Version::HTTP_2 {
                builder = builder.http2_prior_knowledge();
            } else {
                builder = builder.http1_only();
            }

            let client = builder.build().expect("failed to build client");

            let request = client
                .request(reqwest::Method::GET, &url)
                .version(*version)
                .body(RESPONSE_TEXT.to_string())
                .build()
                .expect("failed to build request");

            let resp = client
                .execute(request)
                .await
                .expect("failed to get response")
                .text()
                .await
                .expect("failed to get text");

            assert_eq!(resp, RESPONSE_TEXT);
        }

        handler.shutdown().await;
        handle.await.expect("task failed");
    }

    #[cfg(feature = "tls-rustls")]
    #[allow(dead_code)]
    async fn test_tls_server<F, S>(builder: crate::HttpServerBuilder<F, S>, versions: &[reqwest::Version])
    where
        F: crate::service::HttpServiceFactory + std::fmt::Debug + Clone + Send + 'static,
        F::Error: std::error::Error + Send,
        F::Service: Clone + std::fmt::Debug + Send + 'static,
        <F::Service as crate::service::HttpService>::Error: std::error::Error + Send + Sync,
        <F::Service as crate::service::HttpService>::ResBody: Send,
        <<F::Service as crate::service::HttpService>::ResBody as http_body::Body>::Data: Send,
        <<F::Service as crate::service::HttpService>::ResBody as http_body::Body>::Error: std::error::Error + Send + Sync,
        S: crate::server::http_server_builder::State,
        S::ServiceFactory: crate::server::http_server_builder::IsSet,
        S::Bind: crate::server::http_server_builder::IsUnset,
        S::Ctx: crate::server::http_server_builder::IsUnset,
    {
        let addr = get_available_addr().expect("failed to get available address");
        let (ctx, handler) = scuffle_context::Context::new();

        let server = builder.bind(addr).ctx(ctx).build();

        let handle = tokio::spawn(async move {
            server.run().await.expect("server run failed");
        });

        // Wait for the server to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let url = format!("https://{addr}/");

        for version in versions {
            let mut builder = reqwest::Client::builder().danger_accept_invalid_certs(true).https_only(true);

            if *version == reqwest::Version::HTTP_3 {
                builder = builder.http3_prior_knowledge();
            } else if *version == reqwest::Version::HTTP_2 {
                builder = builder.http2_prior_knowledge();
            } else {
                builder = builder.http1_only();
            }

            let client = builder.build().expect("failed to build client");

            let request = client
                .request(reqwest::Method::GET, &url)
                .version(*version)
                .body(RESPONSE_TEXT.to_string())
                .build()
                .expect("failed to build request");

            let resp = client
                .execute(request)
                .await
                .unwrap_or_else(|_| panic!("failed to get response version {version:?}"))
                .text()
                .await
                .expect("failed to get text");

            assert_eq!(resp, RESPONSE_TEXT);
        }

        handler.shutdown().await;
        handle.await.expect("task failed");
    }

    #[tokio::test]
    #[cfg(feature = "http2")]
    async fn http2_server() {
        let builder = HttpServer::builder().service_factory(service_clone_factory(fn_http_service(|_| async {
            Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
        })));

        #[cfg(feature = "http1")]
        let builder = builder.enable_http1(false);

        test_server(builder, &[reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "http1", feature = "http2"))]
    async fn http12_server() {
        let server = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .enable_http1(true)
            .enable_http2(true);

        test_server(server, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[cfg(feature = "tls-rustls")]
    fn rustls_config() -> rustls::ServerConfig {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("failed to install aws lc provider");

        let certfile = std::fs::File::open("../../assets/cert.pem").expect("cert not found");
        let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(certfile))
            .collect::<Result<Vec<_>, _>>()
            .expect("failed to load certs");
        let keyfile = std::fs::File::open("../../assets/key.pem").expect("key not found");
        let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(keyfile))
            .expect("failed to load key")
            .expect("no key found");

        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("failed to build config")
    }

    #[tokio::test]
    #[cfg(all(feature = "tls-rustls", feature = "http1"))]
    async fn rustls_http1_server() {
        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .rustls_config(rustls_config());

        #[cfg(feature = "http2")]
        let builder = builder.enable_http2(false);

        test_tls_server(builder, &[reqwest::Version::HTTP_11]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "tls-rustls", feature = "http3"))]
    async fn rustls_http3_server() {
        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .rustls_config(rustls_config())
            .enable_http3(true);

        #[cfg(feature = "http2")]
        let builder = builder.enable_http2(false);

        #[cfg(feature = "http1")]
        let builder = builder.enable_http1(false);

        test_tls_server(builder, &[reqwest::Version::HTTP_3]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "tls-rustls", feature = "http1", feature = "http2"))]
    async fn rustls_http12_server() {
        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .rustls_config(rustls_config())
            .enable_http1(true)
            .enable_http2(true);

        test_tls_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "tls-rustls", feature = "http1", feature = "http2", feature = "http3"))]
    async fn rustls_http123_server() {
        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .rustls_config(rustls_config())
            .enable_http1(true)
            .enable_http2(true)
            .enable_http3(true);

        test_tls_server(
            builder,
            &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2, reqwest::Version::HTTP_3],
        )
        .await;
    }

    #[tokio::test]
    async fn no_backend() {
        let addr = get_available_addr().expect("failed to get available address");

        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .bind(addr);

        #[cfg(feature = "http1")]
        let builder = builder.enable_http1(false);

        #[cfg(feature = "http2")]
        let builder = builder.enable_http2(false);

        builder
            .build()
            .run()
            .with_timeout(Duration::from_millis(100))
            .await
            .expect("server timed out")
            .expect("server failed");
    }

    #[tokio::test]
    #[cfg(feature = "tls-rustls")]
    async fn rustls_no_backend() {
        let addr = get_available_addr().expect("failed to get available address");

        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_| async {
                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .rustls_config(rustls_config())
            .bind(addr);

        #[cfg(feature = "http1")]
        let builder = builder.enable_http1(false);

        #[cfg(feature = "http2")]
        let builder = builder.enable_http2(false);

        builder
            .build()
            .run()
            .with_timeout(Duration::from_millis(100))
            .await
            .expect("server timed out")
            .expect("server failed");
    }

    #[tokio::test]
    #[cfg(all(feature = "tower", feature = "http1", feature = "http2"))]
    async fn tower_make_service() {
        let builder = HttpServer::builder()
            .tower_make_service_factory(tower::service_fn(|_| async {
                Ok::<_, Infallible>(tower::service_fn(|_| async move {
                    Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
                }))
            }))
            .enable_http1(true)
            .enable_http2(true);

        test_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "tower", feature = "http1", feature = "http2"))]
    async fn tower_custom_make_service() {
        let builder = HttpServer::builder()
            .custom_tower_make_service_factory(
                tower::service_fn(|target| async move {
                    assert_eq!(target, 42);
                    Ok::<_, Infallible>(tower::service_fn(|_| async move {
                        Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
                    }))
                }),
                42,
            )
            .enable_http1(true)
            .enable_http2(true);

        test_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "tower", feature = "http1", feature = "http2"))]
    async fn tower_make_service_with_addr() {
        use std::net::SocketAddr;

        let builder = HttpServer::builder()
            .tower_make_service_with_addr(tower::service_fn(|addr: SocketAddr| async move {
                assert!(addr.ip().is_loopback());
                Ok::<_, Infallible>(tower::service_fn(|_| async move {
                    Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
                }))
            }))
            .enable_http1(true)
            .enable_http2(true);

        test_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "http1", feature = "http2"))]
    async fn fn_service_factory() {
        use crate::service::fn_http_service_factory;

        let builder = HttpServer::builder()
            .service_factory(fn_http_service_factory(|_| async {
                Ok::<_, Infallible>(fn_http_service(|_| async {
                    Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
                }))
            }))
            .enable_http1(true)
            .enable_http2(true);

        test_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(
        feature = "http1",
        feature = "http2",
        feature = "http3",
        feature = "tls-rustls",
        feature = "tower"
    ))]
    async fn axum_service() {
        let router = axum::Router::new().route(
            "/",
            axum::routing::get(|req: String| async move {
                assert_eq!(req, RESPONSE_TEXT);
                http::Response::new(RESPONSE_TEXT.to_string())
            }),
        );

        let builder = HttpServer::builder()
            .tower_make_service_factory(router.into_make_service())
            .rustls_config(rustls_config())
            .enable_http3(true)
            .enable_http1(true)
            .enable_http2(true);

        test_tls_server(
            builder,
            &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2, reqwest::Version::HTTP_3],
        )
        .await;
    }

    #[tokio::test]
    #[cfg(all(feature = "http1", feature = "http2"))]
    async fn tracked_body() {
        use crate::body::TrackedBody;

        #[derive(Clone)]
        struct TestTracker;

        impl crate::body::Tracker for TestTracker {
            type Error = Infallible;

            fn on_data(&self, size: usize) -> Result<(), Self::Error> {
                assert_eq!(size, RESPONSE_TEXT.len());
                Ok(())
            }
        }

        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|req| async {
                let req = req.map(|b| TrackedBody::new(b, TestTracker));
                let body = req.into_body();
                Ok::<_, Infallible>(http::Response::new(body))
            })))
            .enable_http1(true)
            .enable_http2(true);

        test_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "http1", feature = "http2"))]
    async fn tracked_body_error() {
        use crate::body::TrackedBody;

        #[derive(Clone)]
        struct TestTracker;

        impl crate::body::Tracker for TestTracker {
            type Error = &'static str;

            fn on_data(&self, size: usize) -> Result<(), Self::Error> {
                assert_eq!(size, RESPONSE_TEXT.len());
                Err("test")
            }
        }

        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|req| async {
                let req = req.map(|b| TrackedBody::new(b, TestTracker));
                let body = req.into_body();
                // Use axum to convert the body to bytes
                let bytes = axum::body::to_bytes(axum::body::Body::new(body), usize::MAX).await;
                assert_eq!(bytes.expect_err("expected error").to_string(), "tracker error: test");

                Ok::<_, Infallible>(http::Response::new(RESPONSE_TEXT.to_string()))
            })))
            .enable_http1(true)
            .enable_http2(true);

        test_server(builder, &[reqwest::Version::HTTP_11, reqwest::Version::HTTP_2]).await;
    }

    #[tokio::test]
    #[cfg(all(feature = "http2", feature = "http3", feature = "tls-rustls"))]
    async fn response_trailers() {
        #[derive(Default)]
        struct TestBody {
            data_sent: bool,
        }

        impl http_body::Body for TestBody {
            type Data = bytes::Bytes;
            type Error = Infallible;

            fn poll_frame(
                mut self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
                if !self.data_sent {
                    self.as_mut().data_sent = true;
                    let data = http_body::Frame::data(bytes::Bytes::from_static(RESPONSE_TEXT.as_bytes()));
                    std::task::Poll::Ready(Some(Ok(data)))
                } else {
                    let mut trailers = http::HeaderMap::new();
                    trailers.insert("test", "test".parse().unwrap());
                    std::task::Poll::Ready(Some(Ok(http_body::Frame::trailers(trailers))))
                }
            }
        }

        let builder = HttpServer::builder()
            .service_factory(service_clone_factory(fn_http_service(|_req| async {
                let mut resp = http::Response::new(TestBody::default());
                resp.headers_mut().insert("trailers", "test".parse().unwrap());
                Ok::<_, Infallible>(resp)
            })))
            .rustls_config(rustls_config())
            .enable_http3(true)
            .enable_http2(true);

        #[cfg(feature = "http1")]
        let builder = builder.enable_http1(false);

        test_tls_server(builder, &[reqwest::Version::HTTP_2, reqwest::Version::HTTP_3]).await;
    }
}
