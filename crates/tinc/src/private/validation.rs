use axum::response::IntoResponse;
use tonic_types::{ErrorDetails, StatusExt};

use super::{HttpErrorResponse, HttpErrorResponseCode, HttpErrorResponseDetails, TrackerSharedState};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("error evaluating expression `{expression}` on field `{field}`: {error}")]
    Expression {
        field: Box<str>,
        error: Box<str>,

        expression: &'static str,
    },
    #[error("{0}")]
    FailFast(Box<str>),
}

impl serde::de::Error for ValidationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::FailFast(msg.to_string().into_boxed_str())
    }
}

#[cfg(feature = "tonic")]
impl From<ValidationError> for tonic::Status {
    fn from(value: ValidationError) -> Self {
        tonic::Status::internal(value.to_string())
    }
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        HttpErrorResponse {
            code: HttpErrorResponseCode::Internal,
            message: &message,
            details: HttpErrorResponseDetails::default(),
        }
        .into_response()
    }
}

impl From<ValidationError> for axum::response::Response {
    fn from(value: ValidationError) -> Self {
        value.into_response()
    }
}

pub trait ValidateMessage {
    fn validate(&self) -> Result<(), ValidationError>;

    #[cfg(feature = "tonic")]
    #[allow(clippy::result_large_err)]
    fn validate_tonic(&self) -> Result<(), tonic::Status> {
        let mut state = TrackerSharedState::default();

        state.in_scope(|| self.validate())?;

        if !state.errors.is_empty() {
            let mut details = ErrorDetails::new();

            for error in state.errors {
                details.add_bad_request_violation(error.proto_path.as_ref(), error.message());
            }

            Err(tonic::Status::with_error_details(
                tonic::Code::InvalidArgument,
                "bad request",
                details,
            ))
        } else {
            Ok(())
        }
    }
}

impl<V> ValidateMessage for Box<V>
where
    V: ValidateMessage,
{
    fn validate(&self) -> Result<(), ValidationError> {
        self.as_ref().validate()
    }
}
