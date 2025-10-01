//! Big binary for scuffle.cloud that contains all services.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use std::sync::Arc;

use anyhow::Context;
use diesel_async::pooled_connection::bb8;
use geo_ip::resolver::GeoIpResolver;
use scuffle_batching::DataLoader;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use tonic::transport::ClientTlsConfig;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod dataloaders;

type EmailClientPb = pb::scufflecloud::email::v1::email_service_client::EmailServiceClient<tonic::transport::Channel>;

struct Global {
    config: config::Config,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
    user_loader: DataLoader<dataloaders::UserLoader>,
    organization_loader: DataLoader<dataloaders::OrganizationLoader>,
    organization_member_by_user_id_loader: DataLoader<dataloaders::OrganizationMemberByUserIdLoader>,
    external_http_client: reqwest::Client,
    webauthn: webauthn_rs::Webauthn,
    open_telemetry: opentelemetry::OpenTelemetry,
    redis: fred::clients::Pool,
    email_service_client: EmailClientPb,
    geoip_resolver: GeoIpResolver,
    aws_ses_req_signer: reqsign::Signer<reqsign::aws::Credential>,
    mtls_root_cert: Vec<u8>,
    mtls_core_cert: Vec<u8>,
    mtls_core_private_key: Vec<u8>,
    mtls_email_cert: Vec<u8>,
    mtls_email_private_key: Vec<u8>,
}

impl scuffle_signal::SignalConfig for Global {}

impl core_traits::ConfigInterface for Global {
    fn dashboard_origin(&self) -> &url::Url {
        &self.config.dashboard_origin
    }

    fn email_from_name(&self) -> &str {
        &self.config.email_from_name
    }

    fn email_from_address(&self) -> &str {
        &self.config.email_from_address
    }

    fn google_oauth2_config(&self) -> core_traits::GoogleOAuth2Config<'_> {
        core_traits::GoogleOAuth2Config {
            client_id: self.config.google_oauth2.client_id.as_str().into(),
            client_secret: self.config.google_oauth2.client_secret.as_str().into(),
        }
    }

    fn service_bind(&self) -> std::net::SocketAddr {
        self.config.core_bind
    }

    fn swagger_ui_enabled(&self) -> bool {
        self.config.swagger_ui
    }

    fn timeout_config(&self) -> core_traits::TimeoutConfig {
        core_traits::TimeoutConfig {
            new_user_email_request: self.config.timeouts.new_user_email_request,
            magic_link_request: self.config.timeouts.magic_link_request,
            max_request: self.config.timeouts.max_request_lifetime,
            mfa: self.config.timeouts.mfa,
            user_session: self.config.timeouts.user_session,
            user_session_request: self.config.timeouts.user_session_request,
            user_session_token: self.config.timeouts.user_session_token,
        }
    }

    fn turnstile_secret_key(&self) -> &str {
        &self.config.turnstile_secret_key
    }
}

impl core_traits::DatabaseInterface for Global {
    type Connection<'a>
        = diesel_async::pooled_connection::bb8::PooledConnection<'a, diesel_async::pg::AsyncPgConnection>
    where
        Self: 'a;

    async fn db(&self) -> anyhow::Result<Self::Connection<'_>> {
        self.database.get().await.context("failed to get database connection")
    }
}

#[allow(refining_impl_trait)]
impl core_traits::DataloaderInterface for Global {
    fn organization_loader(&self) -> &DataLoader<dataloaders::OrganizationLoader> {
        &self.organization_loader
    }

    fn user_loader(&self) -> &DataLoader<dataloaders::UserLoader> {
        &self.user_loader
    }

    fn organization_member_by_user_id_loader(&self) -> &DataLoader<dataloaders::OrganizationMemberByUserIdLoader> {
        &self.organization_member_by_user_id_loader
    }
}

impl core_traits::HttpClientInterface for Global {
    fn external_http_client(&self) -> &reqwest::Client {
        &self.external_http_client
    }
}

