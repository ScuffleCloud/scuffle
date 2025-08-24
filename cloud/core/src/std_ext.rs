use std::fmt::Display;

use tonic_types::{ErrorDetails, StatusExt};

pub(crate) trait DisplayExt: Sized {
    fn into_tonic_err(self, code: tonic::Code, msg: &str, details: ErrorDetails) -> tonic::Status;

    fn into_tonic_internal_err(self, msg: &str) -> tonic::Status {
        self.into_tonic_err(tonic::Code::Internal, msg, ErrorDetails::new())
    }

    fn into_tonic_err_with_field_violation(self, field: &str, msg: &str) -> tonic::Status {
        self.into_tonic_err(
            tonic::Code::InvalidArgument,
            format!("{field}: {msg}").as_str(),
            ErrorDetails::with_bad_request_violation(field, msg),
        )
    }
}

impl<D> DisplayExt for D
where
    D: Display,
{
    fn into_tonic_err(self, code: tonic::Code, msg: &str, mut details: ErrorDetails) -> tonic::Status {
        tracing::error!(err = %self, "{}", msg);
        details.set_debug_info(vec![], self.to_string());
        tonic::Status::with_error_details(code, msg, details)
    }
}

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
    E: DisplayExt,
{
    fn into_tonic_err(self, code: tonic::Code, msg: &str, details: ErrorDetails) -> Result<T, tonic::Status> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err(e.into_tonic_err(code, msg, details)),
        }
    }
}

pub(crate) trait OptionExt<T>: Sized {
    fn into_tonic_err(self, code: tonic::Code, msg: &str, details: ErrorDetails) -> Result<T, tonic::Status>;

    fn into_tonic_not_found(self, msg: &str) -> Result<T, tonic::Status> {
        self.into_tonic_err(tonic::Code::NotFound, msg, ErrorDetails::new())
    }

    fn into_tonic_internal_err(self, msg: &str) -> Result<T, tonic::Status> {
        self.into_tonic_err(tonic::Code::Internal, msg, ErrorDetails::new())
    }

    fn require(self, field: &str) -> Result<T, tonic::Status> {
        self.into_tonic_err(
            tonic::Code::InvalidArgument,
            format!("missing {field}").as_str(),
            tonic_types::ErrorDetails::with_bad_request_violation(field, "not set"),
        )
    }
}

impl<T> OptionExt<T> for Option<T> {
    fn into_tonic_err(self, code: tonic::Code, msg: &str, details: ErrorDetails) -> Result<T, tonic::Status> {
        self.ok_or_else(|| {
            tracing::error!("{}", msg);
            tonic::Status::with_error_details(code, msg, details)
        })
    }
}
