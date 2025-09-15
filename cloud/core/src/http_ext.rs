use std::sync::Arc;

use axum::http;
use core_db_types::models::UserSession;
use geo_ip::middleware::IpAddressInfo;
use tonic::Code;
use tonic_types::ErrorDetails;

use crate::middleware::ExpiredSession;
use crate::std_ext::OptionExt;

pub(crate) trait RequestExt {
    fn extensions(&self) -> &http::Extensions;

    fn global<G: Send + Sync + 'static>(&self) -> Result<Arc<G>, tonic::Status> {
        self.extensions()
            .get::<Arc<G>>()
            .map(Arc::clone)
            .into_tonic_internal_err("missing global extension")
    }

    fn session(&self) -> Option<&UserSession> {
        self.extensions().get::<UserSession>()
    }

    fn session_or_err(&self) -> Result<&UserSession, tonic::Status> {
        self.session()
            .into_tonic_err(Code::Unauthenticated, "you must be logged in", ErrorDetails::new())
    }

    fn expired_session_or_err(&self) -> Result<&UserSession, tonic::Status> {
        self.extensions().get::<ExpiredSession>().map(|s| &s.0).into_tonic_err(
            Code::Unauthenticated,
            "you must be logged in",
            ErrorDetails::new(),
        )
    }

    fn ip_address_info(&self) -> Result<IpAddressInfo, tonic::Status> {
        self.extensions()
            .get::<IpAddressInfo>()
            .copied()
            .into_tonic_internal_err("missing IpAddressInfo extension")
    }
}

impl<T> RequestExt for tonic::Request<T> {
    fn extensions(&self) -> &http::Extensions {
        self.extensions()
    }
}

impl RequestExt for tonic::Extensions {
    fn extensions(&self) -> &http::Extensions {
        self
    }
}

impl<T> RequestExt for axum::http::Request<T> {
    fn extensions(&self) -> &http::Extensions {
        self.extensions()
    }
}
