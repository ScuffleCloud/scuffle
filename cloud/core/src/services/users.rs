use pb::scufflecloud::core::v1::user_service_server::UserService;
use pb::scufflecloud::core::v1::{self};
use pb::scufflecloud::core::{self};
use tonic::async_trait;

use crate::services::CoreSvc;

#[async_trait]
impl<G: core_traits::Global> UserService for CoreSvc<G> {
    async fn get(&self, request: tonic::Request<v1::UserGetRequest>) -> tonic::Result<tonic::Response<v1::UserGetResponse>> {
        let _ = request;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update(
        &self,
        request: tonic::Request<v1::UserUpdateRequest>,
    ) -> tonic::Result<tonic::Response<v1::UserUpdateResponse>> {
        let _ = request;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update_password(
        &self,
        request: tonic::Request<v1::UserUpdatePasswordRequest>,
    ) -> tonic::Result<tonic::Response<v1::UserUpdatePasswordResponse>> {
        let _ = request;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update_primary_email(
        &self,
        request: tonic::Request<v1::UserUpdatePrimaryEmailRequest>,
    ) -> tonic::Result<tonic::Response<v1::UserUpdatePrimaryEmailResponse>> {
        let _ = request;
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
