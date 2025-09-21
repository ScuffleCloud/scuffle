use std::borrow::Cow;

use crate::resolver::GeoIpResolver;

mod http_ext;
pub mod middleware;
pub mod resolver;

pub use http_ext::*;
pub use maxminddb;

pub struct ReverseProxyConfig<'a> {
    /// List of networks that bypass the IP address extraction from the configured IP header.
    /// These are typically internal networks and other services that directly connect to the server without going
    /// through the reverse proxy.
    pub internal_networks: Cow<'a, [ipnetwork::IpNetwork]>,
    /// Request header to get the ip address from.
    pub ip_header: Cow<'a, str>,
    /// List of trusted proxy networks that the server accepts connections from.
    /// These are typically the networks of the reverse proxies in front of the server, e.g. Cloudflare, etc.
    pub trusted_proxies: Cow<'a, [ipnetwork::IpNetwork]>,
}

pub trait GeoIpInterface {
    fn geo_ip_resolver(&self) -> &GeoIpResolver;
    fn reverse_proxy_config(&self) -> Option<ReverseProxyConfig<'_>>;
}
