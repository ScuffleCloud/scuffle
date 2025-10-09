#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use std::sync::Arc;

use anyhow::Context;
use diesel_async::pooled_connection::bb8;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::dataloaders::StreamLoader;

mod config;
mod dataloaders;

struct Global {
    config: config::Config,
    database: bb8::Pool<diesel_async::AsyncPgConnection>,
    stream_loader: DataLoader<StreamLoader>,
    open_telemetry: opentelemetry::OpenTelemetry,
}

impl video_api_traits::ConfigInterface for Global {
    fn service_bind(&self) -> std::net::SocketAddr {
        self.config.bind
    }

    fn swagger_ui_enabled(&self) -> bool {
        self.config.swagger_ui
    }
}

impl video_api_traits::DatabaseInterface for Global {
    type Connection<'a>
        = diesel_async::pooled_connection::bb8::PooledConnection<'a, diesel_async::pg::AsyncPgConnection>
    where
        Self: 'a;

    async fn db(&self) -> anyhow::Result<Self::Connection<'_>> {
        self.database.get().await.context("failed to get database connection")
    }
}

impl video_api_traits::DataloaderInterface for Global {
    fn stream_loader(
        &self,
    ) -> &scuffle_batching::DataLoader<
        impl DataLoaderFetcher<Key = db_types::models::StreamId, Value = db_types::models::Stream> + Send + Sync + 'static,
    > {
        &self.stream_loader
    }
}

impl video_api_traits::Global for Global {}

impl scuffle_signal::SignalConfig for Global {}

impl scuffle_bootstrap_telemetry::TelemetryConfig for Global {
    fn enabled(&self) -> bool {
        self.config.telemetry.is_some()
    }

    fn bind_address(&self) -> Option<std::net::SocketAddr> {
        self.config.telemetry.as_ref().map(|telemetry| telemetry.bind)
    }

    fn http_server_name(&self) -> &str {
        "scufflecloud-video-api-telemetry"
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

        let Some(db_url) = config.db_url.as_deref() else {
            anyhow::bail!("DATABASE_URL is not set");
        };

        tracing::info!(db_url = db_url, "creating database connection pool");

        let database = bb8::Pool::builder()
            .build(diesel_async::pooled_connection::AsyncDieselConnectionManager::new(db_url))
            .await
            .context("build database pool")?;

        let stream_loader = StreamLoader::new(database.clone());

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = opentelemetry::OpenTelemetry::new().with_traces(tracer).with_logs(logger);

        Ok(Arc::new(Self {
            config,
            database,
            stream_loader,
            open_telemetry,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_video_api::services::VideoApiSvc::<Global>::default(),
    }
}
