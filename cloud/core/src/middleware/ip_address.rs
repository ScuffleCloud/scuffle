use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, Request};
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

pub(crate) async fn ip_address<G: CoreConfig>(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let _global = req.global::<G>().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get the IP address from the request
    // TODO: IP from headers if behind a proxy
    let info: ConnectInfo<SocketAddr> = axum::RequestExt::extract_parts(&mut req).await.map_err(|e| {
        tracing::error!(err = %e, "failed to extract ConnectInfo");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO: Get location from MaxMindDB

    req.extensions_mut().insert(IpAddressInfo { ip_address: info.ip() });

    Ok(next.run(req).await)
}
