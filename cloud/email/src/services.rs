use std::net::SocketAddr;
use std::sync::Arc;

use axum::Extension;
use axum::http::StatusCode;
use tower_http::trace::TraceLayer;

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

impl<G: email_traits::Global> scuffle_bootstrap::Service<G> for EmailSvc<G> {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
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
            .merge(grpc_router)
            .layer(TraceLayer::new_for_http())
            .layer(Extension(Arc::clone(&global)))
            .fallback(StatusCode::NOT_FOUND);

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
