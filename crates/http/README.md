<!-- cargo-sync-rdme title [[ -->
# scuffle-http
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-http.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-http.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-http)
[![crates.io](https://img.shields.io/crates/v/scuffle-http.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-http)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
An HTTP server with support for HTTP/1, HTTP/2 and HTTP/3.

It abstracts away [`hyper`](https://crates.io/crates/hyper) and [`h3`](https://crates.io/crates/h3) to provide a rather simple interface for creating and running a server that can handle all three protocols.

See the [examples](./examples) directory for usage examples.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`tracing`** —  Enables tracing support
* **`http1`** *(enabled by default)* —  Enables http1 support
* **`http2`** *(enabled by default)* —  Enabled http2 support
* **`http3`** —  Enables http3 support
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

* HTTP/3 webtransport support
* Upgrading to websocket connections from HTTP/3 connections (this is usually done via HTTP/1.1 anyway)

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
