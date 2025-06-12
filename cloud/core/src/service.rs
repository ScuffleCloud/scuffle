use std::sync::Arc;

use axum::Json;
use scuffle_http::http::Method;
use tinc::TincService;
use tower_http::cors::CorsLayer;

use crate::CoreGlobal;

pub struct CoreSvc;

impl<G: CoreGlobal> scuffle_bootstrap::Service<G> for CoreSvc {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let registation_svc_tinc =
            pb::scufflecloud::core::v1::registration_service_tinc::RegistrationServiceTinc::new(CoreSvc);
        let sessions_svc_tinc = pb::scufflecloud::core::v1::sessions_service_tinc::SessionsServiceTinc::new(CoreSvc);

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let mut openapi_spec = registation_svc_tinc.openapi_schema();
        openapi_spec.merge(sessions_svc_tinc.openapi_schema());

        let router = axum::Router::new()
            .route("/openapi", axum::routing::get(Json(openapi_spec)))
            .merge(registation_svc_tinc.into_router())
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

#[async_trait::async_trait]
impl pb::scufflecloud::core::v1::registration_service_server::RegistrationService for CoreSvc {
    async fn register_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_register_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::SessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn register_with_external_provider(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::RegisterWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::RegisterWithExternalProviderResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_register_with_external_provider(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::SessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}

#[async_trait::async_trait]
impl pb::scufflecloud::core::v1::sessions_service_server::SessionsService for CoreSvc {
    async fn create_session(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CreateSessionRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateSessionResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
