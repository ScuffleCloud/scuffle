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
                    <div id="log"></div>
                    <script>
                        const log = (msg) => {
                            console.log(msg);
                            document.getElementById('log').insertAdjacentHTML('beforeend', '<p>' + msg + '</p>');
                        };

                        (async () => {
                        if (!('WebTransport' in window)) {
                            log('WebTransport not supported');
                            return;
                        }
                        try {
                            const wt = new WebTransport('https://' + location.host + '/wt');
                            await wt.ready;
                            log('WebTransport connected');

                            // Test unidirectional stream
                            const uniWriter = await wt.createUnidirectionalStream();
                            const encoder = new TextEncoder();
                            await uniWriter.write(encoder.encode('Hello from uni stream'));
                            await uniWriter.close();
                            log('Sent uni stream');

                            // Test bidirectional stream with echo
                            const { readable, writable } = await wt.createBidirectionalStream();
                            const writer = writable.getWriter();
                            await writer.write(encoder.encode('Hello from bidi stream'));
                            await writer.close();
                            log('Sent bidi stream');

                            const reader = readable.getReader();
                            const { value, done } = await reader.read();
                            if (!done && value) {
                                const response = new TextDecoder().decode(value);
                                log('Received echo: ' + response);
                            }

                            // Test datagram
                            const dgWriter = wt.datagrams.writable.getWriter();
                            await dgWriter.write(encoder.encode('Hello datagram'));
                            dgWriter.releaseLock();
                            log('Sent datagram');

                            // Listen for incoming datagrams
                            const dgReader = wt.datagrams.readable.getReader();
                            const { value: dgValue, done: dgDone } = await dgReader.read();
                            if (!dgDone && dgValue) {
                                const dgResponse = new TextDecoder().decode(dgValue);
                                log('Received datagram: ' + dgResponse);
                            }

                            log('All tests completed successfully!');
                        } catch (e) {
                            console.error(e);
                            log('WebTransport failed: ' + e);
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
            // Extract the WebTransport session from the request
            if let Some(session) = req.extensions().get::<http_srv::backend::h3::webtransport::WebTransportSession>() {
                let session = session.clone();
                tracing::info!("WebTransport session established");

                // Spawn a task to handle incoming bidirectional streams
                tokio::spawn({
                    let session = session.clone();
                    async move {
                        use http_srv::backend::h3::webtransport::AcceptedBi;
                        while let Some(Ok(accepted)) = session.accept_bi().await {
                            match accepted {
                                AcceptedBi::BidiStream(mut stream) => {
                                    tokio::spawn(async move {
                                        // Echo server: read all data and send it back
                                        match stream.read_to_end(64 * 1024).await {
                                            Ok(data) => {
                                                tracing::info!("Received {} bytes on bidi stream, echoing back", data.len());
                                                if let Err(e) = stream.write(data.clone()).await {
                                                    tracing::warn!("Failed to write to bidi stream: {}", e);
                                                } else if let Err(e) = stream.finish().await {
                                                    tracing::warn!("Failed to finish bidi stream: {}", e);
                                                }
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to read from bidi stream: {}", e);
                                            }
                                        }
                                    });
                                }
                                AcceptedBi::Request(_req, _stream) => {
                                    tracing::info!("Received HTTP request over WebTransport");
                                    // Handle HTTP-over-WebTransport requests if needed
                                }
                            }
                        }
                        tracing::info!("Bidi stream acceptor finished");
                    }
                });

                // Spawn a task to handle incoming unidirectional streams
                tokio::spawn({
                    let session = session.clone();
                    async move {
                        while let Some(Ok((_id, mut stream))) = session.accept_uni().await {
                            tokio::spawn(async move {
                                match stream.read_to_end(64 * 1024).await {
                                    Ok(data) => {
                                        tracing::info!("Received {} bytes on uni stream: {:?}",
                                            data.len(),
                                            String::from_utf8_lossy(&data));
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to read from uni stream: {}", e);
                                    }
                                }
                            });
                        }
                        tracing::info!("Uni stream acceptor finished");
                    }
                });

                // Spawn a task to handle incoming datagrams
                tokio::spawn({
                    let session = session.clone();
                    async move {
                        let mut datagram_reader = session.datagram_reader();
                        let mut datagram_sender = session.datagram_sender();

                        loop {
                            match datagram_reader.read_datagram().await {
                                Ok(datagram) => {
                                    let payload = datagram.into_payload();
                                    tracing::info!("Received datagram: {} bytes", payload.len());
                                    let response = format!("Echo: {}", String::from_utf8_lossy(&payload));
                                    if let Err(e) = datagram_sender.send_datagram(bytes::Bytes::from(response)) {
                                        tracing::warn!("Failed to send datagram response: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to read datagram: {}", e);
                                    break;
                                }
                            }
                        }
                        tracing::info!("Datagram handler finished");
                    }
                });

                return Ok::<_, Infallible>(
                    http::Response::builder()
                        .status(StatusCode::OK)
                        .body(String::new())
                        .unwrap()
                );
            }

            Ok::<_, Infallible>(
                http::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("WebTransport session not found".to_string())
                    .unwrap()
            )
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
