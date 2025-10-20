use core_pb::v1::user_email_service_server::UserEmailService;
use core_pb::v1::{
    UserEmailAddRequest, UserEmailAddResponse, UserEmailListRequest, UserEmailListResponse, UserEmailVerifyRequest,
    UserEmailVerifyResponse,
};
use tonic::async_trait;

#[async_trait]
impl<G: core_traits::Global> UserEmailService for crate::services::CoreSvc<G> {
    async fn list(
        &self,
        request: tonic::Request<UserEmailListRequest>,
    ) -> tonic::Result<tonic::Response<UserEmailListResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn add(
        &self,
        request: tonic::Request<UserEmailAddRequest>,
    ) -> tonic::Result<tonic::Response<UserEmailAddResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn verify(
        &self,
        request: tonic::Request<UserEmailVerifyRequest>,
    ) -> tonic::Result<tonic::Response<UserEmailVerifyResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

