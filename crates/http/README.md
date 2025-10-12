<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-http
<!-- sync-readme ]] -->

> [!WARNING]
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-http/0.3.2.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-http/0.3.2)
[![crates.io](https://img.shields.io/badge/crates.io-v0.3.2-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-http/0.3.2)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-http/0.3.2.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-http/0.3.2.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
An HTTP server with support for HTTP/1, HTTP/2 and HTTP/3.

It abstracts away [`hyper`](https://crates.io/crates/hyper) and [`h3`](https://crates.io/crates/h3) to provide a rather simple interface for creating and running a server that can handle all three protocols.

See the [examples](./examples) directory for usage examples.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`tracing`** —  Enables tracing support
* **`http1`** *(enabled by default)* —  Enables http1 support
* **`http2`** *(enabled by default)* —  Enabled http2 support
* **`http3`** —  Enables http3 support
* **`webtransport`** —  Enables WebTransport over HTTP/3 support (enables http3)
* **`tls-rustls`** —  Enables tls via rustls
* **`http3-tls-rustls`** —  Alias for \[“http3”, “tls-rustls”\]
* **`tower`** *(enabled by default)* —  Enables tower service support
* **`docs`** —  Enables changelog and documentation of feature flags

### Why do we need this?

This crate is designed to be a simple and easy to use HTTP server that supports HTTP/1, HTTP/2 and HTTP/3.

Currently, there are simply no other crates that provide support for all three protocols with a unified API.
This crate aims to fill that gap.

### Example

The following example demonstrates how to create a simple HTTP server (without TLS) that responds with “Hello, world!” to all requests on port 3000.

````rust
let service = scuffle_http::service::fn_http_service(|req| async move {
    scuffle_http::Response::builder()
        .status(scuffle_http::http::StatusCode::OK)
        .header(scuffle_http::http::header::CONTENT_TYPE, "text/plain")
        .body("Hello, world!".to_string())
});
let service_factory = scuffle_http::service::service_clone_factory(service);

scuffle_http::HttpServer::builder()
    .service_factory(service_factory)
    .bind("[::]:3000".parse().unwrap())
    .build()
    .run()
    .await
    .expect("server failed");
````

#### Missing Features

* Upgrading to websocket connections from HTTP/3 connections (this is usually done via HTTP/1.1 anyway)

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
