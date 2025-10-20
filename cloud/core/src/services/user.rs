use core_pb::v1::user_service_server::UserService;
use core_pb::v1::{UserGetRequest, UserGetResponse, UserUpdateRequest, UserUpdateResponse};
use tonic::async_trait;

#[async_trait]
impl<G: core_traits::Global> UserService for crate::services::CoreSvc<G> {
    async fn get(
        &self,
        request: tonic::Request<UserGetRequest>,
    ) -> tonic::Result<tonic::Response<UserGetResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update(
        &self,
        request: tonic::Request<UserUpdateRequest>,
    ) -> tonic::Result<tonic::Response<UserUpdateResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

