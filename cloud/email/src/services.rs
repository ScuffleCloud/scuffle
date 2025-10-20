use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::Extension;
use axum::http::StatusCode;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
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
        let email_svc = email_pb::v1::email_service_server::EmailServiceServer::new(EmailSvc::<G>::default());

        let reflection_v1_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(email_pb::ANNOTATIONS_PB)
            .build_v1()?;
        let reflection_v1alpha_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(email_pb::ANNOTATIONS_PB)
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

        // Internal authentication via mTLS
        let root_cert =
            CertificateDer::from_pem_slice(global.mtls_root_cert_pem()).context("failed to parse mTLS root cert")?;
        let cert = CertificateDer::from_pem_slice(global.mtls_cert_pem()).context("failed to parse mTLS cert")?;
        let private_key =
            PrivateKeyDer::from_pem_slice(global.mtls_private_key_pem()).context("failed to parse mTLS private key")?;

        let mut root_cert_store = rustls::RootCertStore::empty();
        root_cert_store
            .add(root_cert.clone())
            .context("failed to add mTLS root cert to root cert store")?;
        let cert_chain = vec![cert, root_cert];

        let rustls_client_verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_cert_store))
            .build()
            .context("failed to create client cert verifier")?;
        let rustls_server_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(rustls_client_verifier)
            .with_single_cert(cert_chain, private_key)
            .context("failed to create rustls ServerConfig")?;

        scuffle_http::HttpServer::builder()
            .tower_make_service_with_addr(router.into_make_service_with_connect_info::<SocketAddr>())
            .bind(global.service_bind())
            .ctx(ctx)
            .rustls_config(rustls_server_config)
            .build()
            .run()
            .await?;

        Ok(())
    }
}
