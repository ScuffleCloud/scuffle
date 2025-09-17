use std::net::SocketAddr;

use anyhow::Context;
use fred::prelude::ClientLike;

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
#[serde(default)]
pub struct TimeoutConfig {
    #[default(chrono::Duration::minutes(2))]
    pub max_request_lifetime: chrono::Duration,
    #[default(chrono::Duration::days(30))]
    pub user_session: chrono::Duration,
    #[default(chrono::Duration::minutes(5))]
    pub mfa: chrono::Duration,
    #[default(chrono::Duration::hours(4))]
    pub user_session_token: chrono::Duration,
    #[default(chrono::Duration::hours(1))]
    pub email_registration_request: chrono::Duration,
    #[default(chrono::Duration::minutes(5))]
    pub user_session_request: chrono::Duration,
    #[default(chrono::Duration::minutes(15))]
    pub magic_link_user_session_request: chrono::Duration,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct GoogleOAuth2Config {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct TelemetryConfig {
    #[default("[::1]:4317".parse().unwrap())]
    pub bind: SocketAddr,
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct RedisConfig {
    #[default(vec!["localhost:6379".to_string()])]
    pub servers: Vec<String>,
    #[default(None)]
    pub username: Option<String>,
    #[default(None)]
    pub password: Option<String>,
    #[default(0)]
    pub database: u8,
    #[default(10)]
    pub max_connections: usize,
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
    pub async fn setup(&self) -> anyhow::Result<fred::clients::Pool> {
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

        let client = fred::clients::Pool::new(config, None, None, None, self.max_connections).context("redis pool")?;
        client.init().await?;

        Ok(client)
    }
}

#[derive(serde_derive::Deserialize, smart_default::SmartDefault, Debug, Clone)]
pub struct ReverseProxyConfig {
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
