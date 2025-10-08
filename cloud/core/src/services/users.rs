use pb::scufflecloud::core::v1::users_service_server::UsersService;
use pb::scufflecloud::core::v1::{
    CompleteCreateTotpCredentialRequest, CompleteCreateUserEmailRequest, CompleteCreateWebauthnCredentialRequest,
    CreateTotpCredentialRequest, CreateTotpCredentialResponse, CreateUserEmailRequest, CreateWebauthnChallengeRequest,
    CreateWebauthnCredentialRequest, CreateWebauthnCredentialResponse, DeleteTotpCredentialRequest, DeleteUserEmailRequest,
    DeleteWebauthnCredentialRequest, GetUserRequest, ListTotpCredentialsRequest, ListUserEmailsRequest,
    ListWebauthnCredentialsRequest, RecoveryCodes, RegenerateRecoveryCodesRequest, TotpCredential, TotpCredentialsList,
    UpdateTotpCredentialRequest, UpdateUserRequest, UpdateWebauthnCredentialRequest, User, UserEmail, UserEmailsList,
    WebauthnChallenge, WebauthnCredential, WebauthnCredentialsList,
};

use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: core_traits::Global> UsersService for CoreSvc<G> {
    async fn get_user(&self, req: tonic::Request<GetUserRequest>) -> Result<tonic::Response<User>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update_user(&self, req: tonic::Request<UpdateUserRequest>) -> Result<tonic::Response<User>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_user_emails(
        &self,
        req: tonic::Request<ListUserEmailsRequest>,
    ) -> Result<tonic::Response<UserEmailsList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_user_email(
        &self,
        req: tonic::Request<CreateUserEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_create_user_email(
        &self,
        req: tonic::Request<CompleteCreateUserEmailRequest>,
    ) -> Result<tonic::Response<UserEmail>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn delete_user_email(
        &self,
        req: tonic::Request<DeleteUserEmailRequest>,
    ) -> Result<tonic::Response<UserEmail>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_webauthn_credential(
        &self,
        req: tonic::Request<CreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<CreateWebauthnCredentialResponse>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_create_webauthn_credential(
        &self,
        req: tonic::Request<CompleteCreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<WebauthnCredential>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_webauthn_credentials(
        &self,
        req: tonic::Request<ListWebauthnCredentialsRequest>,
    ) -> Result<tonic::Response<WebauthnCredentialsList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update_webauthn_credential(
        &self,
        req: tonic::Request<UpdateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<WebauthnCredential>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn delete_webauthn_credential(
        &self,
        req: tonic::Request<DeleteWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<WebauthnCredential>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_webauthn_challenge(
        &self,
        req: tonic::Request<CreateWebauthnChallengeRequest>,
    ) -> Result<tonic::Response<WebauthnChallenge>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn create_totp_credential(
        &self,
        req: tonic::Request<CreateTotpCredentialRequest>,
    ) -> Result<tonic::Response<CreateTotpCredentialResponse>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn complete_create_totp_credential(
        &self,
        req: tonic::Request<CompleteCreateTotpCredentialRequest>,
    ) -> Result<tonic::Response<TotpCredential>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn list_totp_credentials(
        &self,
        req: tonic::Request<ListTotpCredentialsRequest>,
    ) -> Result<tonic::Response<TotpCredentialsList>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn update_totp_credential(
        &self,
        req: tonic::Request<UpdateTotpCredentialRequest>,
    ) -> Result<tonic::Response<TotpCredential>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn delete_totp_credential(
        &self,
        req: tonic::Request<DeleteTotpCredentialRequest>,
    ) -> Result<tonic::Response<TotpCredential>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn regenerate_recovery_codes(
        &self,
        req: tonic::Request<RegenerateRecoveryCodesRequest>,
    ) -> Result<tonic::Response<RecoveryCodes>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn delete_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        let _ = req;
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
