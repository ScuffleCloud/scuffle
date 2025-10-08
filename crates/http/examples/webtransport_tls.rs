use std::convert::Infallible;
use std::net::SocketAddr;

use http::{Method, StatusCode};
use scuffle_http as http_srv;
use scuffle_http::service::{fn_http_service, service_clone_factory};

fn assets_path(item: &str) -> std::path::PathBuf {
    if let Some(env) = std::env::var_os("ASSETS_DIR") {
        std::path::PathBuf::from(env).join(item)
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../assets/{item}"))
    }
}

fn rustls_config() -> rustls::ServerConfig {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("failed to install aws lc provider");
    });

    let certfile = std::fs::File::open(assets_path("cert.pem")).expect("cert not found");
    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(certfile))
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to load certs");
    let keyfile = std::fs::File::open(assets_path("key.pem")).expect("key not found");
    let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(keyfile))
        .expect("failed to load key")
        .expect("no key found");

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("failed to build config")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr: SocketAddr = "[::]:4443".parse().unwrap();

    let service = fn_http_service(|req: http_srv::IncomingRequest| async move {
        if req.uri().path() == "/" && req.method() == Method::GET {
            let html = r#"<!doctype html>
            <html>
                <body>
                    <h2>WebTransport demo</h2>
                    <script>
                        (async () => {
                        if (!('WebTransport' in window)) {
                            document.body.insertAdjacentHTML('beforeend', '<p>WebTransport not supported</p>');
                            return;
                        }
                        try {
                            const wt = new WebTransport('https://' + location.host + '/wt');
                            await wt.ready;
                            console.log('WT connected');
                            document.body.insertAdjacentHTML('beforeend', '<p>WebTransport connected</p>');

                            const uniWriter = await wt.createUnidirectionalStream();
                            const encoder = new TextEncoder();
                            await uniWriter.write(encoder.encode('Hello from uni stream'));
                            await uniWriter.close();
                            document.body.insertAdjacentHTML('beforeend', '<p>Sent uni stream</p>');

                            const { readable, writable } = await wt.createBidirectionalStream();
                            const writer = writable.getWriter();
                            await writer.write(encoder.encode('Hello from bidi stream'));
                            await writer.close();
                            document.body.insertAdjacentHTML('beforeend', '<p>Sent bidi stream</p>');

                            const reader = readable.getReader();
                            const { value, done } = await reader.read();
                            if (!done && value) {
                                const response = new TextDecoder().decode(value);
                                console.log('Received:', response);
                                document.body.insertAdjacentHTML('beforeend', '<p>Received: ' + response + '</p>');
                            }
                        } catch (e) {
                            console.error(e);
                            document.body.insertAdjacentHTML('beforeend', '<p>WebTransport failed: ' + e + '</p>');
                        }
                        })();
                    </script>
                </body>
            </html>
            "#;
            let resp = http::Response::builder()
                .status(StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(html.to_string())
                .unwrap();
            Ok::<_, Infallible>(resp)
        } else if req.uri().path() == "/wt" && req.method() == Method::CONNECT {
            Ok::<_, Infallible>(http::Response::builder().status(StatusCode::NO_CONTENT).body(String::new()).unwrap())
        } else {
            Ok::<_, Infallible>(http::Response::builder().status(StatusCode::NOT_FOUND).body(String::new()).unwrap())
        }
    });

    let server = http_srv::HttpServer::builder()
        .service_factory(service_clone_factory(service))
        .bind(addr)
        .rustls_config(rustls_config())
        .enable_http3(true)
        .build();

    tracing::info!(%addr, "serving WebTransport demo over TLS (HTTP/3)");
    if let Err(e) = server.run().await {
        eprintln!("server error: {e}");
    }
}
