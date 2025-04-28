#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use __private::RequestAlreadyValidated;

pub mod reexports {
    pub use {axum, chrono, headers_accept, http, mediatype, schemars, serde, serde_repr, tonic};
}

#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

pub trait TincService {
    fn into_router(self) -> axum::Router;
}

pub trait TincTonicRequestExt {
    #[allow(clippy::result_large_err)]
    fn validate(&self) -> Result<(), tonic::Status>;
}

impl<T> TincTonicRequestExt for tonic::Request<T> {
    fn validate(&self) -> Result<(), tonic::Status> {
        if self.extensions().get::<RequestAlreadyValidated>().is_some() {
            return Ok(());
        }

        Ok(())
    }
}
