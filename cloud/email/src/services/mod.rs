use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::{Extension, Json};
use scuffle_http::http::Method;
use tinc::TincService;
use tinc::openapi::Server;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::EmailConfig;

mod email;

#[derive(Debug)]
pub struct EmailSvc<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G> Default for EmailSvc<G> {
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

impl<G: EmailConfig> scuffle_bootstrap::Service<G> for EmailSvc<G> {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        // REST
        let email_svc_tinc =
            pb::scufflecloud::email::v1::email_service_tinc::EmailServiceTinc::new(EmailSvc::<G>::default());

        let mut openapi_schema = email_svc_tinc.openapi_schema();
        openapi_schema.info.title = "Scuffle Cloud Mail API".to_string();
        openapi_schema.info.version = "v1".to_string();
        openapi_schema.servers = Some(vec![Server::new("/v1")]);

        let v1_rest_router = axum::Router::new()
            .route("/openapi.json", axum::routing::get(Json(openapi_schema)))
            .merge(email_svc_tinc.into_router())
            .layer(rest_cors_layer());

        // gRPC
        let email_svc = pb::scufflecloud::email::v1::email_service_server::EmailServiceServer::new(EmailSvc::<G>::default());

        let reflection_v1_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::ANNOTATIONS_PB)
            .build_v1()?;
        let reflection_v1alpha_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::ANNOTATIONS_PB)
            .build_v1alpha()?;

        let mut builder = tonic::service::Routes::builder();
        builder.add_service(email_svc);
        builder.add_service(reflection_v1_svc);
        builder.add_service(reflection_v1alpha_svc);
        let grpc_router = builder.routes().prepare().into_axum_router();

        let router = axum::Router::new()
            .nest("/v1", v1_rest_router)
            .merge(grpc_router)
            .layer(TraceLayer::new_for_http())
            .layer(Extension(Arc::clone(&global)))
            .fallback(StatusCode::NOT_FOUND);

        scuffle_http::HttpServer::builder()
            .tower_make_service_with_addr(router.into_make_service_with_connect_info::<SocketAddr>())
            .bind(global.bind())
            .ctx(ctx)
            .build()
            .run()
            .await?;

        Ok(())
    }
}
