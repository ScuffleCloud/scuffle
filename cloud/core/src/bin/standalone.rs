use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use diesel_async::pooled_connection::bb8;
use fred::prelude::ClientLike;
use scuffle_batching::DataLoader;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use scufflecloud_core::dataloaders::{OrganizationLoader, OrganizationMemberByUserIdLoader, UserLoader};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub struct Config {
    #[default(env!("CARGO_PKG_NAME").to_string())]
    pub service_name: String,
    #[default(SocketAddr::from(([127, 0, 0, 1], 3000)))]
    pub bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    #[default(None)]
    pub db_url: Option<String>,
    #[default(false)]
    pub swagger_ui: bool,
    #[default = "scuffle.cloud"]
    pub rp_id: String,
    #[default(url::Url::from_str("https://dashboard.scuffle.cloud").unwrap())]
    pub dashboard_origin: url::Url,
    #[default = "1x0000000000000000000000000000000AA"]
    pub turnstile_secret_key: String,
    pub timeouts: TimeoutConfig,
    pub google_oauth2: GoogleOAuth2Config,
    pub telemetry: Option<TelemetryConfig>,
    pub redis: RedisConfig,
    #[default = "no-reply@scuffle.cloud"]
    pub email_from_address: String,
    #[default("http://localhost:4000".to_string())]
    pub email_service_address: String,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub struct TimeoutConfig {
    #[default(chrono::Duration::minutes(2))]
    pub max_request_lifetime: chrono::Duration,
    #[default(chrono::Duration::days(30))]
    pub user_session: chrono::Duration,
    #[default(chrono::Duration::minutes(5))]
    pub mfa: chrono::Duration,
    #[default(chrono::Duration::hours(4))]
    pub user_session_token: chrono::Duration,
    #[default(chrono::Duration::hours(1))]
    pub email_registration_request: chrono::Duration,
    #[default(chrono::Duration::minutes(5))]
    pub user_session_request: chrono::Duration,
    #[default(chrono::Duration::minutes(15))]
    pub magic_link_user_session_request: chrono::Duration,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct GoogleOAuth2Config {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct RedisConfig {
    #[default(vec!["localhost:6379".to_string()])]
    pub servers: Vec<String>,
    #[default(None)]
    pub username: Option<String>,
    #[default(None)]
    pub password: Option<String>,
    #[default(0)]
    pub database: u8,
    #[default(10)]
    pub max_connections: usize,
    #[default(10)]
    pub pool_size: usize,
}

fn parse_server(server: &str) -> anyhow::Result<fred::types::config::Server> {
    let port_ip = server.split(':').collect::<Vec<_>>();

    if port_ip.len() == 1 {
        Ok(fred::types::config::Server::new(port_ip[0], 6379))
    } else {
        Ok(fred::types::config::Server::new(
            port_ip[0],
            port_ip[1].parse::<u16>().context("invalid port")?,
        ))
    }
}

impl RedisConfig {
    async fn setup(&self) -> anyhow::Result<fred::clients::Pool> {
        let redis_server_config = if self.servers.len() == 1 {
            fred::types::config::ServerConfig::Centralized {
                server: parse_server(&self.servers[0])?,
            }
        } else {
            fred::types::config::ServerConfig::Clustered {
                hosts: self
                    .servers
                    .iter()
                    .map(|s| parse_server(s))
                    .collect::<anyhow::Result<Vec<_>>>()?,
                policy: Default::default(),
            }
        };

        tracing::info!(config = ?redis_server_config, "connecting to redis");

        let config = fred::types::config::Config {
            server: redis_server_config,
            database: Some(self.database),
            fail_fast: true,
            password: self.password.clone(),
            username: self.username.clone(),
            ..Default::default()
        };

        let client = fred::clients::Pool::new(config, None, None, None, self.max_connections).context("redis pool")?;
        client.init().await?;

        Ok(client)
    }
}

scuffle_settings::bootstrap!(Config);

struct Global {
    config: Config,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
    user_loader: DataLoader<UserLoader>,
    organization_loader: DataLoader<OrganizationLoader>,
    organization_member_by_user_id_loader: DataLoader<OrganizationMemberByUserIdLoader>,
    authorizer: cedar_policy::Authorizer,
    http_client: reqwest::Client,
    webauthn: webauthn_rs::Webauthn,
    open_telemetry: opentelemetry::OpenTelemetry,
    redis: fred::clients::Pool,
    email_service_client: pb::scufflecloud::email::v1::email_service_client::EmailServiceClient<tonic::transport::Channel>,
}

impl scufflecloud_core::CoreConfig for Global {
    fn service_name(&self) -> &str {
        &self.config.service_name
    }

    fn bind(&self) -> std::net::SocketAddr {
        self.config.bind
    }

    async fn db(&self) -> anyhow::Result<bb8::PooledConnection<'static, diesel_async::AsyncPgConnection>> {
        self.database.get_owned().await.context("get database connection")
    }

    fn authorizer(&self) -> &cedar_policy::Authorizer {
        &self.authorizer
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    fn webauthn(&self) -> &webauthn_rs::Webauthn {
        &self.webauthn
    }

    fn redis(&self) -> &fred::clients::Pool {
        &self.redis
    }

    fn email_service(
        &self,
    ) -> pb::scufflecloud::email::v1::email_service_client::EmailServiceClient<tonic::transport::Channel> {
        self.email_service_client.clone() // Cloning the client is cheap and recommended by tonic
    }

    fn user_loader(&self) -> &DataLoader<UserLoader> {
        &self.user_loader
    }

    fn organization_loader(&self) -> &DataLoader<OrganizationLoader> {
        &self.organization_loader
    }

    fn organization_member_by_user_id_loader(&self) -> &DataLoader<OrganizationMemberByUserIdLoader> {
        &self.organization_member_by_user_id_loader
    }

    fn swagger_ui_enabled(&self) -> bool {
        self.config.swagger_ui
    }

    fn dashboard_origin(&self) -> &url::Url {
        &self.config.dashboard_origin
    }

    fn turnstile_secret_key(&self) -> &str {
        &self.config.turnstile_secret_key
    }

    fn max_request_lifetime(&self) -> chrono::Duration {
        self.config.timeouts.max_request_lifetime
    }

    fn user_session_timeout(&self) -> chrono::Duration {
        self.config.timeouts.user_session
    }

    fn mfa_timeout(&self) -> chrono::Duration {
        self.config.timeouts.mfa
    }

    fn user_session_token_timeout(&self) -> chrono::Duration {
        self.config.timeouts.user_session_token
    }

    fn email_registration_request_timeout(&self) -> chrono::Duration {
        self.config.timeouts.email_registration_request
    }

    fn user_session_request_timeout(&self) -> chrono::Duration {
        self.config.timeouts.user_session_request
    }

    fn magic_link_user_session_request_timeout(&self) -> chrono::Duration {
        self.config.timeouts.magic_link_user_session_request
    }

    fn google_client_id(&self) -> &str {
        &self.config.google_oauth2.client_id
    }

    fn google_client_secret(&self) -> &str {
        &self.config.google_oauth2.client_secret
    }

    fn email_from_address(&self) -> &str {
        &self.config.email_from_address
    }
}

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

        let http_client = reqwest::Client::builder()
            .user_agent(&config.service_name)
            .build()
            .context("create HTTP client")?;

        let webauthn = webauthn_rs::WebauthnBuilder::new(&config.rp_id, &config.dashboard_origin)
            .context("build webauthn")?
            .allow_subdomains(true)
            .timeout(config.timeouts.mfa.to_std().context("convert mfa timeout to std")?)
            .build()
            .context("initialize webauthn")?;

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = crate::opentelemetry::OpenTelemetry::new()
            .with_traces(tracer)
            .with_logs(logger);

        let redis = config.redis.setup().await?;

        tracing::info!(address = %config.email_service_address, "connecting to email service");
        let email_service_channel = tonic::transport::Channel::from_shared(config.email_service_address.clone())
            .context("create channel to email service")?
            .connect()
            .await
            .context("create channel to email service")?;
        let email_service_client =
            pb::scufflecloud::email::v1::email_service_client::EmailServiceClient::new(email_service_channel);

        Ok(Arc::new(Self {
            config,
            database,
            user_loader,
            organization_loader,
            organization_member_by_user_id_loader,
            authorizer: cedar_policy::Authorizer::new(),
            http_client,
            webauthn,
            open_telemetry,
            redis,
            email_service_client,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::services::CoreSvc::<Global>::default(),
    }
}
