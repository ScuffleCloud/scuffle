use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub(crate) struct Config {
    #[default(env!("CARGO_PKG_NAME").to_string())]
    pub service_name: String,
    #[default = "info"]
    pub level: String,
    #[default("[::]:1935".parse().unwrap())]
    pub rtmp_bind: SocketAddr,
    pub rtmps: Option<RtmpsConfig>,
    pub telemetry: Option<TelemetryConfig>,
}

scuffle_settings::bootstrap!(Config);

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct RtmpsConfig {
    // https://github.com/obsproject/obs-studio/blob/0b1229632063a13dfd26cf1cd9dd43431d8c68f6/plugins/obs-outputs/librtmp/rtmp.c#L718
    // Yes, RTMPS' default port is 443
    #[default("[::]:443".parse().unwrap())]
    pub bind: SocketAddr,
    #[default = "rtmps_certs.pem"]
    pub cert_chain_path: PathBuf,
    #[default = "rtmps_key.pem"]
    pub key_path: PathBuf,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}
