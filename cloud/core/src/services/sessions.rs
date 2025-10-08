use pb::scufflecloud::core::v1::sessions_service_server::SessionsService;
use pb::scufflecloud::core::v1::{
    ApproveUserSessionRequestByCodeRequest, CompleteLoginWithGoogleRequest, CompleteLoginWithGoogleResponse,
    CompleteLoginWithMagicLinkRequest, CompleteUserSessionRequestRequest, CreateUserSessionRequestRequest,
    GetUserSessionRequestByCodeRequest, GetUserSessionRequestRequest, LoginWithEmailAndPasswordRequest,
    LoginWithGoogleRequest, LoginWithGoogleResponse, LoginWithMagicLinkRequest, LoginWithWebauthnRequest,
    NewUserSessionToken, UserSession, UserSessionRequest, ValidateMfaForUserSessionRequest,
};
use crate::http_ext::CoreRequestExt;
use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: core_traits::Global> SessionsService for CoreSvc<G> {
    async fn login_with_magic_link(
        &self,
        req: tonic::Request<LoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_login_with_magic_link(
        &self,
        req: tonic::Request<CompleteLoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<NewUserSessionToken>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn login_with_email_and_password(
        &self,
        req: tonic::Request<LoginWithEmailAndPasswordRequest>,
    ) -> Result<tonic::Response<NewUserSessionToken>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn login_with_google(
        &self,
        req: tonic::Request<LoginWithGoogleRequest>,
    ) -> Result<tonic::Response<LoginWithGoogleResponse>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_login_with_google(
        &self,
        req: tonic::Request<CompleteLoginWithGoogleRequest>,
    ) -> Result<tonic::Response<CompleteLoginWithGoogleResponse>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn login_with_webauthn(
        &self,
        req: tonic::Request<LoginWithWebauthnRequest>,
    ) -> Result<tonic::Response<NewUserSessionToken>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_user_session_request(
        &self,
        req: tonic::Request<CreateUserSessionRequestRequest>,
    ) -> Result<tonic::Response<UserSessionRequest>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn get_user_session_request(
        &self,
        req: tonic::Request<GetUserSessionRequestRequest>,
    ) -> Result<tonic::Response<UserSessionRequest>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn get_user_session_request_by_code(
        &self,
        req: tonic::Request<GetUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<UserSessionRequest>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn approve_user_session_request_by_code(
        &self,
        req: tonic::Request<ApproveUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<UserSessionRequest>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_user_session_request(
        &self,
        req: tonic::Request<CompleteUserSessionRequestRequest>,
    ) -> Result<tonic::Response<NewUserSessionToken>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn validate_mfa_for_user_session(
        &self,
        req: tonic::Request<ValidateMfaForUserSessionRequest>,
    ) -> Result<tonic::Response<UserSession>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn refresh_user_session(
        &self,
        req: tonic::Request<()>,
    ) -> Result<tonic::Response<NewUserSessionToken>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn invalidate_user_session(&self, req: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
