use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderName, Method, StatusCode};
use axum::{Extension, Json};
use tinc::TincService;
use tinc::openapi::Server;
use tower_http::cors::{AllowHeaders, CorsLayer, ExposeHeaders};
use tower_http::trace::TraceLayer;

mod stream;

#[derive(Debug)]
pub struct VideoApiSvc<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G> Default for VideoApiSvc<G> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

fn rest_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}

fn grpc_web_cors_layer() -> CorsLayer {
    // https://github.com/timostamm/protobuf-ts/blob/main/MANUAL.md#grpc-web-transport
    let allow_headers = [
        CONTENT_TYPE,
        HeaderName::from_static("x-grpc-web"),
        HeaderName::from_static("grpc-timeout"),
    ]
    .into_iter();
    // .chain(middleware::auth_headers());

    let expose_headers = [
        HeaderName::from_static("grpc-encoding"),
        HeaderName::from_static("grpc-status"),
        HeaderName::from_static("grpc-status-details-bin"),
        HeaderName::from_static("grpc-message"),
    ];

    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::list(allow_headers))
        .expose_headers(ExposeHeaders::list(expose_headers))
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}

impl<G: video_api_traits::Global> scuffle_bootstrap::Service<G> for VideoApiSvc<G> {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        // REST
        let stream_svc_tinc =
            pb::scufflecloud::video::api::v1::stream_service_tinc::StreamServiceTinc::new(VideoApiSvc::<G>::default());

        let mut openapi_schema = stream_svc_tinc.openapi_schema();
        openapi_schema.info.title = "Scuffle Cloud Video API".to_string();
        openapi_schema.info.version = "v1".to_string();
        openapi_schema.servers = Some(vec![Server::new("/v1")]);

        let v1_rest_router = axum::Router::new()
            .route("/openapi.json", axum::routing::get(Json(openapi_schema)))
            .merge(stream_svc_tinc.into_router())
            .layer(rest_cors_layer());

        // gRPC
        let stream_svc =
            pb::scufflecloud::video::api::v1::stream_service_server::StreamServiceServer::new(VideoApiSvc::<G>::default());

        let reflection_v1_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::ANNOTATIONS_PB)
            .build_v1()?;
        let reflection_v1alpha_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::ANNOTATIONS_PB)
            .build_v1alpha()?;

        let mut builder = tonic::service::Routes::builder();
        builder.add_service(stream_svc);
        builder.add_service(reflection_v1_svc);
        builder.add_service(reflection_v1alpha_svc);

        let grpc_router = builder
            .routes()
            .prepare()
            .into_axum_router()
            .layer(tonic_web::GrpcWebLayer::new())
            .layer(grpc_web_cors_layer());

        let mut router = axum::Router::new()
            .nest("/v1", v1_rest_router)
            .merge(grpc_router)
            // .route_layer(axum::middleware::from_fn(crate::middleware::auth::<G>))
            // .layer(geo_ip::middleware::middleware::<G>())
            .layer(TraceLayer::new_for_http())
            .layer(Extension(Arc::clone(&global)))
            .fallback(StatusCode::NOT_FOUND);

        if global.swagger_ui_enabled() {
            router = router.merge(swagger_ui_dist::generate_routes(swagger_ui_dist::ApiDefinition {
                uri_prefix: "/v1/docs",
                api_definition: swagger_ui_dist::OpenApiSource::Uri("/v1/openapi.json"),
                title: Some("Scuffle Cloud Video API v1 Docs"),
            }));
        }

        scuffle_http::HttpServer::builder()
            .tower_make_service_with_addr(router.into_make_service_with_connect_info::<SocketAddr>())
            .bind(global.service_bind())
            .ctx(ctx)
            .build()
            .run()
            .await?;

        Ok(())
    }
}
