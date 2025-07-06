use std::sync::Arc;

use axum::http;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::CoreConfig;
use crate::cedar::{self, CedarEntity};
use crate::models::UserSession;

pub(crate) trait RequestExt {
    fn extensions(&self) -> &http::Extensions;

    fn global<G: Send + Sync + 'static>(&self) -> Result<Arc<G>, tonic::Status> {
        self.extensions().get::<Arc<G>>().map(Arc::clone).ok_or_else(|| {
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

    fn ip_address_info(&self) -> Result<crate::middleware::IpAddressInfo, tonic::Status> {
        self.extensions()
            .get::<crate::middleware::IpAddressInfo>()
            .copied()
            .ok_or_else(|| {
                tracing::error!("missing IpAddressInfo extension");
                tonic::Status::with_error_details(Code::Internal, "missing IpAddressInfo extension", ErrorDetails::new())
            })
    }

    fn is_authorized<G: CoreConfig>(
        &self,
        principal: impl CedarEntity,
        action: impl CedarEntity,
        resource: impl CedarEntity,
    ) -> Result<(), tonic::Status> {
        cedar::is_authorized(&self.global::<G>()?, self.session(), principal, action, resource)
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
