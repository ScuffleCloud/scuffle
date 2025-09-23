use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use fred::prelude::ClientLike;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub(crate) struct Config {
    #[default(env!("CARGO_PKG_NAME").to_string())]
    pub service_name: String,
    #[default(SocketAddr::from(([127, 0, 0, 1], 3001)))]
    pub core_bind: SocketAddr,
    #[default(SocketAddr::from(([127, 0, 0, 1], 3003)))]
    pub email_bind: SocketAddr,
    #[default = "info"]
    pub level: String,
    #[default(None)]
    pub db_url: Option<String>,
    #[default(false)]
    pub swagger_ui: bool,
    #[default = "scuffle.cloud"]
    pub rp_id: String,
    #[default(url::Url::from_str("https://dashboard.scuffle.cloud").unwrap())]
    pub dashboard_origin: url::Url,
    #[default = "1x0000000000000000000000000000000AA"]
    pub turnstile_secret_key: String,
    pub timeouts: TimeoutConfig,
    pub google_oauth2: GoogleOAuth2Config,
    pub telemetry: Option<TelemetryConfig>,
    pub redis: RedisConfig,
    #[default = "Scuffle"]
    pub email_from_name: String,
    #[default = "no-reply@scuffle.cloud"]
    pub email_from_address: String,
    pub reverse_proxy: Option<ReverseProxyConfig>,
    #[default("./GeoLite2-City.mmdb".parse().unwrap())]
    pub maxminddb_path: PathBuf,
    pub aws: AwsConfig,
}

scuffle_settings::bootstrap!(Config);

const fn days(days: u64) -> std::time::Duration {
    hours(days * 24)
}

const fn hours(hours: u64) -> std::time::Duration {
    minutes(hours * 60)
}

const fn minutes(mins: u64) -> std::time::Duration {
    std::time::Duration::from_secs(mins * 60)
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub(crate) struct TimeoutConfig {
    #[default(minutes(2))]
    pub max_request_lifetime: std::time::Duration,
    #[default(days(30))]
    pub user_session: std::time::Duration,
    #[default(minutes(5))]
    pub mfa: std::time::Duration,
    #[default(hours(4))]
    pub user_session_token: std::time::Duration,
    #[default(hours(1))]
    pub new_user_email_request: std::time::Duration,
    #[default(minutes(5))]
    pub user_session_request: std::time::Duration,
    #[default(minutes(15))]
    pub magic_link_request: std::time::Duration,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct GoogleOAuth2Config {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct RedisConfig {
    #[default(vec!["localhost:6379".to_string()])]
    pub servers: Vec<String>,
    #[default(None)]
    pub username: Option<String>,
    #[default(None)]
    pub password: Option<String>,
    #[default(0)]
    pub database: u8,
    #[default(10)]
    pub pool_size: usize,
}

fn parse_server(server: &str) -> anyhow::Result<fred::types::config::Server> {
    let port_ip = server.split(':').collect::<Vec<_>>();

    if port_ip.len() == 1 {
        Ok(fred::types::config::Server::new(port_ip[0], 6379))
    } else {
        Ok(fred::types::config::Server::new(
            port_ip[0],
            port_ip[1].parse::<u16>().context("invalid port")?,
        ))
    }
}

impl RedisConfig {
    pub(crate) async fn setup(&self) -> anyhow::Result<fred::clients::Pool> {
        let redis_server_config = if self.servers.len() == 1 {
            fred::types::config::ServerConfig::Centralized {
                server: parse_server(&self.servers[0])?,
            }
        } else {
            fred::types::config::ServerConfig::Clustered {
                hosts: self
                    .servers
                    .iter()
                    .map(|s| parse_server(s))
                    .collect::<anyhow::Result<Vec<_>>>()?,
                policy: Default::default(),
            }
        };

        tracing::info!(config = ?redis_server_config, "connecting to redis");

        let config = fred::types::config::Config {
            server: redis_server_config,
            database: Some(self.database),
            fail_fast: true,
            password: self.password.clone(),
            username: self.username.clone(),
            ..Default::default()
        };

        let client = fred::clients::Pool::new(config, None, None, None, self.pool_size).context("redis pool")?;
        client.init().await?;

        Ok(client)
    }
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct ReverseProxyConfig {
    /// List of networks that bypass the IP address extraction from the configured IP header.
    /// These are typically internal networks and other services that directly connect to the server without going
    /// through the reverse proxy.
    pub internal_networks: Vec<ipnetwork::IpNetwork>,
    #[default("x-forwarded-for".to_string())]
    pub ip_header: String,
    /// List of trusted proxy networks that the server accepts connections from.
    /// These are typically the networks of the reverse proxies in front of the server, e.g. Cloudflare, etc.
    pub trusted_proxies: Vec<ipnetwork::IpNetwork>,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub(crate) struct AwsConfig {
    #[default = "us-east-1"]
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}
