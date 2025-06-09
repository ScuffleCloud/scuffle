use std::net::SocketAddr;

pub struct Config {
    pub bind: SocketAddr,
}

impl scuffle_bootstrap::ConfigParser for Config {
    async fn parse() -> anyhow::Result<Self> {
        Ok(Self {
            bind: "[::]:3000".parse().unwrap(),
        })
    }
}
