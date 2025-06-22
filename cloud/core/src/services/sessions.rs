use std::sync::Arc;

use base64::Engine;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use rand::RngCore;

use crate::models::{EmailRegistrationRequest, EmailRegistrationRequestId, UserEmail};
use crate::schema::{email_registration_requests, user_emails};
use crate::services::CoreSvc;
use crate::{CoreConfig, captcha};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::sessions_service_server::SessionsService for CoreSvc<G> {
    async fn register_with_email(
        &self,
        request: tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (_, extensions, payload) = request.into_parts();

        let global = extensions
            .get::<Arc<G>>()
            .ok_or_else(|| tonic::Status::internal("missing global config"))?;

        let mut db = global
            .db()
            .await
            .map_err(|_| tonic::Status::internal("failed to get database connection"))?;

        // Check captcha
        let Some(captcha) = &payload.captcha else {
            return Err(tonic::Status::invalid_argument("missing captcha"));
        };

        match captcha.provider {
            p if p == pb::scufflecloud::core::v1::CaptchaProvider::Turnstile as i32 => {
                // Turnstile
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
            _ => return Err(tonic::Status::unimplemented("unsupported captcha provider")),
        }

        // Check if email is already registered
        let email = payload.email.trim().to_ascii_lowercase();

        if user_emails::dsl::user_emails
            .find(&email)
            .select(UserEmail::as_select())
            .first::<UserEmail>(&mut *db)
            .await
            .optional()
            .map_err(|_| tonic::Status::internal("failed to query database"))?
            .is_some()
        {
            return Err(tonic::Status::already_exists("email is already registered"));
        }

        // Generate random code
        let mut code = [0u8; 32];
        rand::rng().fill_bytes(&mut code);
        let code = base64::prelude::BASE64_URL_SAFE.encode(code);

        // Create email registration request
        let registration_request = EmailRegistrationRequest {
            id: EmailRegistrationRequestId::new(),
            user_id: None,
            email,
            code,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        diesel::insert_into(email_registration_requests::dsl::email_registration_requests)
            .values(registration_request)
            .execute(&mut *db)
            .await
            .map_err(|_| tonic::Status::internal("failed to insert email registration request"))?;

        // TODO: Send email with registration code link

        Ok(tonic::Response::new(()))
    }

    async fn complete_register_with_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        // Delete email registration request
        // Create user with given password
        // Create user session, device and token
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

    async fn invalidate_user_session(&self, _request: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
