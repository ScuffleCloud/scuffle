use argon2::{Argon2, PasswordVerifier};
use base64::Engine;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::http_ext::RequestExt;
use crate::id::Id;
use crate::models::{
    EmailRegistrationRequest, EmailRegistrationRequestId, MagicLinkUserSessionRequest, UserEmail, UserSessionRequest,
};
use crate::schema::{
    email_registration_requests, magic_link_user_session_requests, mfa_webauthn_pks, user_emails, user_session_requests,
};
use crate::services::CoreSvc;
use crate::std_ext::{OptionExt, ResultExt};
use crate::utils::TxError;
use crate::{CoreConfig, captcha};

mod common;
mod crypto;

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
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        // TODO: In transaction?

        // Check if email is already registered
        let email = common::normalize_email(&payload.email);

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
        let code = crypto::generate_random_bytes().into_tonic_internal("failed to generate registration code")?;

        // Create email registration request
        let registration_request = EmailRegistrationRequest {
            id: EmailRegistrationRequestId::new(),
            user_id: None,
            email,
            code: code.to_vec(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        let code_base64 = base64::prelude::BASE64_STANDARD.encode(code);
        tracing::info!(reg_req = ?registration_request, code = %code_base64, "inserting registration request");

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

        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    // Delete email registration request
                    let Some(registration_request) =
                        diesel::delete(email_registration_requests::dsl::email_registration_requests)
                            .filter(email_registration_requests::dsl::code.eq(&payload.code))
                            .returning(EmailRegistrationRequest::as_select())
                            .get_result::<EmailRegistrationRequest>(conn)
                            .await
                            .optional()
                            .into_tonic_internal("failed to delete email registration request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown code", ErrorDetails::new()).into(),
                        );
                    };

                    let (_, new_token) = common::create_new_user_and_session(
                        conn,
                        registration_request.email,
                        Some(&payload.password),
                        device,
                        ip_info,
                    )
                    .await?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn login_with_email_options(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailOptionsRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        // Check captcha
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let user = common::get_user_by_email(&mut db, &payload.email).await?;

        let mut options = vec![pb::scufflecloud::core::v1::LoginWithEmailOptions::MagicLink as i32];

        if user.password_hash.is_some() {
            options.push(pb::scufflecloud::core::v1::LoginWithEmailOptions::Password as i32);
        }

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse { options },
        ))
    }

    async fn login_with_email_and_password(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailAndPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let ip_info = extensions.ip_address_info()?;
        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let device = payload.device.require("device")?;
        let captcha = payload.captcha.require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let user = common::get_user_by_email(conn, &payload.email).await?;

                    // Verify password
                    let Some(password_hash) = &user.password_hash else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::FailedPrecondition,
                            "user does not have a password set",
                            ErrorDetails::new(),
                        )
                        .into());
                    };

                    let password_hash =
                        argon2::PasswordHash::new(password_hash).into_tonic_internal("failed to parse password hash")?;

                    match Argon2::default().verify_password(payload.password.as_bytes(), &password_hash) {
                        Ok(_) => {}
                        Err(argon2::password_hash::Error::Password) => {
                            return Err(tonic::Status::with_error_details(
                                tonic::Code::PermissionDenied,
                                "invalid password",
                                ErrorDetails::with_bad_request_violation("password", "invalid password"),
                            )
                            .into());
                        }
                        Err(_) => {
                            return Err(tonic::Status::with_error_details(
                                tonic::Code::Internal,
                                "failed to verify password",
                                ErrorDetails::new(),
                            )
                            .into());
                        }
                    }

                    Ok(common::create_session(conn, user.id, device, ip_info).await?)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn login_with_magic_link(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let captcha = payload.captcha.require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        db.transaction::<_, TxError, _>(|conn| {
            async move {
                let user = common::get_user_by_email(conn, &payload.email).await?;

                let code = crypto::generate_random_bytes().into_tonic_internal("failed to generate magic link code")?;

                // Insert email link user session request
                let session_request = MagicLinkUserSessionRequest {
                    id: Id::new(),
                    user_id: user.id,
                    code: code.to_vec(),
                    expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                };
                diesel::insert_into(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
                    .values(session_request)
                    .execute(conn)
                    .await
                    .into_tonic_internal("failed to insert magic link user session request")?;

                // TODO: Send email with code

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(tonic::Response::new(()))
    }

    async fn complete_login_with_magic_link(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let ip_info = extensions.ip_address_info()?;
        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    // Find and delete magic link user session request
                    let Some(session_request) =
                        diesel::delete(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
                            .filter(magic_link_user_session_requests::dsl::code.eq(&payload.code))
                            .returning(MagicLinkUserSessionRequest::as_select())
                            .get_result::<MagicLinkUserSessionRequest>(conn)
                            .await
                            .optional()
                            .into_tonic_internal("failed to delete magic link user session request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown code", ErrorDetails::new()).into(),
                        );
                    };

                    // Create a new session for the user
                    let new_token = common::create_session(conn, session_request.user_id, device, ip_info).await?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
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
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyResponse>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        // Check captcha
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let challenge = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let challenge =
                        crypto::generate_random_bytes().into_tonic_internal("failed to generate webauthn challenge")?;

                    let n = diesel::update(mfa_webauthn_pks::dsl::mfa_webauthn_pks)
                        .filter(mfa_webauthn_pks::dsl::pk_id.eq(&payload.pk_id))
                        .set((
                            mfa_webauthn_pks::dsl::current_challenge.eq(&challenge),
                            mfa_webauthn_pks::dsl::current_challenge_expires_at
                                .eq(chrono::Utc::now() + chrono::Duration::minutes(5)),
                        ))
                        .execute(conn)
                        .await
                        .into_tonic_internal("failed to update webauthn public key")?;

                    if n == 0 {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::NotFound,
                            "webauthn public key not found",
                            ErrorDetails::new(),
                        )
                        .into());
                    }

                    Ok(challenge.to_vec())
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyResponse { challenge },
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
        req: tonic::Request<pb::scufflecloud::core::v1::CreateUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let ip_info = extensions.ip_address_info()?;
        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let code = format!("{:06}", rand::random_range(0..=999999));

        let session_request = UserSessionRequest {
            id: Id::new(),
            device_name: payload.name,
            device_ip: ip_info.to_network(),
            code,
            approved_by: None,
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
        };

        diesel::insert_into(user_session_requests::dsl::user_session_requests)
            .values(&session_request)
            .execute(&mut *db)
            .await
            .into_tonic_internal("failed to insert user session request")?;

        Ok(tonic::Response::new(session_request.into()))
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
