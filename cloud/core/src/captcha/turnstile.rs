use std::net::IpAddr;
use std::sync::Arc;

use core_traits::DisplayExt;
use tonic_types::{ErrorDetails, StatusExt};

const TURNSTILE_VERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Debug, serde_derive::Serialize)]
struct TurnstileSiteVerifyPayload {
    pub secret: String,
    pub response: String,
    pub remoteip: Option<String>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct TurnstileSiteVerifyResponse {
    pub success: bool,
    // pub chanllenge_ts: chrono::DateTime<chrono::Utc>,
    // pub hostname: String,
    #[serde(rename = "error-codes")]
    pub error_codes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TrunstileVerifyError {
    #[error("request to verify server failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("turnstile error code: {0}")]
    TurnstileError(String),
    #[error("missing error code in turnstile response")]
    MissingErrorCode,
}

pub(crate) async fn verify<G: core_traits::Global>(
    global: &Arc<G>,
    remote_ip: IpAddr,
    token: &str,
) -> Result<(), TrunstileVerifyError> {
    let payload = TurnstileSiteVerifyPayload {
        secret: global.turnstile_secret_key().to_string(),
        response: token.to_string(),
        remoteip: Some(remote_ip.to_string()),
    };

    let res = global
        .external_http_client()
        .post(TURNSTILE_VERIFY_URL)
        .json(&payload)
        .send()
        .await;

    if res.is_err() {
        tracing::warn!("failed to send turnstile verify request: {:?}", res);
    }

    let res: TurnstileSiteVerifyResponse = res?.json().await?;

    if !res.success {
        let Some(error_code) = res.error_codes.into_iter().next() else {
            return Err(TrunstileVerifyError::MissingErrorCode);
        };
        return Err(TrunstileVerifyError::TurnstileError(error_code));
    }

    Ok(())
}

pub(crate) async fn verify_in_tonic<G: core_traits::Global>(
    global: &Arc<G>,
    remote_ip: IpAddr,
    token: &str,
) -> Result<(), tonic::Status> {
    match verify(global, remote_ip, token).await {
        Ok(_) => Ok(()),
        Err(TrunstileVerifyError::TurnstileError(e)) => Err(tonic::Status::with_error_details(
            tonic::Code::Unauthenticated,
            TrunstileVerifyError::TurnstileError(e).to_string(),
            ErrorDetails::new(),
        )),
        Err(e) => Err(e.into_tonic_internal_err("failed to verify turnstile token")),
    }
}
