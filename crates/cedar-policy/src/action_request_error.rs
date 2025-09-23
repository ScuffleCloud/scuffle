use cedar_policy::{ContextJsonError, RequestValidationError};

/// An error that can occur when building a requst from a CedarAction.
#[derive(thiserror::Error, Debug)]
pub enum CedarActionRequestError {
    /// Error due to request validation
    #[error(transparent)]
    RequestValidationError(Box<RequestValidationError>),
    /// Error due to context parsing
    #[error(transparent)]
    ContextJsonError(Box<ContextJsonError>),
    /// Error from serde json
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl From<RequestValidationError> for CedarActionRequestError {
    fn from(value: RequestValidationError) -> Self {
        Self::RequestValidationError(Box::new(value))
    }
}

impl From<ContextJsonError> for CedarActionRequestError {
    fn from(value: ContextJsonError) -> Self {
        Self::ContextJsonError(Box::new(value))
    }
}