impl geo_ip::GeoIpInterface for Global {
    fn geo_ip_resolver(&self) -> &GeoIpResolver {
        &self.geoip_resolver
    }

    fn reverse_proxy_config(&self) -> Option<geo_ip::ReverseProxyConfig<'_>> {
        let config = self.config.reverse_proxy.as_ref()?;
        Some(geo_ip::ReverseProxyConfig {
            internal_networks: config.internal_networks.as_slice().into(),
            ip_header: config.ip_header.as_str().into(),
            trusted_proxies: config.trusted_proxies.as_slice().into(),
        })
    }
}

impl core_traits::EmailInterface for Global {
    fn email_service(&self) -> impl core_traits::EmailServiceClient {
        struct EmailServiceClient<'a>(&'a EmailClientPb);

        impl core_traits::EmailServiceClient for EmailServiceClient<'_> {
            fn send_email(
                &self,
                email: impl tonic::IntoRequest<pb::scufflecloud::email::v1::SendEmailRequest>,
            ) -> impl Future<Output = Result<tonic::Response<()>, tonic::Status>> + Send {
                let email = email.into_request();
                let mut client = self.0.clone();
                async move { client.send_email(email).await }
            }
        }

        EmailServiceClient(&self.email_service_client)
    }
}

impl core_traits::RedisInterface for Global {
    type RedisConnection<'a>
        = fred::clients::Pool
    where
        Self: 'a;

    fn redis(&self) -> &Self::RedisConnection<'_> {
        &self.redis
    }
}

impl core_traits::WebAuthnInterface for Global {
    fn webauthn(&self) -> &webauthn_rs::Webauthn {
        &self.webauthn
    }
}

impl core_traits::MtlsInterface for Global {
    fn mtls_root_cert_pem(&self) -> &[u8] {
        &self.mtls_root_cert
    }

    fn mtls_cert_pem(&self) -> &[u8] {
        &self.mtls_core_cert
    }

    fn mtls_private_key_pem(&self) -> &[u8] {
        &self.mtls_core_private_key
    }
}

impl core_traits::Global for Global {}

impl email_traits::ConfigInterface for Global {
    fn service_bind(&self) -> std::net::SocketAddr {
        self.config.email_bind
    }
}

impl email_traits::AwsInterface for Global {
    fn aws_region(&self) -> &str {
        &self.config.aws.region
    }

    fn aws_ses_req_signer(&self) -> &reqsign::Signer<reqsign::aws::Credential> {
        &self.aws_ses_req_signer
    }
}

impl email_traits::HttpClientInterface for Global {
    fn external_http_client(&self) -> &reqwest::Client {
        &self.external_http_client
    }
}

impl email_traits::MtlsInterface for Global {
    fn mtls_root_cert_pem(&self) -> &[u8] {
        &self.mtls_root_cert
    }

    fn mtls_cert_pem(&self) -> &[u8] {
        &self.mtls_email_cert
    }

    fn mtls_private_key_pem(&self) -> &[u8] {
        &self.mtls_email_private_key
    }
}

impl email_traits::Global for Global {}

impl scuffle_bootstrap_telemetry::TelemetryConfig for Global {
    fn enabled(&self) -> bool {
        self.config.telemetry.is_some()
    }

    fn bind_address(&self) -> Option<std::net::SocketAddr> {
        self.config.telemetry.as_ref().map(|telemetry| telemetry.bind)
    }

    fn http_server_name(&self) -> &str {
        "scufflecloud-telemetry"
    }

    fn opentelemetry(&self) -> Option<&opentelemetry::OpenTelemetry> {
        Some(&self.open_telemetry)
    }
}

impl scuffle_bootstrap::Global for Global {
    type Config = config::Config;

