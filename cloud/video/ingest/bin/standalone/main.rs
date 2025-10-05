#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use std::sync::Arc;

use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;

struct Global {
    config: config::Config,
    open_telemetry: opentelemetry::OpenTelemetry,
}

impl ingest_traits::ConfigInterface for Global {
    fn rtmp_bind(&self) -> std::net::SocketAddr {
        self.config.rtmp_bind
    }
}

impl ingest_traits::Global for Global {}

impl scuffle_signal::SignalConfig for Global {}

impl scuffle_bootstrap_telemetry::TelemetryConfig for Global {
    fn enabled(&self) -> bool {
        self.config.telemetry.is_some()
    }

    fn bind_address(&self) -> Option<std::net::SocketAddr> {
        self.config.telemetry.as_ref().map(|telemetry| telemetry.bind)
    }

    fn http_server_name(&self) -> &str {
        "scufflecloud-ingest-telemetry"
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

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = opentelemetry::OpenTelemetry::new().with_traces(tracer).with_logs(logger);

        Ok(Arc::new(Self { config, open_telemetry }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_ingest::services::IngestSvc::<Global>::default(),
    }
}
