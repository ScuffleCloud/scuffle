use std::sync::Arc;

use tinc::TincService;

use crate::CoreGlobal;

pub struct CoreSvc;

impl<G: CoreGlobal> scuffle_bootstrap::Service<G> for CoreSvc {
    async fn run(self, global: Arc<G>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let tinc = pb::scufflecloud::core::v1::core_service_tinc::CoreServiceTinc::new(CoreSvc);

        scuffle_http::HttpServer::builder()
            .tower_make_service_factory(tinc.into_router().into_make_service())
            .bind(global.bind())
            .ctx(ctx)
            .build()
            .run()
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl pb::scufflecloud::core::v1::core_service_server::CoreService for CoreSvc {
    async fn register_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::SessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("Not implemented yet"))
    }

    async fn register_with_external_provider(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::RegisterWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::RegisterWithExternalProviderResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("Not implemented yet"))
    }
}
