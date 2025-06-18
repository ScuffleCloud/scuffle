use std::sync::Arc;

use axum::Json;
use scuffle_http::http::Method;
use tinc::TincService;
use tower_http::cors::CorsLayer;

use crate::CoreGlobal;

mod organization_invitations;
mod organizations;
mod sessions;
mod users;

pub struct CoreSvc;

impl<G: CoreGlobal> scuffle_bootstrap::Service<G> for CoreSvc {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let organizations_svc_tinc =
            pb::scufflecloud::core::v1::organizations_service_tinc::OrganizationsServiceTinc::new(CoreSvc);
        let sessions_svc_tinc = pb::scufflecloud::core::v1::sessions_service_tinc::SessionsServiceTinc::new(CoreSvc);
        let users_svc_tinc = pb::scufflecloud::core::v1::users_service_tinc::UsersServiceTinc::new(CoreSvc);

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let mut openapi_schema = organizations_svc_tinc.openapi_schema();
        openapi_schema.merge(sessions_svc_tinc.openapi_schema());
        openapi_schema.merge(users_svc_tinc.openapi_schema());
        openapi_schema.info.title = "Scuffle Cloud Core API".to_string();
        openapi_schema.info.version = "v1".to_string();

        let v1_router = axum::Router::new()
            .route("/openapi.json", axum::routing::get(Json(openapi_schema)))
            .merge(organizations_svc_tinc.into_router())
            .merge(sessions_svc_tinc.into_router())
            .merge(users_svc_tinc.into_router())
            .layer(cors);

        let router = axum::Router::new().nest("/v1", v1_router);

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
