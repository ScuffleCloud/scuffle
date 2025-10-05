#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use diesel_async::pooled_connection::bb8;
use geo_ip::resolver::GeoIpResolver;
use scuffle_batching::DataLoader;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use tonic::transport::{ClientTlsConfig, Identity};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::dataloaders::{OrganizationLoader, OrganizationMemberByUserIdLoader, UserLoader};

mod config;
mod dataloaders;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub struct Config {
    #[default(env!("CARGO_PKG_NAME").to_string())]
    pub service_name: String,
    #[default(SocketAddr::from(([127, 0, 0, 1], 3001)))]
    pub bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    #[default(None)]
    pub db_url: Option<String>,
    #[default(false)]
    pub swagger_ui: bool,
    pub webauthn: config::WebauthnConfig,
    pub dashboard_origin: config::DashboardOrigin,
    #[default = "1x0000000000000000000000000000000AA"]
    pub turnstile_secret_key: String,
    pub timeouts: config::TimeoutConfig,
    pub google_oauth2: config::GoogleOAuth2Config,
    pub telemetry: Option<config::TelemetryConfig>,
    pub redis: config::RedisConfig,
    pub mtls: config::MtlsConfig,
    #[default = "Scuffle"]
    pub email_from_name: String,
    #[default = "no-reply@scuffle.cloud"]
    pub email_from_address: String,
    #[default("http://localhost:3002".to_string())]
    pub email_service_address: String,
    pub reverse_proxy: Option<config::ReverseProxyConfig>,
    #[default("./GeoLite2-City.mmdb".parse().unwrap())]
    pub maxminddb_path: PathBuf,
}

scuffle_settings::bootstrap!(Config);

type EmailClientPb = pb::scufflecloud::email::v1::email_service_client::EmailServiceClient<tonic::transport::Channel>;

struct Global {
    config: Config,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
    user_loader: DataLoader<UserLoader>,
    organization_loader: DataLoader<OrganizationLoader>,
    organization_member_by_user_id_loader: DataLoader<OrganizationMemberByUserIdLoader>,
    external_http_client: reqwest::Client,
    webauthn: webauthn_rs::Webauthn,
    open_telemetry: opentelemetry::OpenTelemetry,
    redis: fred::clients::Pool,
    email_service_client: EmailClientPb,
    geoip_resolver: GeoIpResolver,
    mtls_root_cert: Vec<u8>,
    mtls_cert: Vec<u8>,
    mtls_private_key: Vec<u8>,
}

impl core_traits::ConfigInterface for Global {
    fn dashboard_origin(&self) -> Option<&url::Url> {
        match &self.config.dashboard_origin {
            config::DashboardOrigin::Static(url) => Some(url),
            config::DashboardOrigin::FromRequest => None,
        }
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
        self.config.bind
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
    fn organization_loader(&self) -> &scuffle_batching::DataLoader<OrganizationLoader> {
        &self.organization_loader
    }

    fn user_loader(&self) -> &scuffle_batching::DataLoader<UserLoader> {
        &self.user_loader
    }

    fn organization_member_by_user_id_loader(&self) -> &scuffle_batching::DataLoader<OrganizationMemberByUserIdLoader> {
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
        &self.mtls_cert
    }

    fn mtls_private_key_pem(&self) -> &[u8] {
        &self.mtls_private_key
    }
}

impl core_traits::Global for Global {}

impl scuffle_signal::SignalConfig for Global {}

impl scuffle_bootstrap_telemetry::TelemetryConfig for Global {
    fn enabled(&self) -> bool {
        self.config.telemetry.is_some()
    }

    fn bind_address(&self) -> Option<std::net::SocketAddr> {
        self.config.telemetry.as_ref().map(|telemetry| telemetry.bind)
    }

    fn http_server_name(&self) -> &str {
        "scufflecloud-core-telemetry"
    }

    fn opentelemetry(&self) -> Option<&opentelemetry::OpenTelemetry> {
        Some(&self.open_telemetry)
    }
}

impl scuffle_bootstrap::Global for Global {
    type Config = Config;

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

        // mTLS
        let root_cert = std::fs::read(&config.mtls.root_cert_path).context("failed to read mTLS root cert file")?;
        let cert = std::fs::read(&config.mtls.cert_path).context("failed to read mTLS cert file")?;
        let private_key = std::fs::read(&config.mtls.key_path).context("failed to read mTLS private key file")?;

        let client_tls_config = ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(&root_cert))
            .identity(Identity::from_pem(&cert, &private_key));

        tracing::info!(address = %config.email_service_address, "connecting to email service");
        let email_service_channel = tonic::transport::Endpoint::from_shared(config.email_service_address.clone())
            .context("create channel to email service")?
            .tls_config(client_tls_config)
            .context("configure TLS for email service channel")?
            .connect()
            .await
            .context("create channel to email service")?;
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

        let user_loader = UserLoader::new(database.clone());
        let organization_loader = OrganizationLoader::new(database.clone());
        let organization_member_by_user_id_loader = OrganizationMemberByUserIdLoader::new(database.clone());

        // TODO: find someway to restrict this client to only making requests to external ips.
        // likely via dns.
        let external_http_client = reqwest::Client::builder()
            .user_agent(&config.service_name)
            .tls_built_in_root_certs(true)
            .use_rustls_tls()
            .build()
            .context("create HTTP client")?;

        let webauthn = webauthn_rs::WebauthnBuilder::new(&config.webauthn.rp_id, &config.webauthn.rp_origin)
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
            mtls_root_cert: root_cert,
            mtls_cert: cert,
            mtls_private_key: private_key,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::services::CoreSvc::<Global>::default(),
    }
}
