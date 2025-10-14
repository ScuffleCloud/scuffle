use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{self, CertificateDer, PrivateKeyDer};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

async fn hello_world(req: Request<axum::body::Body>) -> axum::response::Response<String> {
    tracing::info!("received request: {} {}", req.method(), req.uri());

    let mut resp = axum::response::Response::new("Hello, World!\n".to_string());

    // TODO: this has to be part of the library somehow
    resp.headers_mut()
        .insert("Alt-Svc", "h3=\":4443\"; ma=3600, h2=\":4443\"; ma=3600".parse().unwrap());

    resp
}

async fn ws(ws: axum::extract::ws::WebSocketUpgrade) -> Response<Body> {
    ws.on_upgrade(|mut socket| async move {
        while let Some(msg) = socket.recv().await {
            let msg = msg.unwrap();
            socket.send(msg).await.unwrap();
        }
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let make_service = axum::Router::<()>::new()
        .route("/", axum::routing::get(hello_world))
        .route("/ws", axum::routing::get(ws))
        .into_make_service();

    scuffle_http::HttpServer::builder()
        .rustls_config(get_tls_config().expect("failed to load tls config"))
        .tower_make_service_factory(make_service)
        .bind("[::]:4443".parse().unwrap())
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
