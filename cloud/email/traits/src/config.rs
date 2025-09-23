pub trait ConfigInterface: Send + Sync {
    fn service_bind(&self) -> std::net::SocketAddr;
}
