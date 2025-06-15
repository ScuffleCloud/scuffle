use std::sync::Arc;

use axum::Json;
use scuffle_http::http::Method;
use tinc::TincService;
use tower_http::cors::CorsLayer;

use crate::CoreGlobal;

mod sessions;

pub struct CoreSvc;

impl<G: CoreGlobal> scuffle_bootstrap::Service<G> for CoreSvc {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let sessions_svc_tinc = pb::scufflecloud::core::v1::sessions_service_tinc::SessionsServiceTinc::new(CoreSvc);

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let openapi_spec = sessions_svc_tinc.openapi_schema();

        let router = axum::Router::new()
            .route("/openapi", axum::routing::get(Json(openapi_spec)))
            .merge(sessions_svc_tinc.into_router())
            .layer(cors);

        scuffle_http::HttpServer::builder()
            .tower_make_service_factory(router.into_make_service())
            .bind(global.bind())
            .ctx(ctx)
            .build()
            .run()
            .await?;

        Ok(())
    }
}
