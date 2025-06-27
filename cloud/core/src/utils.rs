use tonic_types::{ErrorDetails, StatusExt};

#[derive(Debug, thiserror::Error)]
pub(crate) enum TxError {
    #[error("diesel transaction error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("tonic status error: {0}")]
    Status(#[from] tonic::Status),
}

impl From<TxError> for tonic::Status {
    fn from(err: TxError) -> Self {
        match err {
            TxError::Diesel(e) => tonic::Status::with_error_details(
                tonic::Code::Internal,
                format!("transaction error: {e}"),
                ErrorDetails::new(),
            ),
            TxError::Status(s) => s,
        }
    }
}
