use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rand::TryRngCore;
use sha2::Digest;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::chrono_datetime_ext::ChronoDateTimeExt;
use crate::id::Id;
use crate::models::{EmailRegistrationRequest, EmailRegistrationRequestId, User, UserEmail, UserSession};
use crate::request_ext::RequestExt;
use crate::result_ext::ResultExt;
use crate::schema::{email_registration_requests, user_emails, user_sessions, users};
use crate::services::CoreSvc;
use crate::session_crypto::{encrypt_token, generate_token};
use crate::utils::TxError;
use crate::{CoreConfig, captcha};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::sessions_service_server::SessionsService for CoreSvc<G> {
    async fn register_with_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        // Check captcha
        let Some(captcha) = payload.captcha else {
            return Err(tonic::Status::with_error_details(
                Code::InvalidArgument,
                "missing captcha",
                ErrorDetails::with_bad_request_violation("captcha", "not set"),
            ));
        };

        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        // Check if email is already registered
        let email = payload.email.trim().to_ascii_lowercase();

        if user_emails::dsl::user_emails
            .find(&email)
            .select(UserEmail::as_select())
            .first::<UserEmail>(&mut *db)
            .await
            .optional()
            .into_tonic_internal("failed to query database")?
            .is_some()
        {
            return Err(tonic::Status::with_error_details(
                Code::AlreadyExists,
                "email is already registered",
                ErrorDetails::new(),
            ));
        }

        // Generate random code
        let mut code = [0u8; 32];
        rand::rngs::OsRng
            .try_fill_bytes(&mut code)
            .into_tonic_internal("failed to generate random code")?;

        // Create email registration request
        let registration_request = EmailRegistrationRequest {
            id: EmailRegistrationRequestId::new(),
            user_id: None,
            email,
            code: code.to_vec(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        diesel::insert_into(email_registration_requests::dsl::email_registration_requests)
            .values(registration_request)
            .execute(&mut *db)
            .await
            .into_tonic_internal("failed to insert email registration request")?;

        // TODO: Send email with registration code link
        // let code = base64::prelude::BASE64_URL_SAFE.encode(code);

        Ok(tonic::Response::new(()))
    }

    async fn complete_register_with_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let ip_info = extensions.ip_address_info()?;
        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let Some(device) = payload.device else {
            return Err(tonic::Status::with_error_details(
                Code::InvalidArgument,
                "missing device",
                ErrorDetails::with_bad_request_violation("device", "not set"),
            ));
        };

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    // Delete email registration request
                    let registration_request = diesel::delete(email_registration_requests::dsl::email_registration_requests)
                        .filter(email_registration_requests::dsl::code.eq(&payload.code))
                        .returning(EmailRegistrationRequest::as_select())
                        .get_results::<EmailRegistrationRequest>(conn)
                        .await
                        .into_tonic_internal("failed to delete email registration request")?;
                    let registration_request = registration_request.into_iter().next().ok_or_else(|| {
                        tonic::Status::with_error_details(Code::NotFound, "unknown code", ErrorDetails::new())
                    })?;

                    // Create user with given password
                    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
                    let argon2 = Argon2::default();
                    let password_hash = argon2
                        .hash_password(payload.password.as_bytes(), &salt)
                        .into_tonic_internal("failed to hash password")?
                        .to_string();

                    let user = User {
                        id: Id::new(),
                        preferred_name: None,
                        first_name: None,
                        last_name: None,
                        password_hash: Some(password_hash),
                        primary_email: registration_request.email.clone(),
                    };
                    diesel::insert_into(users::dsl::users)
                        .values(&user)
                        .execute(conn)
                        .await
                        .into_tonic_internal("failed to insert user")?;

                    let user_email = UserEmail {
                        email: registration_request.email.clone(),
                        user_id: user.id,
                        created_at: chrono::Utc::now(),
                    };
                    diesel::insert_into(user_emails::dsl::user_emails)
                        .values(&user_email)
                        .execute(conn)
                        .await
                        .into_tonic_internal("failed to insert user email")?;

                    // Create user session, device and token
                    let device_fingerprint = sha2::Sha256::digest(&device.public_key_data).to_vec();

                    let token_id = Id::new();
                    let token_expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
                    let session_expires_at = chrono::Utc::now() + chrono::Duration::days(30);

                    let token = generate_token().into_tonic_internal("failed to generate token")?;

                    let encrypted_token = encrypt_token(device.algorithm(), &token, &device.public_key_data)?;

                    let user_session = UserSession {
                        user_id: user.id,
                        device_fingerprint,
                        device_algorithm: device.algorithm().into(),
                        device_pk_data: device.public_key_data,
                        last_used_at: chrono::Utc::now(),
                        last_ip: ip_info.to_network(),
                        token_id: Some(token_id),
                        token: Some(token.to_vec()),
                        token_expires_at: Some(token_expires_at),
                        expires_at: session_expires_at,
                    };
                    diesel::insert_into(user_sessions::dsl::user_sessions)
                        .values(&user_session)
                        .execute(conn)
                        .await
                        .into_tonic_internal("failed to insert user session")?;

                    Ok(pb::scufflecloud::core::v1::NewUserSessionToken {
                        id: token_id.to_string(),
                        encrypted_token,
                        expires_at_utc: Some(token_expires_at.to_prost_timestamp_utc()),
                        session_expires_at_utc: Some(session_expires_at.to_prost_timestamp_utc()),
                    })
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn login_with_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithEmailResponse>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn complete_login_with_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn login_with_external_provider(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::LoginWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithExternalProviderResponse>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn complete_login_with_external_provider(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithExternalProviderRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn login_with_webauthn_public_key(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyResponse>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn complete_login_with_webauthn_public_key(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithWebauthnPublicKeyRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn create_user_session_request(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CreateUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn get_user_session_request(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn get_user_session_request_by_code(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn approve_user_session_request_by_code(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::ApproveUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn complete_user_session_request(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CompleteUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn refresh_user_session(
        &self,
        _req: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn invalidate_user_session(&self, _req: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }
}
