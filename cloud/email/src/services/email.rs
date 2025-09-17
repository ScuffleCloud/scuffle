use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::EmailConfig;
use crate::services::EmailSvc;

#[async_trait::async_trait]
impl<G: EmailConfig> pb::scufflecloud::email::v1::email_service_server::EmailService for EmailSvc<G> {
    async fn send_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::email::v1::Email>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            Code::Unimplemented,
            "this call is not implemented yet",
            ErrorDetails::new(),
        ))
    }
}
