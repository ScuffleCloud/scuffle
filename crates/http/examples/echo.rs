//! A simple HTTP echo server.
//!
//! This example demonstrates how to create a simple HTTP server that echoes the request body back to the client.
//!
//! Try with:
//!
//! ```
//! curl -X POST -d 'test' http://localhost:8000/
//! ```

#[tokio::main]
async fn main() {
    let service = scuffle_http::service::fn_http_service(|req| async move {
        scuffle_http::Response::builder()
            .status(scuffle_http::http::StatusCode::OK)
            .body(req.into_body())
    });
    // The simplest option here is a clone factory that clones the given service for each connection.
    let service_factory = scuffle_http::service::service_clone_factory(service);

    // Create a server that listens on all interfaces on port 8000.
    // By default, the server supports unencrypted HTTP/1 and HTTP/2. (no HTTPS)
    // For an HTTPS example, see the `scuffle-http-echo-tls` example.
    scuffle_http::HttpServer::builder()
        .service_factory(service_factory)
        .bind("[::]:8000".parse().unwrap())
        .build()
        .run()
        .await
        .expect("server failed");
}
