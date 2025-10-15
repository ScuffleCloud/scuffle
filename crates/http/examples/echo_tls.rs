//! A simple HTTP echo server with TLS and HTTP/3.
//!
//! This example demonstrates how to create a simple HTTP server that echoes the request body back to the client.
//!
//! It loads a certificate and private key from the `assets` directory and serves the server over HTTPS with HTTP/1, HTTP/2 and HTTP/3.
//!
//! Try with:
//!
//! ```
//! curl --http3-only -X POST -d 'test' https://localhost:8000/
//! ```

use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{self, CertificateDer, PrivateKeyDer};

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
    scuffle_http::HttpServer::builder()
        .service_factory(service_factory)
        .bind("[::]:8000".parse().unwrap())
        .rustls_config(get_tls_config().expect("failed to load tls config"))
        .enable_http3(true)
        .build()
        .run()
        .await
        .expect("server failed");
}

fn assets_path(item: &str) -> std::path::PathBuf {
    if let Some(env) = std::env::var_os("ASSETS_DIR") {
        std::path::PathBuf::from(env).join(item)
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../assets/{item}"))
    }
}

pub fn get_tls_config() -> Result<tokio_rustls::rustls::ServerConfig, pki_types::pem::Error> {
    let certs = CertificateDer::pem_file_iter(assets_path("server_cert.pem"))?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(assets_path("server_key.pem"))?;

    let server_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    Ok(server_config)
}
