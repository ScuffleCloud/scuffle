use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tonic_types::{ErrorDetails, StatusExt};
use totp_rs::{Algorithm, TOTP, TotpUrlError};

use crate::models::UserId;
use crate::schema::mfa_totps;
use crate::std_ext::ResultExt;

const ISSUER: &str = "scuffle.cloud";

#[derive(Debug, thiserror::Error)]
pub(crate) enum TotpError {
    #[error("invalid TOTP secret: {0}")]
    InvalidSecret(#[from] TotpUrlError),
    #[error("system time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("invalid TOTP token")]
    InvalidToken,
}

pub(crate) fn verify_token(secret: Vec<u8>, token: &str) -> Result<(), TotpError> {
    // https://docs.rs/totp-rs/5.7.0/totp_rs/struct.TOTP.html#fields
    // account_name is not used in the verification process, so we can leave it empty.
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, Some(ISSUER.to_string()), String::new())?;

    if totp.check_current(token)? {
        Ok(())
    } else {
        Err(TotpError::InvalidToken)
    }
}

pub(crate) async fn process_token(
    tx: &mut diesel_async::AsyncPgConnection,
    user_id: UserId,
    token: &str,
) -> Result<(), tonic::Status> {
    let secrets = mfa_totps::dsl::mfa_totps
        .filter(mfa_totps::dsl::user_id.eq(user_id))
        .select(mfa_totps::dsl::secret)
        .load::<Vec<u8>>(tx)
        .await
        .into_tonic_internal_err("failed to query TOTP secrets")?;

    for secret in secrets {
        match verify_token(secret, token) {
            Ok(_) => return Ok(()),
            Err(TotpError::InvalidToken) => {} // Try the next secret
            Err(e) => return Err(e).into_tonic_internal_err("failed to verify TOTP token"),
        }
    }

    Err(tonic::Status::with_error_details(
        tonic::Code::PermissionDenied,
        "invalid TOTP token",
        ErrorDetails::new(),
    ))
}
