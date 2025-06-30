use argon2::{Argon2, PasswordVerifier};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::chrono_ext::ChronoDateTimeExt;
use crate::http_ext::RequestExt;
use crate::id::Id;
use crate::models::{
    EmailRegistrationRequest, EmailRegistrationRequestId, MagicLinkUserSessionRequest, UserEmail, UserSession,
    UserSessionRequest, UserSessionRequestId,
};
use crate::schema::{
    email_registration_requests, magic_link_user_session_requests, mfa_webauthn_pks, user_emails, user_session_requests,
    user_sessions,
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

        // Check if email is already registered
        let email = common::normalize_email(&payload.email);

        // Generate random code
        let code = crypto::generate_random_bytes().into_tonic_internal("failed to generate registration code")?;
        // let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(&code);

        db.transaction::<_, TxError, _>(move |conn| {
            async move {
                if user_emails::dsl::user_emails
                    .find(&email)
                    .select(UserEmail::as_select())
                    .first::<UserEmail>(conn)
                    .await
                    .optional()
                    .into_tonic_internal("failed to query database")?
                    .is_some()
                {
                    return Err(tonic::Status::with_error_details(
                        Code::AlreadyExists,
                        "email is already registered",
                        ErrorDetails::new(),
                    )
                    .into());
                }

                // Create email registration request
                let registration_request = EmailRegistrationRequest {
                    id: EmailRegistrationRequestId::new(),
                    user_id: None,
                    email,
                    code: code.to_vec(),
                    expires_at: chrono::Utc::now() + global.email_registration_request_validity(),
                };

                diesel::insert_into(email_registration_requests::dsl::email_registration_requests)
                    .values(registration_request)
                    .execute(conn)
                    .await
                    .into_tonic_internal("failed to insert email registration request")?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        // TODO: Send email with registration code link code_base64

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
                        global,
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

                    Ok(common::create_session(global, conn, user.id, device, ip_info).await?)
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
                    expires_at: chrono::Utc::now() + global.magic_link_user_session_request_validity(),
                };
                diesel::insert_into(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
                    .values(session_request)
                    .execute(conn)
                    .await
                    .into_tonic_internal("failed to insert magic link user session request")?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        // TODO: Send email with code

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
                    let new_token = common::create_session(global, conn, session_request.user_id, device, ip_info).await?;

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
                                .eq(chrono::Utc::now() + global.mfa_webauthn_challenge_validity()),
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
            expires_at: chrono::Utc::now() + global.user_session_request_validity(),
        };

        diesel::insert_into(user_session_requests::dsl::user_session_requests)
            .values(&session_request)
            .execute(&mut db)
            .await
            .into_tonic_internal("failed to insert user session request")?;

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn get_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let id: UserSessionRequestId = payload.id.parse().into_tonic_internal("failed to parse id")?;

        let Some(session_request) = user_session_requests::dsl::user_session_requests
            .find(&id)
            .select(UserSessionRequest::as_select())
            .first::<UserSessionRequest>(&mut db)
            .await
            .optional()
            .into_tonic_internal("failed to query user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user session request not found",
                ErrorDetails::new(),
            ));
        };

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn get_user_session_request_by_code(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let Some(session_request) = user_session_requests::dsl::user_session_requests
            .filter(user_session_requests::dsl::code.eq(&payload.code))
            .select(UserSessionRequest::as_select())
            .first::<UserSessionRequest>(&mut db)
            .await
            .optional()
            .into_tonic_internal("failed to query user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user session request not found",
                ErrorDetails::new(),
            ));
        };

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn approve_user_session_request_by_code(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ApproveUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let session = extensions.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let Some(session_request) = diesel::update(user_session_requests::dsl::user_session_requests)
            .filter(user_session_requests::dsl::code.eq(&payload.code))
            .set(user_session_requests::dsl::approved_by.eq(&session.user_id))
            .returning(UserSessionRequest::as_select())
            .get_result::<UserSessionRequest>(&mut db)
            .await
            .optional()
            .into_tonic_internal("failed to update user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user session request not found",
                ErrorDetails::new(),
            ));
        };

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn complete_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let ip_info = extensions.ip_address_info()?;
        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let id: UserSessionRequestId = payload.id.parse().into_tonic_internal("failed to parse id")?;
        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    // Delete user session request
                    let Some(session_request) = diesel::delete(user_session_requests::dsl::user_session_requests)
                        .filter(user_session_requests::dsl::id.eq(id))
                        .returning(UserSessionRequest::as_select())
                        .get_result::<UserSessionRequest>(conn)
                        .await
                        .optional()
                        .into_tonic_internal("failed to delete user session request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown id", ErrorDetails::new()).into(),
                        );
                    };

                    let Some(approved_by) = session_request.approved_by else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::FailedPrecondition,
                            "user session request is not approved yet",
                            ErrorDetails::new(),
                        )
                        .into());
                    };

                    let new_token = common::create_session(global, conn, approved_by, device, ip_info).await?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn refresh_user_session(
        &self,
        req: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = req.global::<G>()?;
        let session = req.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        let token = crypto::generate_random_bytes().into_tonic_internal("failed to generate token")?;
        let encrypted_token = crypto::encrypt_token(session.device_algorithm.into(), &token, &session.device_pk_data)?;

        let session = diesel::update(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&session.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&session.device_fingerprint)),
            )
            .set((
                user_sessions::dsl::token.eq(token),
                user_sessions::dsl::token_expires_at.eq(chrono::Utc::now() + global.user_session_token_validity()),
            ))
            .returning(UserSession::as_select())
            .get_result::<UserSession>(&mut db)
            .await
            .into_tonic_internal("failed to update user session")?;

        let (Some(token_id), Some(token_expires_at)) = (session.token_id, session.token_expires_at) else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::Internal,
                "user session does not have a token",
                ErrorDetails::new(),
            ));
        };

        let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
            id: token_id.to_string(),
            encrypted_token,
            expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
            session_expires_at: Some(session.expires_at.to_prost_timestamp_utc()),
        };

        Ok(tonic::Response::new(new_token))
    }

    async fn invalidate_user_session(&self, req: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        let global = req.global::<G>()?;
        let session = req.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal("failed to connect to database")?;

        diesel::delete(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&session.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&session.device_fingerprint)),
            )
            .execute(&mut db)
            .await
            .into_tonic_internal("failed to update user session")?;

        Ok(tonic::Response::new(()))
    }
}
