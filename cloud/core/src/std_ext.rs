use std::fmt::Display;

use tonic_types::StatusExt;

pub(crate) trait ResultExt<T>: Sized {
    fn into_tonic(self, code: tonic::Code, msg: &str) -> Result<T, tonic::Status>;

    fn into_tonic_internal(self, msg: &str) -> Result<T, tonic::Status> {
        self.into_tonic(tonic::Code::Internal, msg)
    }
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Display,
{
    fn into_tonic(self, code: tonic::Code, msg: &str) -> Result<T, tonic::Status> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                tracing::error!(err = %e, "{}", msg);
                let status = tonic::Status::with_error_details(
                    code,
                    msg,
                    tonic_types::ErrorDetails::with_debug_info(vec![], e.to_string()),
                );

                Err(status)
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
