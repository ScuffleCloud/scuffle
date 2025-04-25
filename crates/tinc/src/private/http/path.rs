use axum::extract::FromRequestParts;
use axum::response::IntoResponse;

use crate::__private::error::HttpErrorResponse;

pub async fn deserialize_path<T>(parts: &mut http::request::Parts) -> Result<T, axum::response::Response>
where
    T: serde::de::DeserializeOwned + Send,
{
    match axum::extract::Path::<T>::from_request_parts(parts, &()).await {
        Ok(axum::extract::Path(value)) => Ok(value),
        Err(err) => Err(HttpErrorResponse {
            code: tonic::Code::InvalidArgument.into(),
            details: Default::default(),
            message: &format!("invalid path: {err}"),
        }
        .into_response()),
    }
}
