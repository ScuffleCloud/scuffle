use axum::response::IntoResponse;

use super::{
    HttpErrorResponse, HttpErrorResponseCode, HttpErrorResponseDetails, HttpErrorResponseRequestViolation, TrackerFor,
    TrackerSharedState, TrackerWrapper,
};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("error evaluating expression `{expression}` on field `{field}`: {error}")]
    Expression {
        field: &'static str,
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

pub trait TincValidate
where
    Self: TrackerFor,
    Self::Tracker: TrackerWrapper,
{
    fn validate(&self, tracker: Option<&Self::Tracker>) -> Result<(), ValidationError>;

    #[allow(clippy::result_large_err)]
    fn validate_http(&self, mut state: TrackerSharedState, tracker: &Self::Tracker) -> Result<(), axum::response::Response> {
        tinc_cel::CelMode::Serde.set();

        state.in_scope(|| self.validate(Some(tracker)))?;

        if state.errors.is_empty() {
            Ok(())
        } else {
            let mut details = HttpErrorResponseDetails::default();

            for error in &state.errors {
                details.request.violations.push(HttpErrorResponseRequestViolation {
                    field: error.path.as_ref(),
                    description: error.message(),
                })
            }

            Err(HttpErrorResponse {
                code: HttpErrorResponseCode::InvalidArgument,
                message: "bad request",
                details,
            }
            .into_response())
        }
    }

    #[cfg(feature = "tonic")]
    #[allow(clippy::result_large_err)]
    fn validate_tonic(&self) -> Result<(), tonic::Status> {
        tinc_cel::CelMode::Proto.set();

        use tonic_types::{ErrorDetails, StatusExt};

        use crate::__private::TrackerSharedState;

        let mut state = TrackerSharedState::default();

        state.in_scope(|| self.validate(None))?;

        if !state.errors.is_empty() {
            let mut details = ErrorDetails::new();

            for error in state.errors {
                details.add_bad_request_violation(error.path.as_ref(), error.message());
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

impl<V> TincValidate for Box<V>
where
    V: TincValidate,
    V::Tracker: TrackerWrapper,
{
    fn validate(&self, tracker: Option<&Self::Tracker>) -> Result<(), ValidationError> {
        self.as_ref().validate(tracker.map(|t| t.as_ref()))
    }
}
