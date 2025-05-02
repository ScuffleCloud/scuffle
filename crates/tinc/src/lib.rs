#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[doc(hidden)]
pub mod reexports {
    pub use {axum, chrono, headers_accept, http, linkme, mediatype, prost, regex, schemars, serde, serde_repr, tonic};
}

#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

pub trait TincService {
    fn into_router(self) -> axum::Router;
}

#[macro_export]
macro_rules! include_proto {
    ($package:tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}

pub use tonic::{Code, Request, Response, Result, Status, async_trait};
