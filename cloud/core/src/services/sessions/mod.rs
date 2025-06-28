use base64::Engine;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rand::TryRngCore;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::models::{EmailRegistrationRequest, EmailRegistrationRequestId, User, UserEmail};
use crate::request_ext::RequestExt;
use crate::result_ext::ResultExt;
use crate::schema::{email_registration_requests, user_emails, users};
use crate::services::CoreSvc;
use crate::utils::TxError;
use crate::{CoreConfig, captcha};

mod crypto;
mod registration;

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

                    let (_, new_token) = registration::create_new_user_and_session(
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

        let Some((user, _)) = users::dsl::users
            .inner_join(user_emails::dsl::user_emails.on(users::dsl::primary_email.eq(user_emails::dsl::email)))
            .filter(user_emails::dsl::email.eq(&payload.email))
            .select((User::as_select(), user_emails::dsl::email))
            .first::<(User, String)>(&mut *db)
            .await
            .optional()
            .into_tonic_internal("failed to query user by email")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user not found",
                ErrorDetails::new(),
            ));
        };

        let mut options = vec![pb::scufflecloud::core::v1::LoginWithEmailOptions::MagicLink as i32];

        if user.password_hash.is_some() {
            options.push(pb::scufflecloud::core::v1::LoginWithEmailOptions::Password as i32);
        }

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse { options },
        ))
    }

    async fn login_with_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailRequest>,
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
