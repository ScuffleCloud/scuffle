use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, Request};
use axum::http::{self, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use reqwest::StatusCode;

use crate::CoreConfig;
use crate::http_ext::RequestExt;

#[derive(Debug, Clone, Copy)]
pub(crate) struct IpAddressInfo {
    pub ip_address: IpAddr,
    // maxminddb...
}

impl IpAddressInfo {
    pub(crate) fn to_network(self) -> ipnetwork::IpNetwork {
        self.ip_address.into()
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseIpHeaderError {
    #[error("header value is not valid ASCII: {0}")]
    ValueNotAscii(#[from] http::header::ToStrError),
    #[error("header contains invalid IP address: {0}")]
    InvalidIp(#[from] std::net::AddrParseError),
}

fn parse_ip_header(value: &HeaderValue) -> Result<Vec<IpAddr>, ParseIpHeaderError> {
    let s = value.to_str()?;
    let ips = s.split(',').map(|part| part.trim().parse()).collect::<Result<Vec<_>, _>>()?;
    Ok(ips)
}

pub(crate) async fn ip_address<G: CoreConfig>(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let global = req.global::<G>().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get the connecting IP address from the request
    let info: ConnectInfo<SocketAddr> = axum::RequestExt::extract_parts(&mut req).await.map_err(|e| {
        tracing::error!(err = %e, "failed to extract ConnectInfo");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let mut ip_address = info.ip();

    // Extract the real IP address from the configured header if behind a reverse proxy
    if let Some(reverse_proxy_config) = global.reverse_proxy_config() {
        // Ignore all internal networks
        if !reverse_proxy_config
            .internal_networks
            .iter()
            .any(|net| net.contains(ip_address))
        {
            if !reverse_proxy_config
                .trusted_proxies
                .iter()
                .any(|net| net.contains(ip_address))
            {
                tracing::error!(ip = %ip_address, "untrusted ip address, connecting ip not in trusted proxies");
                return Err(StatusCode::BAD_REQUEST);
            }

            let ip_header = req.headers().get(&reverse_proxy_config.ip_header).ok_or_else(|| {
                tracing::error!(headers = ?req.headers(), header = reverse_proxy_config.ip_header, "missing IP header");
                StatusCode::BAD_REQUEST
            })?;
            let ips = parse_ip_header(ip_header).map_err(|e| {
                tracing::error!(err = %e, header = reverse_proxy_config.ip_header, "invalid IP header");
                StatusCode::BAD_REQUEST
            })?;

            // The IP header contains a list of IP addresses, the first one is the original client IP.
            // e.g. CLIENT -> EXTERNAL PROXY -> INTERNAL PROXY -> SERVER

            // We want to find the first IP address in the chain that is not part of our trusted proxies.
            // In this example, that would be EXTERNAL PROXY because it is not a trusted proxy.
            for ip in ips.iter().rev() {
                if !reverse_proxy_config.trusted_proxies.iter().any(|net| net.contains(*ip)) {
                    // Found the client IP
                    ip_address = *ip;
                    break;
                }
            }
        }
    }

    // TODO: Get location from MaxMindDB

    req.extensions_mut().insert(IpAddressInfo { ip_address });

    Ok(next.run(req).await)
}
