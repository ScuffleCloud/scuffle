use base64::Engine;
use sha2::Digest;

use crate::http_ext::RequestExt;
use crate::operations::Operation;
use crate::operations::user_sessions::{InvalidateUserSessionRequest, RefreshUserSessionRequest};
use crate::services::CoreSvc;
use crate::std_ext::OptionExt;
use crate::{CoreConfig, google_api};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::sessions_service_server::SessionsService for CoreSvc<G> {
    async fn login_with_magic_link(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn complete_login_with_magic_link(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn login_with_email_and_password(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailAndPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn login_with_google(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithGoogleRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithGoogleResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let device = payload.device.require("device")?;
        let device_fingerprint = sha2::Sha256::digest(&device.public_key_data);
        let state = base64::prelude::BASE64_URL_SAFE.encode(device_fingerprint);

        let authorization_url = google_api::authorization_url(global, &state);

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::LoginWithGoogleResponse {
            authorization_url,
        }))
    }

    async fn complete_login_with_google(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithGoogleRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn login_with_webauthn(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn create_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn get_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn get_user_session_request_by_code(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn approve_user_session_request_by_code(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ApproveUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn complete_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn validate_mfa_for_user_session(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ValidateMfaForUserSessionRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSession>, tonic::Status> {
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn refresh_user_session(
        &self,
        req: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let (metadata, extensions, _) = req.into_parts();
        let req = tonic::Request::from_parts(metadata, extensions, RefreshUserSessionRequest);
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }

    async fn invalidate_user_session(&self, req: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        let (metadata, extensions, _) = req.into_parts();
        let req = tonic::Request::from_parts(metadata, extensions, InvalidateUserSessionRequest);
        Operation::<G>::run(req).await.map(tonic::Response::new)
    }
}
