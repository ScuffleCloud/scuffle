use std::future::{Ready, ready};
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::ConnectInfo;
use axum::http::{self, HeaderValue, Request, StatusCode};
use axum::response::IntoResponse;
use futures::TryFutureExt;
use futures::future::{Either, MapOk};
use maxminddb::MaxMindDbError;
use tower_layer::Layer;
use tower_service::Service;

use crate::GeoIpInterface;

#[derive(Debug, Clone, Copy)]
pub struct IpAddressInfo {
    pub ip_address: IpAddr,
}

impl IpAddressInfo {
    pub fn to_network(self) -> ipnetwork::IpNetwork {
        self.ip_address.into()
    }

    pub fn lookup_geoip_info<'a, T: serde::Deserialize<'a>>(
        &self,
        global: &'a impl crate::GeoIpInterface,
    ) -> Result<Option<T>, MaxMindDbError> {
        global.geo_ip_resolver().lookup::<T>(self.ip_address)
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
    let ips = s
        .split(',')
        .map(|part| part.trim().parse::<IpAddr>().map(|ip| ip.to_canonical()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ips)
}

pub fn middleware<G>() -> GeoIpLayer<G> {
    GeoIpLayer { _marker: PhantomData }
}

pub struct GeoIpLayer<G> {
    _marker: PhantomData<G>,
}

impl<G> Clone for GeoIpLayer<G> {
    fn clone(&self) -> Self {
        Self { _marker: self._marker }
    }
}

pub struct GeoIpService<G, S> {
    inner: S,
    _marker: PhantomData<G>,
}

impl<G, S: Clone> Clone for GeoIpService<G, S> {
    fn clone(&self) -> Self {
        GeoIpService {
            inner: self.inner.clone(),
            _marker: self._marker,
        }
    }
}

impl<S, G> Layer<S> for GeoIpLayer<G> {
    type Service = GeoIpService<G, S>;

    fn layer(&self, inner: S) -> Self::Service {
        GeoIpService {
            inner,
            _marker: self._marker,
        }
    }
}

macro_rules! try_ret {
    ($result:expr) => {
        match $result {
            Ok(r) => r,
            Err(err) => ret_err!(err),
        }
    };
}

macro_rules! ret_err {
    ($err:expr) => {
        return Either::Right(ready(Ok($err.into_response())))
    };
}

impl<S, B, G> Service<Request<B>> for GeoIpService<G, S>
where
    S: Service<Request<B>>,
    S::Response: IntoResponse,
    G: GeoIpInterface + Send + Sync + 'static,
{
    type Error = S::Error;
    type Future = Either<MapOk<S::Future, fn(S::Response) -> Self::Response>, Ready<Result<Self::Response, S::Error>>>;
    type Response = axum::response::Response;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let Some(info) = req.extensions().get::<ConnectInfo<SocketAddr>>() else {
            tracing::error!("failed to get connection info");
            ret_err!(StatusCode::INTERNAL_SERVER_ERROR);
        };

        let mut ip_address = info.ip().to_canonical();
        let Some(global) = req.extensions().get::<Arc<G>>() else {
            tracing::error!("request missing global");
            ret_err!(StatusCode::INTERNAL_SERVER_ERROR);
        };

        if let Some(reverse_proxy_config) = global.reverse_proxy_config()
            && !reverse_proxy_config
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
                ret_err!(StatusCode::BAD_REQUEST);
            }

            let ip_header = try_ret!(req.headers().get(reverse_proxy_config.ip_header.as_ref()).ok_or_else(|| {
                tracing::error!(headers = ?req.headers(), header = reverse_proxy_config.ip_header.as_ref(), "missing IP header");
                StatusCode::BAD_REQUEST
            }));
            let ips = try_ret!(parse_ip_header(ip_header).map_err(|e| {
                tracing::error!(err = %e, header = reverse_proxy_config.ip_header.as_ref(), "invalid IP header");
                StatusCode::BAD_REQUEST
            }));

            for ip in ips.iter().rev() {
                if !reverse_proxy_config.trusted_proxies.iter().any(|net| net.contains(*ip)) {
                    // Found the client IP
                    ip_address = *ip;
                    break;
                }
            }

            req.extensions_mut().insert(IpAddressInfo { ip_address });
        }

        Either::Left(self.inner.call(req).map_ok(|resp| resp.into_response()))
    }
}
