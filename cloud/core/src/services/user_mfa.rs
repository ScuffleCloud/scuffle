use core_pb::v1::user_mfa_service_server::UserMfaService;
use core_pb::v1::{
    UserMfaRegenerateRecoveryCodesRequest, UserMfaRegenerateRecoveryCodesResponse, UserMfaTotpCredentialFinalizeRequest,
    UserMfaTotpCredentialFinalizeResponse, UserMfaTotpCredentialSetupRequest, UserMfaTotpCredentialSetupResponse,
    UserMfaTotpCredentialUpdateRequest, UserMfaTotpCredentialUpdateResponse, UserMfaWebauthnCredentialFinalizeRequest,
    UserMfaWebauthnCredentialFinalizeResponse, UserMfaWebauthnCredentialSetupRequest,
    UserMfaWebauthnCredentialSetupResponse, UserMfaWebauthnCredentialUpdateRequest,
    UserMfaWebauthnCredentialUpdateResponse,
};
use tonic::async_trait;

#[async_trait]
impl<G: core_traits::Global> UserMfaService for crate::services::CoreSvc<G> {
    async fn webauthn_credential_setup(
        &self,
        request: tonic::Request<UserMfaWebauthnCredentialSetupRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaWebauthnCredentialSetupResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn webauthn_credential_finalize(
        &self,
        request: tonic::Request<UserMfaWebauthnCredentialFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaWebauthnCredentialFinalizeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn webauthn_credential_update(
        &self,
        request: tonic::Request<UserMfaWebauthnCredentialUpdateRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaWebauthnCredentialUpdateResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn totp_credential_setup(
        &self,
        request: tonic::Request<UserMfaTotpCredentialSetupRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaTotpCredentialSetupResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn totp_credential_finalize(
        &self,
        request: tonic::Request<UserMfaTotpCredentialFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaTotpCredentialFinalizeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn totp_credential_update(
        &self,
        request: tonic::Request<UserMfaTotpCredentialUpdateRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaTotpCredentialUpdateResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn regenerate_recovery_codes(
        &self,
        request: tonic::Request<UserMfaRegenerateRecoveryCodesRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaRegenerateRecoveryCodesResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

