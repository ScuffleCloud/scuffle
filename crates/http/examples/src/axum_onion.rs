use axum::http::Request;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

async fn hello_world(req: Request<axum::body::Body>) -> axum::response::Response<String> {
    tracing::info!("received request: {} {}", req.method(), req.uri());
    axum::response::Response::new("Hello, World!\n".to_string())
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
        .into_make_service();

    scuffle_http::HttpServer::builder()
        .service_factory(scuffle_http::service::tower_make_service_factory(make_service))
        .bind("[::]:80".parse().unwrap())
        .onion_service_config(
            scuffle_http::backend::arti::OnionServiceConfigBuilder::default()
                .nickname("test".parse().unwrap())
                .build()
                .unwrap(),
        )
        .build()
        .run()
        .await
        .unwrap();
}
