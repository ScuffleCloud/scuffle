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
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::logs::SdkLoggerProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
struct Config {
    #[default(env!("CARGO_PKG_NAME").to_string())]
    pub service_name: String,
    #[default(SocketAddr::from(([127, 0, 0, 1], 3002)))]
    pub bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    pub telemetry: Option<TelemetryConfig>,
    pub aws: AwsConfig,
    pub mtls: MtlsConfig,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
struct AwsConfig {
    #[default = "us-east-1"]
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
struct MtlsConfig {
    pub root_cert_path: PathBuf,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

scuffle_settings::bootstrap!(Config);

struct Global {
    config: Config,
    open_telemetry: opentelemetry::OpenTelemetry,
    external_http_client: reqwest::Client,
    aws_ses_req_signer: reqsign::Signer<reqsign::aws::Credential>,
    mtls_root_cert: Vec<u8>,
    mtls_cert: Vec<u8>,
    mtls_private_key: Vec<u8>,
}

impl email_traits::ConfigInterface for Global {
    fn service_bind(&self) -> std::net::SocketAddr {
        self.config.bind
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
        &self.mtls_cert
    }

    fn mtls_private_key_pem(&self) -> &[u8] {
        &self.mtls_private_key
    }
}

impl email_traits::Global for Global {}

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

        // TODO: only allow external requests
        let external_http_client = reqwest::Client::builder().user_agent(config.service_name.clone()).build()?;

        let provider = reqsign::aws::StaticCredentialProvider::new(&config.aws.access_key_id, &config.aws.secret_access_key);
        let signer = reqsign::aws::RequestSigner::new("ses", &config.aws.region);
        let aws_ses_req_signer = reqsign::Signer::new(reqsign::Context::new(), provider, signer);

        // mTLS
        let root_cert = std::fs::read(&config.mtls.root_cert_path).context("failed to read mTLS root cert file")?;
        let server_cert = std::fs::read(&config.mtls.cert_path).context("failed to read mTLS server cert file")?;
        let server_private_key =
            std::fs::read(&config.mtls.key_path).context("failed to read mTLS server private key file")?;

        Ok(Arc::new(Self {
            config,
            open_telemetry,
            external_http_client,
            aws_ses_req_signer,
            mtls_root_cert: root_cert,
            mtls_cert: server_cert,
            mtls_private_key: server_private_key,
        }))
    }
}

scuffle_bootstrap::main! {
    Global {
        scuffle_signal::SignalSvc,
        scufflecloud_email::services::EmailSvc::<Global>::default(),
    }
}
