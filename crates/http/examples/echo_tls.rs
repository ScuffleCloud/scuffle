//! A simple HTTP echo server with TLS and HTTP/3.
//!
//! This example demonstrates how to create a simple HTTP server that echoes the request body back to the client.
//!
//! It loads a certificate and private key from the `local` directory and serves the server over HTTPS with HTTP/1, HTTP/2 and HTTP/3.
//!
//! Try with:
//!
//! ```
//! curl --http3-only -X POST -d 'test' https://localhost:8000/
//! ```

use std::{fs, io};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};

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

pub fn get_tls_config() -> io::Result<rustls::ServerConfig> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

    let certs = load_certs("local/fullchain.pem")?;
    let key = load_private_key("local/privkey.pem")?;

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    Ok(server_config)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    // Open certificate file.
    let certfile = fs::File::open(filename).map_err(|e| io::Error::other(format!("failed to open {filename}: {e}")))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    rustls_pemfile::certs(&mut reader).collect()
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    // Open keyfile.
    let keyfile = fs::File::open(filename).map_err(|e| io::Error::other(format!("failed to open {filename}: {e}")))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    rustls_pemfile::private_key(&mut reader)?.ok_or_else(|| io::Error::other(format!("no private key found in {filename}")))
}
