use core_db_types::models::{User, UserId, UserSession, UserSessionTokenId};
use core_db_types::schema::user_sessions;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{OptionExt, RequestExt, ResultExt};
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{Action, CoreApplication};
use crate::chrono_ext::ChronoDateTimeExt;
use crate::http_ext::CoreRequestExt;
use crate::operations::{Operation, OperationDriver};
use crate::{common, totp};

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ValidateMfaForUserSessionRequest> {
    type Principal = User;
    type Resource = UserSession;
    type Response = pb::scufflecloud::core::v1::UserSession;

    const ACTION: Action = Action::ValidateMfaForUserSession;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let session = self.session_or_err()?;
        Ok(session.clone())
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let payload = self.into_inner();

        let conn = driver.conn().await?;

        // Verify MFA challenge response
        match payload.response.require("response")? {
            pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::Totp(
                pb::scufflecloud::core::v1::ValidateMfaForUserSessionTotp { code },
            ) => {
                totp::process_token(conn, resource.user_id, &code).await?;
            }
            pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::Webauthn(
                pb::scufflecloud::core::v1::ValidateMfaForUserSessionWebauthn { response_json },
            ) => {
                let pk_cred: webauthn_rs::prelude::PublicKeyCredential = serde_json::from_str(&response_json)
                    .into_tonic_err_with_field_violation("response_json", "invalid public key credential")?;
                common::finish_webauthn_authentication(global, conn, resource.user_id, &pk_cred).await?;
            }
            pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::RecoveryCode(
                pb::scufflecloud::core::v1::ValidateMfaForUserSessionRecoveryCode { code },
            ) => {
                common::process_recovery_code(conn, resource.user_id, &code).await?;
            }
        }

        // Set mfa_pending=false and reset session expiry
        let session = diesel::update(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&resource.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&resource.device_fingerprint)),
            )
            .set((
                user_sessions::dsl::mfa_pending.eq(false),
                user_sessions::dsl::expires_at.eq(chrono::Utc::now() + global.timeout_config().user_session_token),
            ))
            .returning(UserSession::as_select())
            .get_result::<UserSession>(conn)
            .await
            .into_tonic_internal_err("failed to update user session")?;

        Ok(session.into())
    }
}

pub(crate) struct RefreshUserSessionRequest;

impl<G: core_traits::Global> Operation<G> for tonic::Request<RefreshUserSessionRequest> {
    type Principal = User;
    type Resource = UserSession;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::RefreshUserSession;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.expired_session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let session = self.expired_session_or_err()?;
        Ok(session.clone())
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let token_id = UserSessionTokenId::new();
        let token = common::generate_random_bytes().into_tonic_internal_err("failed to generate token")?;
        let encrypted_token = common::encrypt_token(resource.device_algorithm.into(), &token, &resource.device_pk_data)?;
        let conn = driver.conn().await?;

        let session = diesel::update(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&resource.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&resource.device_fingerprint)),
            )
            .set((
                user_sessions::dsl::token_id.eq(token_id),
                user_sessions::dsl::token.eq(token),
                user_sessions::dsl::token_expires_at.eq(chrono::Utc::now() + global.timeout_config().user_session_token),
            ))
            .returning(UserSession::as_select())
            .get_result::<UserSession>(conn)
            .await
            .into_tonic_internal_err("failed to update user session")?;

        let (Some(token_id), Some(token_expires_at)) = (session.token_id, session.token_expires_at) else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::Internal,
                "user session does not have a token",
                ErrorDetails::new(),
            ));
        };

        let mfa_options = if session.mfa_pending {
            common::mfa_options(conn, session.user_id).await?
        } else {
            vec![]
        };

        let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
            id: token_id.to_string(),
            encrypted_token,
            user_id: session.user_id.to_string(),
            expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
            session: Some(session.into()),
            mfa_options: mfa_options.into_iter().map(|o| o as i32).collect(),
        };

        Ok(new_token)
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::InvalidateUserSessionRequest> {
    type Principal = User;
    type Resource = UserSession;
    type Response = ();

    const ACTION: Action = Action::InvalidateUserSession;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        let user_id: UserId = self
            .get_ref()
            .user_id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;
        let device_fingerprint = &self.get_ref().device_fingerprint;

        let session = diesel::delete(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(device_fingerprint)),
            )
            .returning(UserSession::as_select())
            .get_result(driver.conn().await?)
            .await
            .into_tonic_internal_err("failed to delete user session")?;

        Ok(session)
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        Ok(())
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ListUserSessionsRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::ListUserSessionsResponse;

    const ACTION: Action = Action::ListUserSessions;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        _driver: &mut OperationDriver<'_, G>,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let sessions = user_sessions::dsl::user_sessions
            .filter(user_sessions::dsl::user_id.eq(principal.id))
            .load::<UserSession>(&mut db)
            .await
            .into_tonic_internal_err("failed to load user sessions")?;

        Ok(pb::scufflecloud::core::v1::ListUserSessionsResponse {
            sessions: sessions.into_iter().map(Into::into).collect(),
        })
    }
}