    async fn init(config: Self::Config) -> anyhow::Result<Arc<Self>> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(config.level.parse()?)),
            )
            .init();

        if rustls::crypto::aws_lc_rs::default_provider().install_default().is_err() {
            anyhow::bail!("failed to install aws-lc-rs as default TLS provider");
        }

        let maxminddb_data = tokio::fs::read(&config.maxminddb_path)
            .await
            .context("failed to read maxmind db path")?;
        let geoip_resolver = GeoIpResolver::new_from_data(maxminddb_data).context("failed to parse maxmind db")?;

        // TODO: Remove mTLS from this binary once we don't use a real connection anymore.
        // mTLS
        let root_cert = std::fs::read(&config.mtls.root_cert_path).context("failed to read mTLS root cert file")?;
        let core_cert = std::fs::read(&config.mtls.core_cert_path).context("failed to read core mTLS cert file")?;
        let core_private_key =
            std::fs::read(&config.mtls.core_key_path).context("failed to read core mTLS private key file")?;
        let email_cert = std::fs::read(&config.mtls.email_cert_path).context("failed to read email mTLS cert file")?;
        let email_private_key =
            std::fs::read(&config.mtls.email_key_path).context("failed to read email mTLS private key file")?;

        let client_tls_config = ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(&root_cert))
            .identity(tonic::transport::Identity::from_pem(&core_cert, &core_private_key));

        let email_service_address = format!("http://{}", config.email_bind);
        // Connect lazily because the service isn't up yet.
        let email_service_channel = tonic::transport::Endpoint::from_shared(email_service_address)
            .context("create channel to email service")?
            .tls_config(client_tls_config)
            .context("configure TLS for email service channel")?
            .connect_lazy();
        let email_service_client =
            pb::scufflecloud::email::v1::email_service_client::EmailServiceClient::new(email_service_channel);

        let Some(db_url) = config.db_url.as_deref() else {
            anyhow::bail!("DATABASE_URL is not set");
        };

        tracing::info!(db_url = config.db_url, "creating database connection pool");

        let database = bb8::Pool::builder()
            .build(diesel_async::pooled_connection::AsyncDieselConnectionManager::new(db_url))
            .await
            .context("build database pool")?;

        let user_loader = dataloaders::UserLoader::new(database.clone());
        let organization_loader = dataloaders::OrganizationLoader::new(database.clone());
        let organization_member_by_user_id_loader = dataloaders::OrganizationMemberByUserIdLoader::new(database.clone());

        // TODO: find someway to restrict this client to only making requests to external ips.
        // likely via dns.
        let external_http_client = reqwest::Client::builder()
            .user_agent(&config.service_name)
            .tls_built_in_root_certs(true)
            .use_rustls_tls()
            .build()
            .context("create HTTP client")?;

        let webauthn = webauthn_rs::WebauthnBuilder::new(&config.rp_id, &config.dashboard_origin)
            .context("build webauthn")?
            .allow_subdomains(true)
            .timeout(config.timeouts.mfa)
            .build()
            .context("initialize webauthn")?;

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = crate::opentelemetry::OpenTelemetry::new()
            .with_traces(tracer)
            .with_logs(logger);

        let redis = config.redis.setup().await?;

        let provider = reqsign::aws::StaticCredentialProvider::new(&config.aws.access_key_id, &config.aws.secret_access_key);
        let signer = reqsign::aws::RequestSigner::new("ses", &config.aws.region);
        let aws_ses_req_signer = reqsign::Signer::new(reqsign::Context::new(), provider, signer);

        Ok(Arc::new(Self {
            config,
            database,
            user_loader,
            organization_loader,
            organization_member_by_user_id_loader,
            external_http_client,
            webauthn,
            open_telemetry,
            redis,
            email_service_client,
            geoip_resolver,
            aws_ses_req_signer,
            mtls_root_cert: root_cert,
            mtls_core_cert: core_cert,
            mtls_core_private_key: core_private_key,
            mtls_email_cert: email_cert,
            mtls_email_private_key: email_private_key,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::services::CoreSvc::<Global>::default(),
        email::services::EmailSvc::<Global>::default(),
    }
}
