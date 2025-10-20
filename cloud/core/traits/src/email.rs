pub trait EmailInterface {
    fn email_service(&self) -> impl EmailServiceClient;
}

pub trait EmailServiceClient: Send + Sync {
    fn send_email(
        &self,
        email: impl tonic::IntoRequest<email_pb::v1::SendEmailRequest>,
    ) -> impl Future<Output = tonic::Result<tonic::Response<()>>> + Send;
}
