use rand::TryRngCore;
use tonic_types::{ErrorDetails, StatusExt};

pub(crate) fn generate_random_bytes() -> Result<[u8; 32], rand::rand_core::OsError> {
    let mut token = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut token)?;
    Ok(token)
}

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
