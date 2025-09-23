use std::sync::Arc;

use base64::Engine;
use email_traits::AwsInterface;
use reqwest::Method;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SendEmailError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("request builder error: {0}")]
    Request(#[from] http::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to sign request: {0}")]
    Signer(#[from] reqsign::Error),
}

pub(crate) async fn send_email<G: email_traits::HttpClientInterface + AwsInterface>(
    global: &Arc<G>,
    raw_email: Vec<u8>,
) -> Result<(), SendEmailError> {
    // https://docs.aws.amazon.com/general/latest/gr/ses.html

    let data = base64::prelude::BASE64_STANDARD.encode(raw_email);
    let payload = SendEmailPayload {
        content: EmailContent::Raw(RawMessage { data }),
    };

    let req_body = serde_json::to_vec(&payload)?;
    let mut req_parts = http::Request::builder()
        .method(Method::POST)
        .uri(format!(
            "https://email.{}.amazonaws.com/v2/email/outbound-emails",
            global.aws_region()
        ))
        .header("content-type", "application/x-amz-json-1.1")
        .header("x-amz-content-sha256", &reqsign::hash::hex_sha256(&req_body))
        .body(())? // body is set later
        .into_parts()
        .0;

    global.aws_ses_req_signer().sign(&mut req_parts, None).await?;

    let req = http::Request::from_parts(req_parts, req_body).try_into()?;
    global.external_http_client().execute(req).await?.error_for_status()?;

    Ok(())
}

/// https://docs.aws.amazon.com/ses/latest/APIReference-V2/API_SendEmail.html
#[derive(serde_derive::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailPayload {
    content: EmailContent,
}

/// https://docs.aws.amazon.com/ses/latest/APIReference-V2/API_EmailContent.html
#[derive(serde_derive::Serialize)]
#[serde(rename_all = "PascalCase")]
enum EmailContent {
    Raw(RawMessage),
}

/// https://docs.aws.amazon.com/ses/latest/APIReference-V2/API_RawMessage.html
#[derive(serde_derive::Serialize)]
#[serde(rename_all = "PascalCase")]
struct RawMessage {
    /// Base64-encoded email
    data: String,
}
