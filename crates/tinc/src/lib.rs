pub mod reexports {
    pub use {axum, chrono, http, schemars, serde, serde_repr, tonic};
}

pub mod helpers;
pub mod value;

pub trait TincService {
    fn into_router(self) -> axum::Router;
}
