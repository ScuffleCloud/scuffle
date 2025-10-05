use std::net::SocketAddr;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub(crate) struct Config {
    #[default(env!("CARGO_PKG_NAME").to_string())]
    pub service_name: String,
    #[default = "info"]
    pub level: String,
    #[default("[::]:1935".parse().unwrap())]
    pub rtmp_bind: SocketAddr,
    pub telemetry: Option<TelemetryConfig>,
}

scuffle_settings::bootstrap!(Config);

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}
