pub trait ConfigInterface: Send + Sync {
    fn service_bind(&self) -> std::net::SocketAddr;
    fn swagger_ui_enabled(&self) -> bool;
}
