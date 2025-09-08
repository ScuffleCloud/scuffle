use std::net::SocketAddr;
use std::sync::Arc;

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
    pub telemetry: Option<TelemetryConfig>,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}

scuffle_settings::bootstrap!(Config);

struct Global {
    config: Config,
    open_telemetry: opentelemetry::OpenTelemetry,
}

impl scufflecloud_email::EmailConfig for Global {
    fn service_name(&self) -> &str {
        &self.config.service_name
    }

    fn bind(&self) -> std::net::SocketAddr {
        self.config.bind
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
        "scufflecloud-email-telemetry"
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

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = crate::opentelemetry::OpenTelemetry::new()
            .with_traces(tracer)
            .with_logs(logger);

        Ok(Arc::new(Self { config, open_telemetry }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_email::services::EmailSvc::<Global>::default(),
    }
}
