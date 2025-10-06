#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use std::sync::Arc;

use anyhow::Context;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;

struct Global {
    config: config::Config,
    rtmps: Option<RtmpsGlobal>,
    open_telemetry: opentelemetry::OpenTelemetry,
}

struct RtmpsGlobal {
    bind: std::net::SocketAddr,
    rustls_config: Arc<tokio_rustls::rustls::ServerConfig>,
}

impl ingest_traits::RtmpConfigInterface for RtmpsGlobal {
    fn rtmps_bind(&self) -> std::net::SocketAddr {
        self.bind
    }

    fn rtmps_rustls_server_config(&self) -> Arc<tokio_rustls::rustls::ServerConfig> {
        self.rustls_config.clone()
    }
}

impl ingest_traits::ConfigInterface for Global {
    fn rtmp_bind(&self) -> std::net::SocketAddr {
        self.config.rtmp_bind
    }
}

impl ingest_traits::RtmpsInterface for Global {
    type RtmpsConfig = RtmpsGlobal;

    fn rtmps_config(&self) -> Option<&Self::RtmpsConfig> {
        self.rtmps.as_ref()
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

        let rtmps = if let Some(rtmps) = config.rtmps.as_ref() {
            let cert_chain = CertificateDer::pem_file_iter(&rtmps.cert_chain_path)
                .context("load RTMPS cert chain file")?
                .collect::<Result<Vec<_>, _>>()
                .context("load RTMPS cert chain")?;
            let key_der = PrivateKeyDer::from_pem_file(&rtmps.key_path).context("load RTMPS private key")?;

            let rustls_config = tokio_rustls::rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key_der)
                .context("create RTMPS rustls server config")?;

            Some(RtmpsGlobal {
                bind: rtmps.bind,
                rustls_config: Arc::new(rustls_config),
            })
        } else {
            None
        };

        let tracer = SdkTracerProvider::default();
        opentelemetry::global::set_tracer_provider(tracer.clone());

        let logger = SdkLoggerProvider::builder().build();

        let open_telemetry = opentelemetry::OpenTelemetry::new().with_traces(tracer).with_logs(logger);

        Ok(Arc::new(Self {
            config,
            rtmps,
            open_telemetry,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_ingest::services::IngestSvc::<Global>::default(),
    }
}
