use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use tonic_types::{ErrorDetails, StatusExt};

use crate::http_ext::RequestExt;
use crate::models::{User, UserId, UserSession};
use crate::schema::{mfa_webauthn_pks, users};
use crate::services::CoreSvc;
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, captcha, utils};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::users_service_server::UsersService for CoreSvc<G> {
    async fn get_user(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        if req.extensions().get::<UserSession>().is_none() {
            return Err(tonic::Status::unauthenticated("authentication required"));
        }

        let mut db = req
            .global::<G>()?
            .db()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let user_id: UserId = req
            .get_ref()
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let user = users::dsl::users
            .find(user_id)
            .select(User::as_select())
            .first::<User>(&mut db)
            .await
            .optional()
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found("user not found"))?;

        Ok(tonic::Response::new(user.into()))
    }

    async fn update_user_password(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn update_user_names(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserNamesRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn update_user_primary_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPrimaryEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn list_user_emails(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::ListUserEmailsResponse>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn create_user_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CreateUserEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn complete_create_user_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn delete_user_email(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::DeleteUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }

    async fn create_webauthn_credential_challenge(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateWebauthnCredentialChallengeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CreateWebauthnCredentialChallengeResponse>, tonic::Status> {
        let (_, extensions, payload) = req.into_parts();
        let global = extensions.global::<G>()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        // Check captcha
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let challenge = utils::generate_random_bytes().into_tonic_internal_err("failed to generate webauthn challenge")?;

        let n = diesel::update(mfa_webauthn_pks::dsl::mfa_webauthn_pks)
            .filter(mfa_webauthn_pks::dsl::credential_id.eq(&payload.credential_id))
            .set((
                mfa_webauthn_pks::dsl::current_challenge.eq(&challenge),
                mfa_webauthn_pks::dsl::current_challenge_expires_at.eq(chrono::Utc::now() + global.mfa_timeout()),
            ))
            .execute(&mut db)
            .await
            .into_tonic_internal_err("failed to update webauthn public key")?;

        if n == 0 {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "webauthn public key not found",
                ErrorDetails::new(),
            ));
        }

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::CreateWebauthnCredentialChallengeResponse {
                challenge: challenge.to_vec(),
            },
        ))
    }

    async fn list_user_webauthn_credentials(
        &self,
        _req: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::ListUserWebauthnCredentialsResponse>, tonic::Status> {
        Err(tonic::Status::with_error_details(
            tonic::Code::Unimplemented,
            "this endpoint is not implemented",
            ErrorDetails::new(),
        ))
    }
}
