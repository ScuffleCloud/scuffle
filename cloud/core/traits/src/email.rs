pub trait EmailInterface {
    fn email_service(&self) -> impl EmailServiceClient;
}

pub trait EmailServiceClient: Send + Sync {
    fn send_email(
        &self,
        email: impl tonic::IntoRequest<pb::scufflecloud::email::v1::SendEmailRequest>,
    ) -> impl Future<Output = Result<tonic::Response<()>, tonic::Status>> + Send;
}
