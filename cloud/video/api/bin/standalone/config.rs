use std::net::SocketAddr;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub(crate) struct Config {
    #[default("[::]:3001".parse().unwrap())]
    pub bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    #[default = true]
    pub swagger_ui: bool,
    pub telemetry: Option<TelemetryConfig>,
}

scuffle_settings::bootstrap!(Config);

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}
