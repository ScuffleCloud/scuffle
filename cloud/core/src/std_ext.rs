use std::fmt::Display;

use tonic_types::{ErrorDetails, StatusExt};

pub(crate) trait ResultExt<T>: Sized {
    fn into_tonic_err(self, code: tonic::Code, msg: &str, details: ErrorDetails) -> Result<T, tonic::Status>;

    fn into_tonic_internal_err(self, msg: &str) -> Result<T, tonic::Status> {
        self.into_tonic_err(tonic::Code::Internal, msg, ErrorDetails::new())
    }

    fn into_tonic_err_with_field_violation(self, field: &str, msg: &str) -> Result<T, tonic::Status> {
        self.into_tonic_err(
            tonic::Code::InvalidArgument,
            format!("{field}: {msg}").as_str(),
            ErrorDetails::with_bad_request_violation(field, msg),
        )
    }
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Display,
{
    fn into_tonic_err(self, code: tonic::Code, msg: &str, mut details: ErrorDetails) -> Result<T, tonic::Status> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                tracing::error!(err = %e, "{}", msg);
                details.set_debug_info(vec![], e.to_string());
                Err(tonic::Status::with_error_details(code, msg, details))
            }
        }
    }
}

pub(crate) trait OptionExt<T> {
    fn require(self, field: &str) -> Result<T, tonic::Status>;
}

impl<T> OptionExt<T> for Option<T> {
    fn require(self, field: &str) -> Result<T, tonic::Status> {
        self.ok_or_else(|| {
            tonic::Status::with_error_details(
                tonic::Code::InvalidArgument,
                format!("missing {field}"),
                tonic_types::ErrorDetails::with_bad_request_violation(field, "not set"),
            )
        })
    }
}
