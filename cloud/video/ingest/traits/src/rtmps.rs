use std::sync::Arc;

pub trait RtmpsInterface: Send + Sync {
    type RtmpsConfig: RtmpConfigInterface + Send + Sync;

    fn rtmps_config(&self) -> Option<&Self::RtmpsConfig>;
}

pub trait RtmpConfigInterface: Send + Sync {
    fn rtmps_bind(&self) -> std::net::SocketAddr;
    fn rtmps_rustls_server_config(&self) -> Arc<rustls::ServerConfig>;
}
