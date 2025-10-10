pub trait ConfigInterface: Send + Sync {
    fn rtmp_bind(&self) -> std::net::SocketAddr;
}
