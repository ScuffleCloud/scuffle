use std::net::SocketAddr;

pub mod schema;
pub mod service;

pub trait CoreGlobal: scuffle_bootstrap::Global + scuffle_signal::SignalConfig {
    fn bind(&self) -> SocketAddr;
}
