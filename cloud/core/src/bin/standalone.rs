use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use base64::Engine;
use diesel_async::pooled_connection::bb8;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
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
    #[default = "https://dashboard.scuffle.cloud"]
    pub dashboard_url: String,
    #[default = "1x0000000000000000000000000000000AA"]
    pub turnstile_secret_key: String,
    pub timeouts: TimeoutConfig,
    pub google_oauth2: GoogleOAuth2Config,
    pub telemetry: Option<TelemetryConfig>,
    /// Base64 encoded JWT secret key.
    #[default = "fEoUb9KpeJJTtfo3uUhehNHJBAeBL47fatN01OBlceg="]
    pub jwt_secret: String,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub struct TimeoutConfig {
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

scuffle_settings::bootstrap!(Config);

struct Global {
    config: Config,
    decoded_jwt_secret: Vec<u8>,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
    authorizer: cedar_policy::Authorizer,
    http_client: reqwest::Client,
    open_telemetry: opentelemetry::OpenTelemetry,
}

impl scufflecloud_core::CoreConfig for Global {
    fn service_name(&self) -> &str {
        &self.config.service_name
    }

    fn bind(&self) -> std::net::SocketAddr {
        self.config.bind
    }

    async fn db(&self) -> anyhow::Result<bb8::PooledConnection<'_, diesel_async::AsyncPgConnection>> {
        self.database.get().await.context("get database connection")
    }

    fn authorizer(&self) -> &cedar_policy::Authorizer {
        &self.authorizer
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    fn swagger_ui_enabled(&self) -> bool {
        self.config.swagger_ui
    }

    fn dashboard_url(&self) -> &str {
        &self.config.dashboard_url
    }

    fn turnstile_secret_key(&self) -> &str {
        &self.config.turnstile_secret_key
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

    fn webauthn_challenge_secret(&self) -> &[u8] {
        &self.decoded_jwt_secret
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

        let decoded_jwt_secret = base64::prelude::BASE64_STANDARD
            .decode(config.jwt_secret.as_bytes())
            .context("decode JWT secret")?;

        let Some(db_url) = config.db_url.as_deref() else {
            anyhow::bail!("DATABASE_URL is not set");
        };

        tracing::info!(db_url = config.db_url, "creating database connection pool");

        let database = bb8::Pool::builder()
            .build(diesel_async::pooled_connection::AsyncDieselConnectionManager::new(db_url))
            .await
            .context("build database pool")?;

        let http_client = reqwest::Client::builder()
            .user_agent(&config.service_name)
            .build()
            .context("create HTTP client")?;

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = crate::opentelemetry::OpenTelemetry::new()
            .with_traces(tracer)
            .with_logs(logger);

        Ok(Arc::new(Self {
            config,
            decoded_jwt_secret,
            database,
            authorizer: cedar_policy::Authorizer::new(),
            http_client,
            open_telemetry,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::services::CoreSvc::<Global>::default(),
    }
}
