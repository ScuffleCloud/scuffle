use std::sync::Arc;

use aws_sdk_sesv2::config::http::HttpResponse;
use aws_sdk_sesv2::error::SdkError;
use aws_sdk_sesv2::operation::send_email::SendEmailError;
use aws_sdk_sesv2::primitives::Blob;
use aws_sdk_sesv2::types::{EmailContent, RawMessage};

pub(crate) async fn send_email<G: email_traits::AwsSesInterface>(
    global: &Arc<G>,
    raw_email: Vec<u8>,
) -> Result<(), SdkError<SendEmailError, HttpResponse>> {
    let raw_message = RawMessage::builder()
        .data(Blob::new(raw_email))
        .build()
        .expect("data is always set");

    global
        .aws_ses_client()
        .send_email()
        .content(EmailContent::builder().raw(raw_message).build())
        .send()
        .await?;

    Ok(())
}
