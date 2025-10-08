use std::net::SocketAddr;
use std::sync::Arc;

use axum::Extension;
use axum::http::{HeaderName, StatusCode};
use pb::scufflecloud::core::v1::organization_invitations_service_server::OrganizationInvitationsServiceServer;
use pb::scufflecloud::core::v1::organization_invitations_service_tinc::OrganizationInvitationsServiceTinc;
use pb::scufflecloud::core::v1::organizations_service_server::OrganizationsServiceServer;
use pb::scufflecloud::core::v1::organizations_service_tinc::OrganizationsServiceTinc;
use pb::scufflecloud::core::v1::sessions_service_server::SessionsServiceServer;
use pb::scufflecloud::core::v1::sessions_service_tinc::SessionsServiceTinc;
use pb::scufflecloud::core::v1::users_service_server::UsersServiceServer;
use pb::scufflecloud::core::v1::users_service_tinc::UsersServiceTinc;
use reqwest::header::CONTENT_TYPE;
use scuffle_http::http::Method;
use tinc::TincService;
use tinc::openapi::{OpenApi, Server};
use tower_http::cors::{AllowHeaders, CorsLayer, ExposeHeaders};
use tower_http::trace::TraceLayer;

mod organization_invitations;
mod organizations;
mod sessions;
mod users;

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

pub struct CoreSvc<G>(std::marker::PhantomData<G>);

impl<G> std::fmt::Debug for CoreSvc<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CoreSvc")
    }
}

impl<G> Clone for CoreSvc<G> {
    fn clone(&self) -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<G> Copy for CoreSvc<G> {}

impl<G> Default for CoreSvc<G> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<G: core_traits::Global> CoreSvc<G> {
    pub fn openapi_schema(self: &Arc<Self>) -> OpenApi {
        let mut openapi_schema = OpenApi::default();
        openapi_schema.merge(OrganizationInvitationsServiceTinc::from_arc(self.clone()).openapi_schema());
        openapi_schema.merge(OrganizationsServiceTinc::from_arc(self.clone()).openapi_schema());
        openapi_schema.merge(SessionsServiceTinc::from_arc(self.clone()).openapi_schema());
        openapi_schema.merge(UsersServiceTinc::from_arc(self.clone()).openapi_schema());
        openapi_schema.info.title = "Scuffle Cloud Core API".to_string();
        openapi_schema.info.version = "v1".to_string();
        openapi_schema.servers = Some(vec![Server::new("/v1")]);
        openapi_schema
    }

    pub fn grpc_router(self: &Arc<Self>) -> axum::Router {
        let mut builder = tonic::service::Routes::builder();
        builder.add_service(OrganizationInvitationsServiceServer::from_arc(self.clone()));
        builder.add_service(OrganizationsServiceServer::from_arc(self.clone()));
        builder.add_service(SessionsServiceServer::from_arc(self.clone()));
        builder.add_service(UsersServiceServer::from_arc(self.clone()));
        builder
            .routes()
            .prepare()
            .into_axum_router()
            .layer(tonic_web::GrpcWebLayer::new())
            .layer(grpc_web_cors_layer())
    }

    pub fn rest_router(self: &Arc<Self>) -> axum::Router {
        axum::Router::new()
            .merge(OrganizationInvitationsServiceTinc::from_arc(self.clone()).into_router())
            .merge(OrganizationsServiceTinc::from_arc(self.clone()).into_router())
            .merge(SessionsServiceTinc::from_arc(self.clone()).into_router())
            .merge(UsersServiceTinc::from_arc(self.clone()).into_router())
            .layer(rest_cors_layer())
    }
}

impl<G: core_traits::Global> scuffle_bootstrap::Service<G> for CoreSvc<G> {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let svc = Arc::new(Self::new());
        // REST

        let mut openapi_schema = OpenApi::default();
        openapi_schema.merge(organization_invitations_svc_tinc.openapi_schema());
        openapi_schema.merge(organizations_svc_tinc.openapi_schema());
        openapi_schema.merge(sessions_svc_tinc.openapi_schema());
        openapi_schema.merge(users_svc_tinc.openapi_schema());
        openapi_schema.info.title = "Scuffle Cloud Core API".to_string();
        openapi_schema.info.version = "v1".to_string();
        openapi_schema.servers = Some(vec![Server::new("/v1")]);

        let rest_router = axum::Router::new()
            .merge(organization_invitations_svc_tinc.into_router())
            .merge(organizations_svc_tinc.into_router())
            .merge(sessions_svc_tinc.into_router())
            .merge(users_svc_tinc.into_router())
            .layer(rest_cors_layer());

        // gRPC
        let organization_invitations_svc = OrganizationInvitationsServiceServer::from_arc(svc.clone());
        let organizations_svc = OrganizationsServiceServer::from_arc(svc.clone());
        let sessions_svc = SessionsServiceServer::from_arc(svc.clone());
        let users_svc = UsersServiceServer::from_arc(svc.clone());

        let mut builder = tonic::service::Routes::builder();
        builder.add_service(organization_invitations_svc);
        builder.add_service(organizations_svc);
        builder.add_service(sessions_svc);
        builder.add_service(users_svc);

        let grpc_router = builder.routes().prepare().into_axum_router();

        let mut router = axum::Router::new()
            .merge(rest_router)
            .merge(grpc_router)
            .route_layer(axum::middleware::from_fn(crate::middleware::auth::<G>))
            .layer(geo_ip::middleware::middleware::<G>())
            .layer(TraceLayer::new_for_http())
            .layer(Extension(Arc::clone(&global)))
            .layer(tonic_web::GrpcWebLayer::new())
            .layer(grpc_web_cors_layer())
            .fallback(StatusCode::NOT_FOUND);

        if global.swagger_ui_enabled() {
            router = router.merge(swagger_ui_dist::generate_routes(swagger_ui_dist::ApiDefinition {
                uri_prefix: "/v1/docs",
                api_definition: swagger_ui_dist::OpenApiSource::Uri("/v1/openapi.json"),
                title: Some("Scuffle Core v1 Api Docs"),
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
