use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::{Extension, Json};
use scuffle_http::http::Method;
use tinc::TincService;
use tinc::openapi::Server;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

use crate::CoreConfig;

mod organization_invitations;
mod organizations;
mod sessions;
mod users;
mod webauthn_challenges;

#[derive(Debug)]
pub struct CoreSvc<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G> Default for CoreSvc<G> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<G: CoreConfig> scuffle_bootstrap::Service<G> for CoreSvc<G> {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        // REST
        let organization_invitations_svc_tinc =
            pb::scufflecloud::core::v1::organization_invitations_service_tinc::OrganizationInvitationsServiceTinc::new(
                CoreSvc::<G>::default(),
            );
        let organizations_svc_tinc =
            pb::scufflecloud::core::v1::organizations_service_tinc::OrganizationsServiceTinc::new(CoreSvc::<G>::default());
        let sessions_svc_tinc =
            pb::scufflecloud::core::v1::sessions_service_tinc::SessionsServiceTinc::new(CoreSvc::<G>::default());
        let users_svc_tinc = pb::scufflecloud::core::v1::users_service_tinc::UsersServiceTinc::new(CoreSvc::<G>::default());
        let webauthn_challenges_svc_tinc =
            pb::scufflecloud::core::v1::webauthn_challenges_service_tinc::WebauthnChallengesServiceTinc::new(
                CoreSvc::<G>::default(),
            );

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let mut openapi_schema = organization_invitations_svc_tinc.openapi_schema();
        openapi_schema.merge(organizations_svc_tinc.openapi_schema());
        openapi_schema.merge(sessions_svc_tinc.openapi_schema());
        openapi_schema.merge(users_svc_tinc.openapi_schema());
        openapi_schema.merge(webauthn_challenges_svc_tinc.openapi_schema());
        openapi_schema.info.title = "Scuffle Cloud Core API".to_string();
        openapi_schema.info.version = "v1".to_string();
        openapi_schema.servers = Some(vec![Server::new("/v1")]);

        let v1_rest_router = axum::Router::new()
            .route("/openapi.json", axum::routing::get(Json(openapi_schema)))
            .merge(organization_invitations_svc_tinc.into_router())
            .merge(organizations_svc_tinc.into_router())
            .merge(sessions_svc_tinc.into_router())
            .merge(users_svc_tinc.into_router())
            .merge(webauthn_challenges_svc_tinc.into_router())
            .layer(cors);

        // gRPC
        let organization_invitations_svc =
            pb::scufflecloud::core::v1::organization_invitations_service_server::OrganizationInvitationsServiceServer::new(
                CoreSvc::<G>::default(),
            );
        let organizations_svc = pb::scufflecloud::core::v1::organizations_service_server::OrganizationsServiceServer::new(
            CoreSvc::<G>::default(),
        );
        let sessions_svc =
            pb::scufflecloud::core::v1::sessions_service_server::SessionsServiceServer::new(CoreSvc::<G>::default());
        let users_svc = pb::scufflecloud::core::v1::users_service_server::UsersServiceServer::new(CoreSvc::<G>::default());
        let webauthn_challenges_svc =
            pb::scufflecloud::core::v1::webauthn_challenges_service_server::WebauthnChallengesServiceServer::new(
                CoreSvc::<G>::default(),
            );

        let reflection_v1_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::ANNOTATIONS_PB)
            .build_v1()?;
        let reflection_v1alpha_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::ANNOTATIONS_PB)
            .build_v1alpha()?;

        let mut builder = tonic::service::Routes::builder();
        builder.add_service(organization_invitations_svc);
        builder.add_service(organizations_svc);
        builder.add_service(sessions_svc);
        builder.add_service(users_svc);
        builder.add_service(webauthn_challenges_svc);
        builder.add_service(reflection_v1_svc);
        builder.add_service(reflection_v1alpha_svc);
        let grpc_router = builder.routes().prepare().into_axum_router();

        let mut router = axum::Router::new()
            .nest("/v1", v1_rest_router)
            .merge(grpc_router)
            .route_layer(axum::middleware::from_fn(crate::middleware::auth::<G>))
            .layer(axum::middleware::from_fn(crate::middleware::ip_address::<G>))
            .layer(TraceLayer::new_for_http())
            .layer(Extension(Arc::clone(&global)))
            .fallback(StatusCode::NOT_FOUND);

        if global.swagger_ui_enabled() {
            router = router.merge(SwaggerUi::new("/v1/docs").config(utoipa_swagger_ui::Config::from("/v1/openapi.json")));
        }

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
