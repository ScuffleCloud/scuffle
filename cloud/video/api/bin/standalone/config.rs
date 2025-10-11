use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub(crate) struct Config {
    #[default("[::]:3001".parse().unwrap())]
    pub bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    #[default(None)]
    pub db_url: Option<String>,
    #[default = true]
    pub swagger_ui: bool,
    pub telemetry: Option<TelemetryConfig>,
    pub mtls: MtlsConfig,
}

scuffle_settings::bootstrap!(Config);

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct MtlsConfig {
    pub root_cert_path: PathBuf,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}
