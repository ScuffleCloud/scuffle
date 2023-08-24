use std::net::SocketAddr;

use anyhow::Result;
use common::config::{LoggingConfig, NatsConfig, TlsConfig};

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct EdgeConfig {
    /// Bind Address
    pub bind_address: SocketAddr,

    /// If we should use TLS
    pub tls: Option<TlsConfig>,

    /// The session key to use for signing session tokens
    pub session_key: String,

    /// The segment key to use for signing segment tokens
    pub media_key: String,

    /// The name of the key value store to use for metadata
    pub metadata_kv_store: String,

    /// The name of the object store to use for media
    pub media_ob_store: String,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            bind_address: "[::]:9080".to_string().parse().unwrap(),
            tls: None,
            media_key: "media_key".to_string(),
            session_key: "session_key".to_string(),
            metadata_kv_store: "transcoder-metadata".to_string(),
            media_ob_store: "transcoder-media".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct GrpcConfig {
    /// The bind address for the gRPC server
    pub bind_address: SocketAddr,

    /// If we should use TLS for the gRPC server
    pub tls: Option<TlsConfig>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            bind_address: "[::]:50055".to_string().parse().unwrap(),
            tls: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// The database URL to use
    pub uri: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            uri: "postgres://root@localhost:5432/scuffle_video".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Name of this instance
    pub name: String,

    /// The path to the config file.
    pub config_file: Option<String>,

    /// The log level to use, this is a tracing env filter
    pub logging: LoggingConfig,

    /// API client configuration
    pub edge: EdgeConfig,

    /// gRPC server configuration
    pub grpc: GrpcConfig,

    /// Nats configuration
    pub nats: NatsConfig,

    /// Database configuration
    pub database: DatabaseConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "scuffle-transcoder".to_string(),
            config_file: Some("config".to_string()),
            edge: EdgeConfig::default(),
            grpc: GrpcConfig::default(),
            logging: LoggingConfig::default(),
            nats: NatsConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn parse() -> Result<Self> {
        let (mut config, config_file) =
            common::config::parse::<Self>(!cfg!(test), Self::default().config_file)?;

        config.config_file = config_file;

        Ok(config)
    }
}
