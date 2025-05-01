#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod reexports {
    pub use {axum, chrono, headers_accept, http, mediatype, prost, schemars, serde, serde_repr, tonic};
}

#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

pub trait TincService {
    fn into_router(self) -> axum::Router;
}
