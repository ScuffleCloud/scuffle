use std::sync::Arc;

use crate::OptionExt;

pub trait RequestExt {
    fn extensions(&self) -> &http::Extensions;

    fn global<G: Send + Sync + 'static>(&self) -> Result<Arc<G>, tonic::Status> {
        self.extensions()
            .get::<Arc<G>>()
            .map(Arc::clone)
            .into_tonic_internal_err("missing global extension")
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

impl<T> RequestExt for http::Request<T> {
    fn extensions(&self) -> &http::Extensions {
        self.extensions()
    }
}
