use crate::CoreConfig;
use crate::operations::Operation;
use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::users_service_server::UsersService for CoreSvc<G> {
    async fn get_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn update_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_user_emails(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListUserEmailsRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmailsList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn create_user_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateUserEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn complete_create_user_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn delete_user_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn create_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateWebauthnCredentialResponse>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn complete_create_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredential>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_webauthn_credentials(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListWebauthnCredentialsRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredentialsList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn delete_webauthn_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteWebauthnCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnCredential>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn create_webauthn_challenge(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnChallengeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::WebauthnChallenge>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn create_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateTotpCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateTotpCredentialResponse>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn complete_create_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateTotpCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredential>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn list_totp_credentials(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ListTotpCredentialsRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredentialsList>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn delete_totp_credential(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteTotpCredentialRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::TotpCredential>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn regenerate_recovery_codes(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::RegenerateRecoveryCodesRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::RecoveryCodes>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn delete_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::DeleteUserRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }
}
