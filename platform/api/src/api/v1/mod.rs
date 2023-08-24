use std::sync::Arc;

use hyper::Body;
use routerify::Router;

use crate::global::GlobalState;

use super::error::RouteError;

pub mod gql;
pub mod health;
pub mod jwt;

pub fn routes(global: &Arc<GlobalState>) -> Router<Body, RouteError> {
    Router::builder()
        .scope("/health", health::routes(global))
        .scope("/gql", gql::routes(global))
        .build()
        .expect("failed to build router")
}
