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

    fn origin(&self) -> Option<url::Url>;
}

impl<T> RequestExt for tonic::Request<T> {
    fn extensions(&self) -> &http::Extensions {
        self.extensions()
    }

    fn origin(&self) -> Option<url::Url> {
        self.metadata().get("origin")?.to_str().ok()?.parse().ok()
    }
}

impl RequestExt for tonic::Extensions {
    fn extensions(&self) -> &http::Extensions {
        self
    }

    fn origin(&self) -> Option<url::Url> {
        None
    }
}

impl<T> RequestExt for http::Request<T> {
    fn extensions(&self) -> &http::Extensions {
        self.extensions()
    }

    fn origin(&self) -> Option<url::Url> {
        self.headers().get("origin")?.to_str().ok()?.parse().ok()
    }
}
