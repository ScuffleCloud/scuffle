use core_pb::v1::user_login_service_server::UserLoginService;
use core_pb::v1::{
    UserLoginCompleteMfaRequest, UserLoginCompleteMfaResponse, UserLoginEmailPasswordRequest,
    UserLoginEmailPasswordResponse, UserLoginGoogleFinalizeRequest, UserLoginGoogleFinalizeResponse,
    UserLoginMagicLinkFinalizeRequest, UserLoginMagicLinkFinalizeResponse, UserLoginRequestGoogleRequest,
    UserLoginRequestGoogleResponse, UserLoginRequestMagicLinkRequest, UserLoginRequestMagicLinkResponse,
    UserLoginRequestMfaCodeRequest, UserLoginRequestMfaCodeResponse, UserLoginRequestMfaRequest,
    UserLoginRequestMfaResponse, UserLoginRequestWebauthnRequest, UserLoginRequestWebauthnResponse,
    UserLoginWebauthnFinalizeRequest, UserLoginWebauthnFinalizeResponse,
};
use tonic::async_trait;

#[async_trait]
impl<G: core_traits::Global> UserLoginService for crate::services::CoreSvc<G> {
    async fn email_password(
        &self,
        request: tonic::Request<UserLoginEmailPasswordRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginEmailPasswordResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn request_magic_link(
        &self,
        request: tonic::Request<UserLoginRequestMagicLinkRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginRequestMagicLinkResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn magic_link_finalize(
        &self,
        request: tonic::Request<UserLoginMagicLinkFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginMagicLinkFinalizeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn request_webauthn(
        &self,
        request: tonic::Request<UserLoginRequestWebauthnRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginRequestWebauthnResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn webauthn_finalize(
        &self,
        request: tonic::Request<UserLoginWebauthnFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginWebauthnFinalizeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn request_google(
        &self,
        request: tonic::Request<UserLoginRequestGoogleRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginRequestGoogleResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn google_finalize(
        &self,
        request: tonic::Request<UserLoginGoogleFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginGoogleFinalizeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_mfa(
        &self,
        request: tonic::Request<UserLoginCompleteMfaRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginCompleteMfaResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn request_mfa(
        &self,
        request: tonic::Request<UserLoginRequestMfaRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginRequestMfaResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn request_mfa_code(
        &self,
        request: tonic::Request<UserLoginRequestMfaCodeRequest>,
    ) -> tonic::Result<tonic::Response<UserLoginRequestMfaCodeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

