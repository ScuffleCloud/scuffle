use std::sync::Arc;

pub mod config;
pub mod schema;
pub mod service;

pub struct Global {
    config: config::Config,
}

impl scuffle_bootstrap::Global for Global {
    type Config = config::Config;

    async fn init(config: Self::Config) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self { config }))
    }
}

impl scuffle_signal::SignalConfig for Global {}
