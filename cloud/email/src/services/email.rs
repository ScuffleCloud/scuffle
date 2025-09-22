use ext_traits::{RequestExt, ResultExt};

use crate::services::EmailSvc;
use crate::{aws_ses, email_builder};

#[async_trait::async_trait]
impl<G: email_traits::Global> pb::scufflecloud::email::v1::email_service_server::EmailService for EmailSvc<G> {
    async fn send_email(
        &self,
        req: tonic::Request<pb::scufflecloud::email::v1::Email>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let global = &req.global::<G>()?;

        let raw_email = email_builder::build_raw_email(req.into_inner())?;
        aws_ses::send_email(global, raw_email)
            .await
            .into_tonic_internal_err("failed to send email with AWS SES")?;

        Ok(tonic::Response::new(()))
    }
}
