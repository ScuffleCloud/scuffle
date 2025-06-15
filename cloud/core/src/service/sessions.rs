use crate::service::CoreSvc;

#[async_trait::async_trait]
impl pb::scufflecloud::core::v1::sessions_service_server::SessionsService for CoreSvc {
    async fn register_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_register_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn login_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithEmailResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_login_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn login_with_external_provider(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::LoginWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithExternalProviderResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_login_with_external_provider(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn login_with_webauthn_public_key(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_login_with_webauthn_public_key(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithWebauthnPublicKeyRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn create_user_session_request(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CreateUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_user_session_request(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_user_session_request_by_code(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn approve_user_session_request_by_code(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::ApproveUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_user_session_request(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn refresh_user_session(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
