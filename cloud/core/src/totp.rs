use core_db_types::models::{MfaTotpCredential, UserId};
use core_db_types::schema::mfa_totp_credentials;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{DisplayExt, ResultExt};
use tonic_types::{ErrorDetails, StatusExt};
use totp_rs::{Algorithm, TOTP, TotpUrlError};

use crate::common;

const ISSUER: &str = "scuffle.cloud";

#[derive(Debug, thiserror::Error)]
pub(crate) enum TotpError {
    #[error("invalid TOTP secret: {0}")]
    InvalidSecret(#[from] TotpUrlError),
    #[error("system time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("invalid TOTP token")]
    InvalidToken,
    #[error("failed to generate random secret: {0}")]
    GenerateSecret(#[from] rand::Error),
}

pub(crate) fn new_token(account_name: String) -> Result<TOTP, TotpError> {
    // Generate a new secret for TOTP
    let secret = common::generate_random_bytes()?;
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_vec(),
        Some(ISSUER.to_string()),
        account_name,
    )?;
    Ok(totp)
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
    tx: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    user_id: UserId,
    token: &str,
) -> Result<(), tonic::Status> {
    let credentials = mfa_totp_credentials::dsl::mfa_totp_credentials
        .filter(mfa_totp_credentials::dsl::user_id.eq(user_id))
        .select(MfaTotpCredential::as_select())
        .load::<MfaTotpCredential>(tx)
        .await
        .into_tonic_internal_err("failed to query TOTP secrets")?;

    for credential in credentials {
        match verify_token(credential.secret, token) {
            Ok(_) => {
                // Update the last used timestamp
                diesel::update(mfa_totp_credentials::dsl::mfa_totp_credentials)
                    .filter(mfa_totp_credentials::dsl::id.eq(credential.id))
                    .set(mfa_totp_credentials::dsl::last_used_at.eq(chrono::Utc::now()))
                    .execute(tx)
                    .await
                    .into_tonic_internal_err("failed to update TOTP credential")?;
                return Ok(());
            }
            Err(TotpError::InvalidToken) => {} // Try the next secret
            Err(e) => return Err(e.into_tonic_internal_err("failed to verify TOTP token")),
        }
    }

    Err(tonic::Status::with_error_details(
        tonic::Code::PermissionDenied,
        "invalid TOTP token",
        ErrorDetails::new(),
    ))
}
