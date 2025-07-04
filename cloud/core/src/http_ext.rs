use std::sync::Arc;

use axum::http;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::middleware::UnverifiedUserSession;
use crate::models::UserSession;

pub(crate) trait RequestExt {
    fn extensions(&self) -> &http::Extensions;

    fn global<G: Send + Sync + 'static>(&self) -> Result<&Arc<G>, tonic::Status> {
        self.extensions().get::<Arc<G>>().ok_or_else(|| {
            tracing::error!("missing global extension");
            tonic::Status::with_error_details(Code::Internal, "missing global extension", ErrorDetails::new())
        })
    }

    fn session(&self) -> Option<&UserSession> {
        self.extensions().get::<UserSession>()
    }

    fn session_or_err(&self) -> Result<&UserSession, tonic::Status> {
        self.session().ok_or_else(|| {
            tonic::Status::with_error_details(Code::Unauthenticated, "you must be logged in", ErrorDetails::new())
        })
    }

    fn unverified_session(&self) -> Option<&UnverifiedUserSession> {
        self.extensions().get::<UnverifiedUserSession>()
    }

    fn unverified_session_or_err(&self) -> Result<&UnverifiedUserSession, tonic::Status> {
        self.unverified_session().ok_or_else(|| {
            tonic::Status::with_error_details(Code::Unauthenticated, "you must be logged in", ErrorDetails::new())
        })
    }

    fn ip_address_info(&self) -> Result<&crate::middleware::IpAddressInfo, tonic::Status> {
        self.extensions().get::<crate::middleware::IpAddressInfo>().ok_or_else(|| {
            tracing::error!("missing IpAddressInfo extension");
            tonic::Status::with_error_details(Code::Internal, "missing IpAddressInfo extension", ErrorDetails::new())
        })
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
