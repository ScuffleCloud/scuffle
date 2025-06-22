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
}

scuffle_settings::bootstrap!(Config);

struct Global {
    config: Config,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
}

impl scufflecloud_core::CoreGlobal for Global {
    fn bind(&self) -> std::net::SocketAddr {
        self.config.bind
    }

    async fn db(&self) -> anyhow::Result<bb8::PooledConnection<'_, diesel_async::AsyncPgConnection>> {
        Ok(self.database.get().await.context("get database connection")?)
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

        Ok(Arc::new(Self { config, database }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::services::CoreSvc::<Global>::default(),
    }
}
