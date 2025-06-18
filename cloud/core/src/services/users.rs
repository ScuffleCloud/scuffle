use crate::services::CoreSvc;

#[async_trait::async_trait]
impl pb::scufflecloud::core::v1::users_service_server::UsersService for CoreSvc {
    async fn get_user(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn update_user_password(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn update_user_names(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UpdateUserNamesRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn update_user_primary_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPrimaryEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_user_emails(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::GetUserEmailsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn create_user_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CreateUserEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_create_user_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn delete_user_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::DeleteUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
