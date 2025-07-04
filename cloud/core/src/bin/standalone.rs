use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use diesel_async::pooled_connection::bb8;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub struct Config {
    #[default(SocketAddr::from(([127, 0, 0, 1], 3000)))]
    pub bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    #[default(None)]
    pub db_url: Option<String>,
    #[default = "https://dashboard.scuffle.cloud"]
    pub dashboard_url: String,
    #[default = "1x0000000000000000000000000000000AA"]
    pub turnstile_secret_key: String,
    pub timeouts: TimeoutConfig,
    pub google_oauth2: GoogleOAuth2Config,
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

scuffle_settings::bootstrap!(Config);

struct Global {
    config: Config,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
    http_client: reqwest::Client,
}

impl scufflecloud_core::CoreConfig for Global {
    fn bind(&self) -> std::net::SocketAddr {
        self.config.bind
    }

    async fn db(&self) -> anyhow::Result<bb8::PooledConnection<'_, diesel_async::AsyncPgConnection>> {
        self.database.get().await.context("get database connection")
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
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
}

impl scuffle_signal::SignalConfig for Global {}

impl scuffle_bootstrap::Global for Global {
    type Config = Config;

    async fn init(config: Self::Config) -> anyhow::Result<Arc<Self>> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(config.level.parse()?)),
            )
            .init();

        let Some(db_url) = config.db_url.as_deref() else {
            anyhow::bail!("DATABASE_URL is not set");
        };

        tracing::info!(db_url = config.db_url, "creating database connection pool");

        let database = bb8::Pool::builder()
            .build(diesel_async::pooled_connection::AsyncDieselConnectionManager::new(db_url))
            .await
            .context("build database pool")?;

        let http_client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("create HTTP client")?;

        Ok(Arc::new(Self {
            config,
            database,
            http_client,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::services::CoreSvc::<Global>::default(),
    }
}
